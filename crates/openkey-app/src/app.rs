use std::path::Path;

use openkey_engine::convert_tool::{convert_util, ConvertToolConfig};
use openkey_engine::macro_store::MacroStore;
use openkey_engine::types::InputType;
use openkey_macos::config::{MacConfig, TABLE_CODE_NAMES};
use openkey_macos::event_tap::{is_accessibility_trusted, CoreGraphicsEventSender, EventTapContext};
use openkey_macos::handler::MacEventHandler;
use openkey_macos::sender::RecordingEventSender;
use crate::ui::{
    create_macos_app_bundle, on_runtime_status_changed, register_active_runtime,
    run_cocoa_app_loop, setup_cocoa_status_bar, AppUiController,
};

pub fn usage() -> String {
    [
        "OpenKey (Rust macOS Port)",
        "",
        "Usage:",
        "  openkey run [--config <path>]                            Start macOS CGEventTap daemon",
        "  openkey type-sim [--input telex|vni|simple1|simple2] [--modern] <keys>",
        "                                                           Simulate typing a key sequence",
        "  openkey convert [--from 0..4] [--to 0..4] [--remove-mark] [--all-caps] [--non-caps] [--caps-first] [--caps-each] <text>",
        "                                                           Convert Vietnamese text encoding/casing",
        "  openkey macro [--file <path>] list                       List macros in file",
        "  openkey macro [--file <path>] add <key> <expansion>      Add/update macro in file",
        "  openkey macro [--file <path>] delete <key>               Delete macro from file",
        "  openkey bundle [--out-dir <dir>]                         Build macOS OpenKey.app bundle",
        "  openkey status                                           Show accessibility & table codes",
    ]
    .join("\n")
}

pub fn run_cli(args: &[String]) -> Result<String, String> {
    if args.is_empty() || args[0].starts_with("-psn_") {
        return handle_run(args);
    }
    if args[0] == "--help" || args[0] == "-h" || args[0] == "help" {
        return Ok(usage());
    }

    match args[0].as_str() {
        "type-sim" => handle_type_sim(&args[1..]),
        "convert" => handle_convert(&args[1..]),
        "macro" => handle_macro(&args[1..]),
        "status" => handle_status(),
        "run" => handle_run(&args[1..]),
        "bundle" => handle_bundle(&args[1..]),
        other => Err(format!("unknown command {:?}\n\n{}", other, usage())),
    }
}

fn handle_type_sim(args: &[String]) -> Result<String, String> {
    let mut cfg = MacConfig::default();
    cfg.engine.fix_recommend_browser = 0;
    let mut i = 0usize;
    let mut sequence: Option<String> = None;

    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                i += 1;
                let val = args.get(i).ok_or("missing value for --input")?;
                cfg.engine.input_type = match val.as_str() {
                    "telex" => InputType::Telex as i32,
                    "vni" => InputType::Vni as i32,
                    "simple1" => InputType::SimpleTelex1 as i32,
                    "simple2" => InputType::SimpleTelex2 as i32,
                    _ => return Err(format!("invalid input type: {}", val)),
                };
            }
            "--modern" => {
                cfg.engine.use_modern_orthography = 1;
            }
            "--quick-telex" => {
                cfg.engine.quick_telex = 1;
            }
            s => {
                sequence = Some(s.to_string());
            }
        }
        i += 1;
    }

    let seq = sequence.ok_or("missing key sequence argument for type-sim")?;
    let mut handler = MacEventHandler::new(cfg);
    let mut sender = RecordingEventSender::default();
    sender.simulate_key_sequence(&mut handler, &seq);
    Ok(sender.rendered_text())
}

fn handle_convert(args: &[String]) -> Result<String, String> {
    let mut cfg = ConvertToolConfig::default();
    let mut i = 0usize;
    let mut text: Option<String> = None;

    while i < args.len() {
        match args[i].as_str() {
            "--from" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or("missing value for --from")?
                    .parse::<u8>()
                    .map_err(|_| "invalid --from code table index")?;
                cfg.from_code = val;
            }
            "--to" => {
                i += 1;
                let val = args
                    .get(i)
                    .ok_or("missing value for --to")?
                    .parse::<u8>()
                    .map_err(|_| "invalid --to code table index")?;
                cfg.to_code = val;
            }
            "--remove-mark" => cfg.remove_mark = true,
            "--all-caps" => cfg.to_all_caps = true,
            "--non-caps" => cfg.to_all_non_caps = true,
            "--caps-first" => cfg.to_caps_first_letter = true,
            "--caps-each" => cfg.to_caps_each_word = true,
            s => text = Some(s.to_string()),
        }
        i += 1;
    }

    let src = text.ok_or("missing text argument for convert")?;
    Ok(convert_util(&src, &cfg))
}

