use openkey_engine::engine::Engine;
use openkey_engine::types::*;
use openkey_engine::vietnamese::character_to_key_code;

#[repr(C)]
#[derive(Clone, Copy)]
struct CppEngineConfig {
    language: i32,
    input_type: i32,
    free_mark: i32,
    code_table: i32,
    switch_key_status: i32,
    check_spelling: i32,
    use_modern_orthography: i32,
    quick_telex: i32,
    restore_if_wrong_spelling: i32,
    fix_recommend_browser: i32,
    use_macro: i32,
    use_macro_in_english_mode: i32,
    auto_caps_macro: i32,
    use_smart_switch_key: i32,
    upper_case_first_char: i32,
    temp_off_spelling: i32,
    allow_consonant_zfwj: i32,
    quick_start_consonant: i32,
    quick_end_consonant: i32,
    remember_code: i32,
    other_language: i32,
    temp_off_open_key: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CppHookSnapshot {
    code: u8,
    backspace_count: u8,
    new_char_count: u8,
    ext_code: u8,
    char_data: [u32; MAX_BUFF],
    macro_key_len: u32,
    macro_key: [u32; 64],
    macro_data_len: u32,
    macro_data: [u32; 128],
}

impl Default for CppHookSnapshot {
    fn default() -> Self {
        Self {
            code: 0,
            backspace_count: 0,
            new_char_count: 0,
            ext_code: 0,
            char_data: [0; MAX_BUFF],
            macro_key_len: 0,
            macro_key: [0; 64],
            macro_data_len: 0,
            macro_data: [0; 128],
        }
    }
}

#[link(name = "openkey_cpp_ref", kind = "static")]
#[link(name = "c++")]
extern "C" {
    fn cpp_engine_init(cfg: *const CppEngineConfig);
    fn cpp_engine_add_macro(text: *const std::os::raw::c_char, content: *const std::os::raw::c_char);
    fn cpp_engine_handle_event(
        event: i32,
        state: i32,
        data: u16,
        caps_status: u8,
        other_control_key: bool,
        out: *mut CppHookSnapshot,
    );
    fn cpp_engine_english_mode(
        state: i32,
        data: u16,
        is_caps: bool,
        other_control_key: bool,
        out: *mut CppHookSnapshot,
    );
    fn cpp_engine_start_new_session();
    fn cpp_get_macro_save_data(out_buf: *mut u8, max_len: u32) -> u32;
    fn cpp_init_smart_switch_key();
    fn cpp_get_app_input_method_status(bundle_id: *const std::os::raw::c_char, current_input_method: i32) -> i32;
    fn cpp_set_app_input_method_status(bundle_id: *const std::os::raw::c_char, language: i32);
    fn cpp_get_smart_switch_save_data(out_buf: *mut u8, max_len: u32) -> u32;
    fn cpp_convert_util(
        src: *const std::os::raw::c_char,
        from_code: u8,
        to_code: u8,
        all_caps: bool,
        non_caps: bool,
        caps_first: bool,
        caps_each_word: bool,
        remove_mark: bool,
        out_buf: *mut u8,
        max_len: u32,
    ) -> u32;
}

fn to_cpp_config(cfg: &EngineConfig) -> CppEngineConfig {
    CppEngineConfig {
        language: cfg.language,
        input_type: cfg.input_type,
        free_mark: cfg.free_mark,
        code_table: cfg.code_table,
        switch_key_status: cfg.switch_key_status,
        check_spelling: cfg.check_spelling,
        use_modern_orthography: cfg.use_modern_orthography,
        quick_telex: cfg.quick_telex,
        restore_if_wrong_spelling: cfg.restore_if_wrong_spelling,
        fix_recommend_browser: cfg.fix_recommend_browser,
        use_macro: cfg.use_macro,
        use_macro_in_english_mode: cfg.use_macro_in_english_mode,
        auto_caps_macro: cfg.auto_caps_macro,
        use_smart_switch_key: cfg.use_smart_switch_key,
        upper_case_first_char: cfg.upper_case_first_char,
        temp_off_spelling: cfg.temp_off_spelling,
        allow_consonant_zfwj: cfg.allow_consonant_zfwj,
        quick_start_consonant: cfg.quick_start_consonant,
        quick_end_consonant: cfg.quick_end_consonant,
        remember_code: cfg.remember_code,
        other_language: cfg.other_language,
        temp_off_open_key: cfg.temp_off_open_key,
    }
}

fn assert_parity_for_sequence(cfg: EngineConfig, macros: &[(&str, &str)], sequence: &str) {
    let mut rust_engine = Engine::with_config(cfg.clone());
    let cpp_cfg = to_cpp_config(&cfg);
    unsafe {
        cpp_engine_init(&cpp_cfg);
    }
    for &(k, v) in macros {
        rust_engine.macros.add_macro(k, v, cfg.code_table as usize);
        let c_k = std::ffi::CString::new(k).unwrap();
        let c_v = std::ffi::CString::new(v).unwrap();
        unsafe {
            cpp_engine_add_macro(c_k.as_ptr(), c_v.as_ptr());
        }
    }

    for (step_idx, ch) in sequence.chars().enumerate() {
        let (keycode, caps_status) = if ch == '\x08' {
            (KEY_DELETE, 0)
        } else if ch == '\n' {
            (KEY_RETURN, 0)
        } else if ch == '\t' {
            (KEY_TAB, 0)
        } else if ch == '\x1b' {
            (KEY_ESC, 0)
        } else {
            let mapped = character_to_key_code(ch as u32)
                .unwrap_or_else(|| panic!("unmapped char {:?}", ch));
            let kc = (mapped & CHAR_MASK) as u16;
            let caps = if (mapped & CAPS_MASK) != 0 { 1 } else { 0 };
            (kc, caps)
        };

        let mut cpp_snap = CppHookSnapshot::default();
        if cfg.language == 0 {
            rust_engine.english_mode(KeyEventState::KeyDown, keycode, caps_status != 0, false);
            unsafe {
                cpp_engine_english_mode(
                    KeyEventState::KeyDown as i32,
                    keycode,
                    caps_status != 0,
                    false,
                    &mut cpp_snap,
                );
            }
        } else {
            rust_engine.handle_event(
                KeyEvent::Keyboard,
                KeyEventState::KeyDown,
                keycode,
                caps_status,
                false,
            );
            unsafe {
                cpp_engine_handle_event(
                    KeyEvent::Keyboard as i32,
                    KeyEventState::KeyDown as i32,
                    keycode,
                    caps_status,
                    false,
                    &mut cpp_snap,
                );
            }
        }

        let rh = &rust_engine.hook_state;
        assert_eq!(
            rh.code, cpp_snap.code,
            "Mismatch in code at step {} ({:?}) of {:?} under {:?}",
            step_idx, ch, sequence, cfg
        );
        assert_eq!(
            rh.backspace_count, cpp_snap.backspace_count,
            "Mismatch in backspace_count at step {} ({:?}) of {:?}",
            step_idx, ch, sequence
        );
        assert_eq!(
            rh.new_char_count, cpp_snap.new_char_count,
            "Mismatch in new_char_count at step {} ({:?}) of {:?}",
            step_idx, ch, sequence
        );
        assert_eq!(
            rh.ext_code, cpp_snap.ext_code,
            "Mismatch in ext_code at step {} ({:?}) of {:?}",
            step_idx, ch, sequence
        );
        let n = rh.new_char_count as usize;
        assert_eq!(
            &rh.char_data[..n],
            &cpp_snap.char_data[..n],
            "Mismatch in char_data[..{}] at step {} ({:?}) of {:?}",
            n,
            step_idx,
            ch,
            sequence
        );
        let mlen = cpp_snap.macro_data_len as usize;
        assert_eq!(
            &rh.macro_data[..],
            &cpp_snap.macro_data[..mlen],
            "Mismatch in macro_data at step {} ({:?}) of {:?}",
            step_idx,
            ch,
            sequence
        );

        // Mirror OpenKey.mm behavior when RestoreAndStartNewSession is returned
        if rh.code == HookCodeState::RestoreAndStartNewSession as u8 {
            rust_engine.start_new_session();
            unsafe {
                cpp_engine_start_new_session();
            }
        }
    }
}

#[test]
fn cpp_differential_parity_suite() {
    let telex_corpus = [
        "Tieengs Vieetj ",
        "Coongj hoaf xax hooij chur nghiax Vieetj Nam ",
        "DDoojc laapj - Tuwj do - Hanhj phucs ",
        "ddi ddaau ddoos ",
        "nguyeexn ",
        "khuyeens khichs ",
        "thuowng ",
        "huowu ",
        "quyeens rux ",
        "gioongs nhau ",
        "gifn giuwx ",
        "quas khuwss ",
        "hoaf binhf ",
        "hoas hocj ",
        "thuyeetf ",
        "huychj ",
        "as ",
        "uwowng ",
        "w ",
        "[ ",
        "] ",
        "trw ",
        "nhw ",
        "chuwaf ",
        "xooong ",
        "thoongf ",
        // Backspace & word-restore flows
        "tieengs\x08\x08ng ",
        "vieetj \x08\x08t ",
        "anh, \x08\x08em ",
        // Invalid words for restore-if-wrong-spelling
        "stf ",
        "clgt, ",
        "window. ",
        " Words with z removal: tieengsz ",
        // Double key restore: a -> â -> aa, w -> ư -> ww, d -> đ -> dd, s -> sắc -> ss
        "caaa ",
        "twww ",
        "dddi ",
        "tasss ",
    ];

    // 1. Telex across all 5 code tables (0..=4) and both orthographies (0, 1)
    for code_table in 0..=4 {
        for modern in [0, 1] {
            let mut cfg = EngineConfig::default();
            cfg.input_type = InputType::Telex as i32;
            cfg.code_table = code_table;
            cfg.use_modern_orthography = modern;
            for seq in &telex_corpus {
                assert_parity_for_sequence(cfg.clone(), &[], seq);
            }
        }
    }

    // 2. VNI input type
    let vni_corpus = [
        "Tie61ng Vie65t ",
        "Co65ng hoa2 xa4 ho65i chu3 nghi4a ",
        "D9o65c la65p Tu75 do Ha5nh phu1c ",
        "thu7o7ng ",
        "ngu7o72i ",
        "quye61n ",
        "ca66n ",
        "ta11n ",
        "tie61ng0 ",
    ];
    for code_table in [0, 2, 3] {
        let mut cfg = EngineConfig::default();
        cfg.input_type = InputType::Vni as i32;
        cfg.code_table = code_table;
        for seq in &vni_corpus {
            assert_parity_for_sequence(cfg.clone(), &[], seq);
        }
    }

    // 3. Simple Telex 1 & 2
    for input_type in [InputType::SimpleTelex1, InputType::SimpleTelex2] {
        let mut cfg = EngineConfig::default();
        cfg.input_type = input_type as i32;
        for seq in ["tieengs vieetj ", "w [ ] uwowng "] {
            assert_parity_for_sequence(cfg.clone(), &[], seq);
        }
    }

    // 4. Quick Telex, Quick Start/End Consonants, Allow ZFWJ, UpperCaseFirstChar
    {
        let mut cfg = EngineConfig::default();
        cfg.quick_telex = 1;
        cfg.quick_start_consonant = 1;
        cfg.quick_end_consonant = 1;
        cfg.allow_consonant_zfwj = 1;
        cfg.upper_case_first_char = 1;
        for seq in [
            "ccaanf ",
            "ggiaf ",
            "kkhoong ",
            "nnguwowif ",
            "qqua ",
            "pphoos ",
            "tthuwowng ",
            "uung ",
            "fanh ",
            "jang ",
            "wen ",
            "hag ",
            "vih ",
            "bak ",
            "zao. tieeps\nddoongf ",
        ] {
            assert_parity_for_sequence(cfg.clone(), &[], seq);
        }
    }

    // 5. Macros in Vietnamese and English modes with AutoCaps
    {
        let mut cfg = EngineConfig::default();
        cfg.use_macro = 1;
        cfg.auto_caps_macro = 1;
        let macros = &[("vn", "Việt Nam"), ("btw", "by the way")];
        for seq in ["vn ", "VN ", "btw ", "Btw ", "BTW ", "vn, ", "btw. "] {
            assert_parity_for_sequence(cfg.clone(), macros, seq);
        }

        cfg.language = 0;
        cfg.use_macro_in_english_mode = 1;
        for seq in ["vn ", "btw ", "Btw ", "BTW "] {
            assert_parity_for_sequence(cfg.clone(), macros, seq);
        }
    }

    // 6. Long word (> 32 characters) and backspace recovery
    {
        let cfg = EngineConfig::default();
        assert_parity_for_sequence(
            cfg,
            &[],
            "abcdefghijklmnopqrstuvwxyzabcdefghijkl\x08\x08\x08\x08\x08\x08\x08\x08 ",
        );
    }

    // 7. Binary serialization byte-for-byte parity: MacroStore & SmartSwitchStore
    {
        let cfg = EngineConfig::default();
        let cpp_cfg = to_cpp_config(&cfg);
        unsafe {
            cpp_engine_init(&cpp_cfg);
        }
        let mut rust_macros = openkey_engine::macro_store::MacroStore::new();
        for &(k, v) in &[("vn", "Việt Nam"), ("btw", "by the way"), ("ok", "OpenKey")] {
            rust_macros.add_macro(k, v, 0);
            let c_k = std::ffi::CString::new(k).unwrap();
            let c_v = std::ffi::CString::new(v).unwrap();
            unsafe {
                cpp_engine_add_macro(c_k.as_ptr(), c_v.as_ptr());
            }
        }
        let rust_macro_bytes = rust_macros.to_bytes();
        let mut cpp_macro_buf = vec![0u8; 4096];
        let cpp_len = unsafe {
            cpp_get_macro_save_data(cpp_macro_buf.as_mut_ptr(), cpp_macro_buf.len() as u32)
        } as usize;
        assert_eq!(rust_macro_bytes, &cpp_macro_buf[..cpp_len]);

        unsafe {
            cpp_init_smart_switch_key();
        }
        let mut rust_ss = openkey_engine::smart_switch::SmartSwitchStore::new();
        for &(bundle, status) in &[
            ("com.apple.Safari", 1),
            ("com.microsoft.VSCode", 0),
            ("com.google.Chrome", 3),
        ] {
            let c_b = std::ffi::CString::new(bundle).unwrap();
            let r1 = rust_ss.get_app_input_method_status(bundle, status);
            let c1 = unsafe { cpp_get_app_input_method_status(c_b.as_ptr(), status) };
            assert_eq!(r1, c1);
            rust_ss.set_app_input_method_status(bundle, status);
            unsafe { cpp_set_app_input_method_status(c_b.as_ptr(), status) };
        }
        let rust_ss_bytes = rust_ss.to_bytes();
        let mut cpp_ss_buf = vec![0u8; 4096];
        let cpp_ss_len = unsafe {
            cpp_get_smart_switch_save_data(cpp_ss_buf.as_mut_ptr(), cpp_ss_buf.len() as u32)
        } as usize;
        assert_eq!(rust_ss_bytes, &cpp_ss_buf[..cpp_ss_len]);
    }

    // 8. ConvertTool byte-for-byte parity vs C++ convertUtil
    {
        let sample = "Cộng hòa Xã hội Chủ nghĩa Việt Nam. Độc lập - Tự do - Hạnh phúc!\nXin chào thế giới.";
        let c_sample = std::ffi::CString::new(sample).unwrap();
        for from_code in [0u8, 3u8] {
            for to_code in 0..=4u8 {
                for &(all_caps, non_caps, caps_first, caps_each, remove_mark) in &[
                    (false, false, false, false, false),
                    (true, false, false, false, false),
                    (false, true, false, false, false),
                    (false, false, true, false, false),
                    (false, false, false, true, false),
                    (false, false, true, false, true),
                ] {
                    // First convert sample from 0 -> from_code so input is valid in from_code
                    let mut prep_cfg = openkey_engine::convert_tool::ConvertToolConfig::default();
                    prep_cfg.from_code = 0;
                    prep_cfg.to_code = from_code;
                    prep_cfg.to_caps_first_letter = true;
                    let prepared = if from_code == 0 {
                        sample.to_string()
                    } else {
                        openkey_engine::convert_tool::convert_util(sample, &prep_cfg)
                    };
                    let c_prepared = std::ffi::CString::new(prepared.as_str()).unwrap();

                    let cfg = openkey_engine::convert_tool::ConvertToolConfig {
                        dont_alert_when_completed: false,
                        to_all_caps: all_caps,
                        to_all_non_caps: non_caps,
                        to_caps_first_letter: caps_first,
                        to_caps_each_word: caps_each,
                        remove_mark,
                        from_code,
                        to_code,
                        hot_key: 0,
                    };
                    let rust_out = openkey_engine::convert_tool::convert_util(&prepared, &cfg);
                    let mut cpp_buf = vec![0u8; 4096];
                    let cpp_len = unsafe {
                        cpp_convert_util(
                            c_prepared.as_ptr(),
                            from_code,
                            to_code,
                            all_caps,
                            non_caps,
                            caps_first,
                            caps_each,
                            remove_mark,
                            cpp_buf.as_mut_ptr(),
                            cpp_buf.len() as u32,
                        )
                    } as usize;
                    let cpp_out = String::from_utf8_lossy(&cpp_buf[..cpp_len]);
                    assert_eq!(
                        rust_out, cpp_out,
                        "ConvertTool mismatch for {:?}",
                        cfg
                    );
                    let _ = &c_sample;
                }
            }
        }
    }
}
