pub mod engine;
pub mod convert_tool;
pub mod macro_store;
pub mod smart_switch;
pub mod types;
pub mod vietnamese;

#[cfg(test)]
mod tests {
    use super::convert_tool::*;
    use super::macro_store::*;
    use super::smart_switch::*;
    use super::types::*;
    use super::vietnamese::*;

    #[test]
    fn maps_ascii_characters_and_keycodes_roundtrip() {
        assert_eq!(character_to_key_code('a' as u32), Some(KEY_A as u32));
        assert_eq!(character_to_key_code('A' as u32), Some((KEY_A as u32) | CAPS_MASK));
        assert_eq!(key_code_to_character(KEY_A as u32), 'a' as u16);
        assert_eq!(key_code_to_character((KEY_A as u32) | CAPS_MASK), 'A' as u16);
        assert_eq!(key_code_to_character((KEY_1 as u32) | CAPS_MASK), '!' as u16);
    }

    #[test]
    fn resolves_unicode_and_vni_character_codes_with_tones_and_marks() {
        assert_eq!(
            get_character_code((KEY_A as u32) | TONEW_MASK, 0),
            0x0103 | CHAR_CODE_MASK
        );
        assert_eq!(
            get_character_code((KEY_A as u32) | TONE_MASK | MARK1_MASK | CAPS_MASK, 0),
            0x1EA4 | CHAR_CODE_MASK
        );
        assert_eq!(
            get_character_code((KEY_D as u32) | TONE_MASK, 0),
            0x0111 | CHAR_CODE_MASK
        );
        assert_eq!(
            get_character_code((KEY_A as u32) | MARK1_MASK, 2),
            0xF961 | CHAR_CODE_MASK
        );
    }

    #[test]
    fn smart_switch_store_binary_roundtrip_and_caching() {
        let mut store = SmartSwitchStore::new();
        // First query for unknown app inserts current_input_method and returns -1
        assert_eq!(store.get_app_input_method_status("com.apple.Safari", 1), -1);
        // Subsequent query returns cached value 1
        assert_eq!(store.get_app_input_method_status("com.apple.Safari", 0), 1);

        store.set_app_input_method_status("com.microsoft.VSCode", 0);
        assert_eq!(store.get_app_input_method_status("com.microsoft.VSCode", 1), 0);

        let bytes = store.to_bytes();
        let mut restored = SmartSwitchStore::new();
        restored.init_from_bytes(&bytes);
        assert_eq!(restored.get_app_input_method_status("com.apple.Safari", 0), 1);
        assert_eq!(restored.get_app_input_method_status("com.microsoft.VSCode", 1), 0);
    }

    #[test]
    fn macro_store_binary_roundtrip_and_autocaps_lookup() {
        let mut macros = MacroStore::new();
        assert!(macros.add_macro("vn", "Việt Nam", 0));
        assert!(macros.add_macro("btw", "by the way", 0));
        assert!(macros.has_macro("vn", 0));

        // Binary round-trip
        let saved = macros.to_bytes();
        let mut restored = MacroStore::new();
        restored.init_from_bytes(&saved, 0);
        assert!(restored.has_macro("vn", 0));
        assert!(restored.has_macro("btw", 0));

        // Exact lowercase match: "btw"
        let mut key_lower = vec![KEY_B as u32, KEY_T as u32, KEY_W as u32];
        let expanded_lower = restored.find_macro(&mut key_lower, 0, true).expect("should match btw");
        assert_eq!(expanded_lower[0], KEY_B as u32);

        // Title-case match with auto_caps: "Btw" -> "By the way"
        let mut key_title = vec![(KEY_B as u32) | CAPS_MASK, KEY_T as u32, KEY_W as u32];
        let expanded_title = restored.find_macro(&mut key_title, 0, true).expect("should match Btw");
        assert_eq!(expanded_title[0], (KEY_B as u32) | CAPS_MASK);
        assert_eq!(expanded_title[1], KEY_Y as u32);

        // ALL-CAPS match with auto_caps: "BTW" -> "BY THE WAY"
        let mut key_all_caps = vec![
            (KEY_B as u32) | CAPS_MASK,
            (KEY_T as u32) | CAPS_MASK,
            (KEY_W as u32) | CAPS_MASK,
        ];
        let expanded_caps = restored.find_macro(&mut key_all_caps, 0, true).expect("should match BTW");
        assert_eq!(expanded_caps[0], (KEY_B as u32) | CAPS_MASK);
        assert_eq!(expanded_caps[1], (KEY_Y as u32) | CAPS_MASK);
    }

    #[test]
    fn convert_tool_converts_between_code_tables_and_removes_marks() {
        // Remove mark + all caps on Unicode
        let mut cfg = ConvertToolConfig::default();
        cfg.from_code = 0;
        cfg.to_code = 0;
        cfg.remove_mark = true;
        cfg.to_all_caps = true;
        assert_eq!(convert_util("Tiếng Việt", &cfg), "TIENG VIET");

        // Unicode -> Unicode Compound -> Unicode round-trip
        let mut to_compound = ConvertToolConfig::default();
        to_compound.from_code = 0;
        to_compound.to_code = 3;
        to_compound.to_caps_first_letter = true;
        let compound_str = convert_util("Cộng hòa xã hội", &to_compound);
        assert_ne!(compound_str, "Cộng hòa xã hội");

        let mut back_to_unicode = ConvertToolConfig::default();
        back_to_unicode.from_code = 3;
        back_to_unicode.to_code = 0;
        back_to_unicode.to_caps_first_letter = true;
        assert_eq!(convert_util(&compound_str, &back_to_unicode), "Cộng hòa xã hội");
    }
}