fn handle_macro(args: &[String]) -> Result<String, String> {
    let mut file_path = "openkey_macros.txt".to_string();
    let mut i = 0usize;
    while i < args.len() {
        if args[i] == "--file" {
            i += 1;
            file_path = args
                .get(i)
                .ok_or("missing value for --file")?
                .to_string();
            i += 1;
        } else {
            break;
        }
    }

    let sub = args.get(i).ok_or("missing macro subcommand (list|add|delete)")?;
    let mut store = MacroStore::new();
    if Path::new(&file_path).exists() {
        store
            .read_from_file(&file_path, false, 0)
            .map_err(|e| format!("failed to read macro file: {}", e))?;
    }

    match sub.as_str() {
        "list" => {
            let entries: Vec<String> = store
                .get_all_macros()
                .into_iter()
                .map(|(_, k, v)| format!("{}:{}", k, v))
                .collect();
            Ok(entries.join("\n"))
        }
        "add" => {
            let k = args.get(i + 1).ok_or("missing macro key")?;
            let v = args.get(i + 2).ok_or("missing macro expansion")?;
            store.add_macro(k, v, 0);
            store
                .save_to_file(&file_path)
                .map_err(|e| format!("failed to save macro file: {}", e))?;
            Ok(format!("added macro: {} -> {}", k, v))
        }
        "delete" => {
            let k = args.get(i + 1).ok_or("missing macro key")?;
            let removed = store.delete_macro(k, 0);
            store
                .save_to_file(&file_path)
                .map_err(|e| format!("failed to save macro file: {}", e))?;
            Ok(format!("deleted macro {}: {}", k, removed))
        }
        other => Err(format!("unknown macro subcommand: {}", other)),
    }
}

fn handle_status() -> Result<String, String> {
    let trusted = is_accessibility_trusted();
    let tables = TABLE_CODE_NAMES
        .iter()
        .enumerate()
        .map(|(idx, name)| format!("  {}: {}", idx, name))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!(
        "Accessibility trusted: {}\nSupported code tables:\n{}",
        trusted, tables
    ))
}

fn handle_bundle(args: &[String]) -> Result<String, String> {
    let mut out_dir = std::path::PathBuf::from("target/release");
    let mut i = 0usize;
    while i < args.len() {
        if args[i] == "--out-dir" {
            i += 1;
            let val = args.get(i).ok_or("missing value for --out-dir")?;
            out_dir = std::path::PathBuf::from(val);
        }
        i += 1;
    }
    let bundle_path = create_macos_app_bundle(&out_dir)?;
    Ok(format!("Built macOS app bundle at {}", bundle_path.display()))
}

