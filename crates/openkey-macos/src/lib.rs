pub mod config;
pub mod event_tap;
pub mod handler;
pub mod hotkeys;
pub mod sender;
pub mod workaround;

#[cfg(test)]
mod tests {
    use openkey_engine::types::*;

    use super::config::*;
    use super::handler::*;
    use super::hotkeys::*;
    use super::sender::*;
    use super::workaround::*;
    use super::event_tap::*;

    #[test]
    fn re_enables_event_tap_when_macos_disables_by_timeout_or_user_input() {
        let mut handler = MacEventHandler::new(MacConfig::default());
        let mut sender = RecordingEventSender::default();

        let res1 = handler.handle_raw_event(
            MacEventType::TapDisabledByTimeout,
            0,
            0,
            false,
            &mut sender,
        );
        assert_eq!(res1, CallbackAction::PassThrough);
        assert_eq!(sender.tap_reenabled_count, 1);

        let res2 = handler.handle_raw_event(
            MacEventType::TapDisabledByUserInput,
            0,
            0,
            false,
            &mut sender,
        );
        assert_eq!(res2, CallbackAction::PassThrough);
        assert_eq!(sender.tap_reenabled_count, 2);
    }

    #[test]
    fn switches_language_on_hotkey_and_modifiers_release() {
        let mut cfg = MacConfig::default();
        // Ctrl + Shift (0x100 | 0x800) + no key (0xFE)
        cfg.engine.switch_key_status = 0xFE | 0x100 | 0x800 | 0x8000;
        let mut handler = MacEventHandler::new(cfg);
        let mut sender = RecordingEventSender::default();

        assert_eq!(handler.engine.config.language, 1);

        // Press Ctrl, then Ctrl+Shift, then release -> should switch language to 0 and beep
        handler.handle_raw_event(
            MacEventType::FlagsChanged,
            0,
            CG_FLAG_CONTROL,
            false,
            &mut sender,
        );
        handler.handle_raw_event(
            MacEventType::FlagsChanged,
            0,
            CG_FLAG_CONTROL | CG_FLAG_SHIFT,
            false,
            &mut sender,
        );
        let action = handler.handle_raw_event(
            MacEventType::FlagsChanged,
            0,
            CG_FLAG_CONTROL,
            false,
            &mut sender,
        );
        assert_eq!(action, CallbackAction::Swallow);
        assert_eq!(handler.engine.config.language, 0);
        assert_eq!(sender.beep_count, 1);
    }

    #[test]
    fn spotlight_uses_shift_left_selection_replacement_instead_of_backspace() {
        let mut cfg = MacConfig::default();
        cfg.engine.fix_recommend_browser = 1;
        let mut handler = MacEventHandler::new(cfg);
        handler.set_frontmost_app("com.apple.Spotlight", true);
        let mut sender = RecordingEventSender::default();

        // Type 'a' then 's' -> 'á' in Spotlight
        sender.simulate_key_sequence(&mut handler, "as");
        assert_eq!(sender.rendered_text(), "á");
        assert!(sender.actions.contains(&SentAction::ShiftLeftArrow));
        assert!(!sender.actions.contains(&SentAction::Backspace));
    }

    #[test]
    fn browser_autocomplete_workaround_sends_empty_char_or_shift_left_for_chromium() {
        let mut cfg = MacConfig::default();
        cfg.engine.fix_recommend_browser = 1;
        cfg.fix_chromium_browser = false;
        let mut handler = MacEventHandler::new(cfg);
        handler.set_frontmost_app("com.apple.Safari", false);
        let mut sender = RecordingEventSender::default();

        sender.simulate_key_sequence(&mut handler, "tieengs");
        assert_eq!(sender.rendered_text(), "tiếng");
        assert!(sender.actions.contains(&SentAction::EmptyChar(0x202F)));

        // Sublime Text uses 0x200C empty char
        assert_eq!(empty_character_for_app("com.sublimetext.3"), 0x200C);
        assert_eq!(empty_character_for_app("com.apple.Safari"), 0x202F);
    }

    #[test]
    fn smart_switch_key_remembers_per_app_language_and_code_table() {
        let mut cfg = MacConfig::default();
        cfg.engine.use_smart_switch_key = 1;
        cfg.engine.remember_code = 1;
        let mut handler = MacEventHandler::new(cfg);
        let mut sender = RecordingEventSender::default();

        // Start in Safari (Vietnamese = 1, code_table = 0)
        handler.on_active_app_changed("com.apple.Safari", &mut sender);
        assert_eq!(handler.engine.config.language, 1);

        // Switch to Terminal and toggle language to English (0)
        handler.on_active_app_changed("com.apple.Terminal", &mut sender);
        handler.switch_language(&mut sender);
        assert_eq!(handler.engine.config.language, 0);

        // Switch back to Safari -> automatically restores Vietnamese (1)
        handler.on_active_app_changed("com.apple.Safari", &mut sender);
        assert_eq!(handler.engine.config.language, 1);

        // Switch back to Terminal -> automatically restores English (0)
        handler.on_active_app_changed("com.apple.Terminal", &mut sender);
        assert_eq!(handler.engine.config.language, 0);
    }

