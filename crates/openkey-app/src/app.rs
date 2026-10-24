use std::path::Path;

use openkey_engine::convert_tool::{convert_util, ConvertToolConfig};
use openkey_engine::macro_store::MacroStore;
use openkey_engine::types::InputType;
use openkey_macos::config::{MacConfig, TABLE_CODE_NAMES};
use openkey_macos::event_tap::{is_accessibility_trusted, CoreGraphicsEventSender, EventTapContext};
use openkey_macos::handler::MacEventHandler;
use openkey_macos::sender::RecordingEventSender;

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
        "  openkey status                                           Show accessibility & table codes",
    ]
    .join("\n")
}

pub fn run_cli(args: &[String]) -> Result<String, String> {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" || args[0] == "help" {
        return Ok(usage());
    }

    match args[0].as_str() {
        "type-sim" => handle_type_sim(&args[1..]),
        "convert" => handle_convert(&args[1..]),
        "macro" => handle_macro(&args[1..]),
        "status" => handle_status(),
        "run" => handle_run(&args[1..]),
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

fn handle_run(args: &[String]) -> Result<String, String> {
    let mut cfg = MacConfig::default();
    let mut i = 0usize;
    while i < args.len() {
        if args[i] == "--config" {
            i += 1;
            let path = args.get(i).ok_or("missing value for --config")?;
            cfg.load_from_file(path)
                .map_err(|e| format!("failed to load config {}: {}", path, e))?;
        }
        i += 1;
    }

    #[cfg(target_os = "macos")]
    unsafe {
        let mut ctx = Box::new(EventTapContext {
            handler: MacEventHandler::new(cfg),
            sender: CoreGraphicsEventSender::new(),
            run_loop_source: std::ptr::null_mut(),
        });
        ctx.start_event_tap()?;
        ctx.run_loop();
        ctx.stop_event_tap();
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
}