fn handle_run(args: &[String]) -> Result<String, String> {
    let mut ui = AppUiController::load_from_domain(openkey_macos::config::OPENKEY_BUNDLE);
    let mut i = 0usize;
    while i < args.len() {
        if args[i] == "--config" {
            i += 1;
            let path = args.get(i).ok_or("missing value for --config")?;
            ui.config
                .load_from_file(path)
                .map_err(|e| format!("failed to load config {}: {}", path, e))?;
            ui.handler = MacEventHandler::new(ui.config.clone());
        }
        i += 1;
    }

    #[cfg(target_os = "macos")]
    unsafe {
        let status_item = setup_cocoa_status_bar(&ui);
        let mut sender = CoreGraphicsEventSender::new();
        sender.convert_tool_config = ui.handler.engine.convert_tool.clone();
        sender.on_status_changed = Some(on_runtime_status_changed);
        let ctx_ptr = Box::into_raw(Box::new(EventTapContext {
            handler: ui.handler,
            sender,
            run_loop_source: std::ptr::null_mut(),
        }));
        register_active_runtime(ctx_ptr, status_item);
        let _ = (*ctx_ptr).start_event_tap();
        run_cocoa_app_loop();
        (*ctx_ptr).stop_event_tap();
    }
    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(items: &[&str]) -> Vec<String> {
        items.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn cli_type_sim_converts_telex_and_vni_sequences() {
        let out = run_cli(&s(&["type-sim", "--input", "telex", "Coongj hoaf xax hooij"]))
            .expect("type-sim telex should succeed");
        assert_eq!(out, "Cộng hòa xã hội");

        let out_vni = run_cli(&s(&["type-sim", "--input", "vni", "Tie61ng Vie65t"]))
            .expect("type-sim vni should succeed");
        assert_eq!(out_vni, "Tiếng Việt");
    }

    #[test]
    fn cli_convert_removes_marks_and_changes_case() {
        let out = run_cli(&s(&[
            "convert",
            "--remove-mark",
            "--all-caps",
            "Tiếng Việt",
        ]))
        .expect("convert should succeed");
        assert_eq!(out, "TIENG VIET");
    }

    #[test]
    fn cli_macro_export_and_import_roundtrip() {
        let tmp = std::env::temp_dir().join("openkey_rust_macro_test.txt");
        let tmp_str = tmp.to_string_lossy().to_string();
        let _ = std::fs::remove_file(&tmp);
        let out = run_cli(&s(&[
            "macro",
            "--file",
            &tmp_str,
            "add",
            "vn",
            "Việt Nam",
        ]))
        .expect("macro add should succeed");
        assert!(out.contains("vn -> Việt Nam"));

        let list_out = run_cli(&s(&["macro", "--file", &tmp_str, "list"]))
            .expect("macro list should succeed");
        assert!(list_out.contains("vn:Việt Nam"));
        let _ = std::fs::remove_file(tmp);
    }

    #[test]
    fn menu_bar_and_control_panel_controller_syncs_state_and_preferences() {
        use crate::ui::AppUiController;

        let domain = "com.tuyenmai.openkey.rust_ui_test";
        let mut ui = AppUiController::with_domain(domain);
        ui.load_default_config();

        let snap = ui.menu_snapshot();
        assert!(snap.vietnamese_enabled);
        assert_eq!(snap.status_icon_name, "Status");
        assert_eq!(snap.status_title, "V");
        assert_eq!(snap.selected_input_type, 0);
        assert_eq!(snap.selected_code_table, 0);
        assert!(snap.is_template_icon);

        // Toggle input method to English
        ui.on_input_method_selected();
        let snap2 = ui.menu_snapshot();
        assert!(!snap2.vietnamese_enabled);
        assert_eq!(snap2.status_icon_name, "StatusEng");
        assert_eq!(snap2.status_title, "E");

        // Change input type to VNI (1) and code table to Unicode Compound (3)
        ui.on_input_type_selected(1);
        ui.on_code_table_changed(3);
        let snap3 = ui.menu_snapshot();
        assert_eq!(snap3.selected_input_type, 1);
        assert_eq!(snap3.selected_code_table, 3);

        // Update switch key modifiers (Control + Shift + Beep + 'z')
        ui.set_switch_key_control(true);
        ui.set_switch_key_option(false);
        ui.set_switch_key_command(false);
        ui.set_switch_key_shift(true);
        ui.set_switch_key_beep(true);
        ui.set_switch_key_char(6, b'z' as u16);

        // Reload from CFPreferences into a fresh controller to verify persistence
        let reloaded = AppUiController::load_from_domain(domain);
        assert_eq!(reloaded.handler.engine.config.language, 0);
        assert_eq!(reloaded.handler.engine.config.input_type, 1);
        assert_eq!(reloaded.handler.engine.config.code_table, 3);
        assert_ne!(reloaded.handler.engine.config.switch_key_status & 0x100, 0);
        assert_eq!(reloaded.handler.engine.config.switch_key_status & 0x200, 0);
        assert_ne!(reloaded.handler.engine.config.switch_key_status & 0x800, 0);
        assert_ne!(reloaded.handler.engine.config.switch_key_status & 0x8000, 0);
        assert_eq!(reloaded.handler.engine.config.switch_key_status & 0xFF, 6);
    }

    #[test]
    fn builds_macos_openkey_app_bundle_with_plist_and_resources() {
        let tmp_dir = std::env::temp_dir().join("openkey_rust_bundle_test");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        let out_str = tmp_dir.to_string_lossy().to_string();

        let res = run_cli(&s(&["bundle", "--out-dir", &out_str]))
            .expect("bundle command should succeed");
        assert!(res.contains("OpenKey.app"));

        let app_dir = tmp_dir.join("OpenKey.app").join("Contents");
        assert!(app_dir.join("Info.plist").exists());
        assert!(app_dir.join("Resources").join("Status.png").exists());
        assert!(app_dir.join("Resources").join("StatusEng.png").exists());

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn control_panel_and_convert_tool_options_enforce_mutual_exclusion_and_persist() {
        use crate::ui::AppUiController;

        let domain = "com.tuyenmai.openkey.rust_vc_test";
        let mut ui = AppUiController::with_domain(domain);
        ui.load_default_config();

        ui.set_free_mark(true);
        ui.set_modern_orthography(true);
        ui.set_check_spelling(false);
        ui.set_quick_telex(true);
        ui.set_restore_if_invalid_word(true);
        ui.set_temp_off_spell_checking(true);
        ui.set_allow_zfwj(true);
        ui.set_fix_recommend_browser(true);
        ui.set_use_macro(true);
        ui.set_use_macro_in_english_mode(true);
        ui.set_auto_caps_macro(true);
        ui.set_send_key_step_by_step(true);
        ui.set_perform_layout_compat(true);
        ui.set_use_smart_switch_key(true);
        ui.set_upper_case_first_char(true);
        ui.set_quick_start_consonant(true);
        ui.set_quick_end_consonant(true);
        ui.set_temp_off_open_key(true);
        ui.set_remember_table_code(true);
        ui.set_other_language(true);
        ui.set_fix_chromium_browser(true);
        ui.set_show_icon_on_dock(true);
        ui.set_gray_icon(false);
        ui.set_run_on_startup(false);
        ui.set_show_ui_on_startup(true);
        ui.set_check_new_version_on_startup(false);

        // ConvertTool casing options are mutually exclusive (matching turnOffAllOption in ConvertToolViewController.mm)
        ui.set_convert_to_all_caps(true);
        assert!(ui.handler.engine.convert_tool.to_all_caps);
        ui.set_convert_to_non_caps(true);
        assert!(!ui.handler.engine.convert_tool.to_all_caps);
        assert!(ui.handler.engine.convert_tool.to_all_non_caps);
        ui.set_convert_to_caps_first_letter(true);
        assert!(!ui.handler.engine.convert_tool.to_all_non_caps);
        assert!(ui.handler.engine.convert_tool.to_caps_first_letter);
        ui.set_convert_to_caps_each_word(true);
        assert!(!ui.handler.engine.convert_tool.to_caps_first_letter);
        assert!(ui.handler.engine.convert_tool.to_caps_each_word);

        ui.set_convert_from_code(0);
        ui.set_convert_to_code(2);
        ui.reverse_convert_codes();
        assert_eq!(ui.handler.engine.convert_tool.from_code, 2);
        assert_eq!(ui.handler.engine.convert_tool.to_code, 0);

        let reloaded = AppUiController::load_from_domain(domain);
        assert_eq!(reloaded.handler.engine.config.free_mark, 1);
        assert_eq!(reloaded.handler.engine.config.use_modern_orthography, 1);
        assert_eq!(reloaded.handler.engine.config.check_spelling, 0);
        assert_eq!(reloaded.handler.engine.config.quick_telex, 1);
        assert_eq!(reloaded.handler.engine.config.restore_if_wrong_spelling, 1);
        assert_eq!(reloaded.handler.engine.config.allow_consonant_zfwj, 1);
        assert_eq!(reloaded.handler.engine.config.use_macro_in_english_mode, 1);
        assert_eq!(reloaded.handler.engine.config.auto_caps_macro, 1);
        assert!(reloaded.config.send_key_step_by_step);
        assert!(reloaded.config.perform_layout_compat);
        assert!(reloaded.config.fix_chromium_browser);
        assert!(reloaded.config.show_on_dock);
        assert!(!reloaded.config.gray_icon);
        assert!(!reloaded.config.run_on_startup);
        assert!(reloaded.config.show_ui_on_startup);
        assert!(reloaded.config.dont_check_update);
        assert_eq!(reloaded.handler.engine.convert_tool.from_code, 2);
        assert_eq!(reloaded.handler.engine.convert_tool.to_code, 0);
        assert!(reloaded.handler.engine.convert_tool.to_caps_each_word);
    }

    #[test]
    fn registers_objc_delegate_class_with_menu_and_window_selectors() {
        use crate::ui::register_objc_delegate_class;

        let cls_ptr = register_objc_delegate_class();
        assert!(!cls_ptr.is_null());
        // Calling a second time is idempotent and returns the same registered Class
        assert_eq!(register_objc_delegate_class(), cls_ptr);
    }
}