    #[test]
    fn batches_long_macro_expansions_over_16_utf16_units() {
        let mut cfg = MacConfig::default();
        cfg.engine.use_macro = 1;
        let mut handler = MacEventHandler::new(cfg);
        let long_expansion = "Cộng hòa Xã hội Chủ nghĩa Việt Nam Độc lập Tự do";
        handler.engine.macros.add_macro("chxh", long_expansion, 0);
        handler.set_frontmost_app("com.apple.TextEdit", false);

        let mut sender = RecordingEventSender::default();
        sender.simulate_key_sequence(&mut handler, "chxh ");
        assert_eq!(sender.rendered_text(), format!("{} ", long_expansion));
        // Should have sent multiple UnicodeString chunks of <= 16 UTF-16 code units
        let max_chunk = sender
            .actions
            .iter()
            .filter_map(|a| match a {
                SentAction::UnicodeString(v) => Some(v.len()),
                _ => None,
            })
            .max()
            .unwrap_or(0);
        assert_eq!(max_chunk, 16);
    }

    #[test]
    fn cf_preferences_roundtrips_config_and_binary_blobs() {
        let domain = "com.tuyenmai.openkey.rust_unit_test";
        let mut cfg = MacConfig::default();
        cfg.engine.language = 0;
        cfg.engine.input_type = 1; // VNI
        cfg.engine.code_table = 3; // Unicode Compound
        cfg.engine.quick_telex = 1;
        cfg.engine.use_modern_orthography = 1;
        cfg.fix_chromium_browser = true;
        cfg.perform_layout_compat = true;
        cfg.gray_icon = true;
        cfg.macro_data = vec![1, 0, 2, b'v', b'n', 8, 0, b'V', b'i', b'e', b't', b' ', b'N', b'a', b'm'];
        cfg.smart_switch_data = vec![1, 0, 16, b'c', b'o', b'm', b'.', b'a', b'p', b'p', b'l', b'e', b'.', b'S', b'a', b'f', b'a', b'r', b'i', 1];

        cfg.save_to_cf_preferences(domain);
        let mut loaded = MacConfig::default();
        loaded.load_from_cf_preferences(domain);

        assert_eq!(loaded.engine.language, 0);
        assert_eq!(loaded.engine.input_type, 1);
        assert_eq!(loaded.engine.code_table, 3);
        assert_eq!(loaded.engine.quick_telex, 1);
        assert_eq!(loaded.engine.use_modern_orthography, 1);
        assert!(loaded.fix_chromium_browser);
        assert!(loaded.perform_layout_compat);
        assert!(loaded.gray_icon);
        assert_eq!(loaded.macro_data, cfg.macro_data);
        assert_eq!(loaded.smart_switch_data, cfg.smart_switch_data);
    }

    #[test]
    fn formats_quick_convert_menu_title_matching_app_delegate() {
        assert_eq!(
            format_quick_convert_menu_title(EMPTY_HOTKEY),
            "Chuyển mã nhanh"
        );
        // Control (0x100) + Shift (0x800) + 'c' in top byte ((b'c' as i32) << 24) | KEY_C
        let hk = 0x100 | 0x800 | ((b'c' as i32) << 24) | (KEY_C as i32);
        assert_eq!(
            format_quick_convert_menu_title(hk),
            "Chuyển mã nhanh - [⌃ + ⇧ + C]"
        );
        // Command (0x400) + Space (KEY_SPACE = 49 in top byte)
        let hk_space = 0x400 | ((KEY_SPACE as i32) << 24) | (KEY_SPACE as i32);
        assert_eq!(
            format_quick_convert_menu_title(hk_space),
            "Chuyển mã nhanh - [⌘ + ␣ ]"
        );
    }

    #[test]
    fn skips_vietnamese_conversion_when_other_language_input_source_is_active() {
        let mut cfg = MacConfig::default();
        cfg.engine.other_language = 1;
        let mut handler = MacEventHandler::new(cfg);
        handler.set_current_input_source_is_english(false);

        let mut sender = RecordingEventSender::default();
        sender.simulate_key_sequence(&mut handler, "tieengs");
        // Because current input source is non-English (e.g., Japanese/Chinese IME), OpenKey passes through raw keys
        assert_eq!(sender.rendered_text(), "tieengs");

        // When input source is English, OpenKey converts normally
        let mut sender_en = RecordingEventSender::default();
        handler.set_current_input_source_is_english(true);
        sender_en.simulate_key_sequence(&mut handler, "tieengs");
        assert_eq!(sender_en.rendered_text(), "tiếng");
    }

    #[test]
    fn converts_layout_compat_key_strings_and_queries_macos_window_and_tis_state() {
        assert_eq!(convert_key_string_to_key_code("q", 99), KEY_Q);
        assert_eq!(convert_key_string_to_key_code("A", 99), KEY_A);
        assert_eq!(convert_key_string_to_key_code("{", 99), KEY_LEFT_BRACKET);
        assert_eq!(convert_key_string_to_key_code("unknown", 99), 99);

        // Smoke test live macOS queries (do not panic)
        let _ = is_spotlight_visible();
        let _ = is_current_input_source_english();
    }
}
