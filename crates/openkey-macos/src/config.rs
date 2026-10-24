use std::ffi::c_void;
use std::fs;
use std::path::Path;
use std::ptr;

use openkey_engine::convert_tool::ConvertToolConfig;
use openkey_engine::types::*;

use crate::hotkeys::EMPTY_HOTKEY;

pub const OPENKEY_BUNDLE: &str = "com.tuyenmai.openkey";

pub const TABLE_CODE_NAMES: [&str; 5] = [
    "Unicode",
    "TCVN3 (ABC)",
    "VNI Windows",
    "Unicode tổ hợp",
    "Vietnamese Locale CP 1258",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacConfig {
    pub engine: EngineConfig,
    pub convert_tool: ConvertToolConfig,
    pub send_key_step_by_step: bool,
    pub fix_chromium_browser: bool,
    pub perform_layout_compat: bool,
    pub run_on_startup: bool,
    pub gray_icon: bool,
    pub show_on_dock: bool,
    pub show_ui_on_startup: bool,
    pub dont_check_update: bool,
    pub macro_data: Vec<u8>,
    pub smart_switch_data: Vec<u8>,
}

impl Default for MacConfig {
    fn default() -> Self {
        let mut convert_tool = ConvertToolConfig::default();
        convert_tool.hot_key = EMPTY_HOTKEY;
        Self {
            engine: EngineConfig::default(),
            convert_tool,
            send_key_step_by_step: false,
            fix_chromium_browser: false,
            perform_layout_compat: false,
            run_on_startup: false,
            gray_icon: false,
            show_on_dock: false,
            show_ui_on_startup: false,
            dont_check_update: false,
            macro_data: Vec::new(),
            smart_switch_data: Vec::new(),
        }
    }
}

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const c_void);
    fn CFStringCreateWithBytes(
        alloc: *const c_void,
        bytes: *const u8,
        num_bytes: isize,
        encoding: u32,
        is_external_representation: u8,
    ) -> *const c_void;
    fn CFNumberCreate(
        allocator: *const c_void,
        the_type: isize,
        value_ptr: *const c_void,
    ) -> *const c_void;
    fn CFNumberGetValue(
        number: *const c_void,
        the_type: isize,
        value_ptr: *mut c_void,
    ) -> u8;
    fn CFDataCreate(
        allocator: *const c_void,
        bytes: *const u8,
        length: isize,
    ) -> *const c_void;
    fn CFDataGetLength(the_data: *const c_void) -> isize;
    fn CFDataGetBytePtr(the_data: *const c_void) -> *const u8;
    fn CFPreferencesSetAppValue(
        key: *const c_void,
        value: *const c_void,
        application_id: *const c_void,
    );
    fn CFPreferencesCopyAppValue(
        key: *const c_void,
        application_id: *const c_void,
    ) -> *const c_void;
    fn CFPreferencesAppSynchronize(application_id: *const c_void) -> u8;
}

#[cfg(target_os = "macos")]
const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
#[cfg(target_os = "macos")]
const K_CF_NUMBER_SINT32_TYPE: isize = 3;

#[cfg(target_os = "macos")]
unsafe fn cf_string(s: &str) -> *const c_void {
    CFStringCreateWithBytes(
        ptr::null(),
        s.as_ptr(),
        s.len() as isize,
        K_CF_STRING_ENCODING_UTF8,
        0,
    )
}

#[cfg(target_os = "macos")]
unsafe fn cf_pref_set_i32(domain: *const c_void, key: &str, val: i32) {
    let k = cf_string(key);
    let num = CFNumberCreate(
        ptr::null(),
        K_CF_NUMBER_SINT32_TYPE,
        (&val as *const i32) as *const c_void,
    );
    CFPreferencesSetAppValue(k, num, domain);
    CFRelease(num);
    CFRelease(k);
}

#[cfg(target_os = "macos")]
unsafe fn cf_pref_get_i32(domain: *const c_void, key: &str) -> Option<i32> {
    let k = cf_string(key);
    let val_ref = CFPreferencesCopyAppValue(k, domain);
    CFRelease(k);
    if val_ref.is_null() {
        return None;
    }
    let mut out: i32 = 0;
    let ok = CFNumberGetValue(
        val_ref,
        K_CF_NUMBER_SINT32_TYPE,
        (&mut out as *mut i32) as *mut c_void,
    );
    CFRelease(val_ref);
    if ok != 0 {
        Some(out)
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
unsafe fn cf_pref_set_bytes(domain: *const c_void, key: &str, bytes: &[u8]) {
    let k = cf_string(key);
    let data = CFDataCreate(ptr::null(), bytes.as_ptr(), bytes.len() as isize);
    CFPreferencesSetAppValue(k, data, domain);
    CFRelease(data);
    CFRelease(k);
}

#[cfg(target_os = "macos")]
unsafe fn cf_pref_get_bytes(domain: *const c_void, key: &str) -> Option<Vec<u8>> {
    let k = cf_string(key);
    let val_ref = CFPreferencesCopyAppValue(k, domain);
    CFRelease(k);
    if val_ref.is_null() {
        return None;
    }
    let len = CFDataGetLength(val_ref);
    let ptr = CFDataGetBytePtr(val_ref);
    let out = if len > 0 && !ptr.is_null() {
        std::slice::from_raw_parts(ptr, len as usize).to_vec()
    } else {
        Vec::new()
    };
    CFRelease(val_ref);
    Some(out)
}

impl MacConfig {
    pub fn save_to_cf_preferences(&self, bundle_domain: &str) {
        #[cfg(target_os = "macos")]
        unsafe {
            let domain = cf_string(bundle_domain);
            cf_pref_set_i32(domain, "InputMethod", self.engine.language);
            cf_pref_set_i32(domain, "InputType", self.engine.input_type);
            cf_pref_set_i32(domain, "FreeMark", self.engine.free_mark);
            cf_pref_set_i32(domain, "CodeTable", self.engine.code_table);
            cf_pref_set_i32(domain, "SwitchKeyStatus", self.engine.switch_key_status);
            cf_pref_set_i32(domain, "Spelling", self.engine.check_spelling);
            cf_pref_set_i32(domain, "ModernOrthography", self.engine.use_modern_orthography);
            cf_pref_set_i32(domain, "QuickTelex", self.engine.quick_telex);
            cf_pref_set_i32(
                domain,
                "RestoreIfInvalidWord",
                self.engine.restore_if_wrong_spelling,
            );
            cf_pref_set_i32(
                domain,
                "FixRecommendBrowser",
                self.engine.fix_recommend_browser,
            );
            cf_pref_set_i32(domain, "UseMacro", self.engine.use_macro);
            cf_pref_set_i32(
                domain,
                "UseMacroInEnglishMode",
                self.engine.use_macro_in_english_mode,
            );
            cf_pref_set_i32(domain, "vAutoCapsMacro", self.engine.auto_caps_macro);
            cf_pref_set_i32(
                domain,
                "SendKeyStepByStep",
                if self.send_key_step_by_step { 1 } else { 0 },
            );
            cf_pref_set_i32(domain, "UseSmartSwitchKey", self.engine.use_smart_switch_key);
            cf_pref_set_i32(
                domain,
                "UpperCaseFirstChar",
                self.engine.upper_case_first_char,
            );
            cf_pref_set_i32(domain, "vTempOffSpelling", self.engine.temp_off_spelling);
            cf_pref_set_i32(
                domain,
                "vAllowConsonantZFWJ",
                self.engine.allow_consonant_zfwj,
            );
            cf_pref_set_i32(
                domain,
                "vQuickStartConsonant",
                self.engine.quick_start_consonant,
            );
            cf_pref_set_i32(
                domain,
                "vQuickEndConsonant",
                self.engine.quick_end_consonant,
            );
            cf_pref_set_i32(domain, "vRememberCode", self.engine.remember_code);
            cf_pref_set_i32(domain, "vOtherLanguage", self.engine.other_language);
            cf_pref_set_i32(domain, "vTempOffOpenKey", self.engine.temp_off_open_key);
            cf_pref_set_i32(
                domain,
                "vFixChromiumBrowser",
                if self.fix_chromium_browser { 1 } else { 0 },
            );
            cf_pref_set_i32(
                domain,
                "vPerformLayoutCompat",
                if self.perform_layout_compat { 1 } else { 0 },
            );
            cf_pref_set_i32(
                domain,
                "RunOnStartup",
                if self.run_on_startup { 1 } else { 0 },
            );
            cf_pref_set_i32(domain, "GrayIcon", if self.gray_icon { 1 } else { 0 });
            cf_pref_set_i32(
                domain,
                "vShowIconOnDock",
                if self.show_on_dock { 1 } else { 0 },
            );
            cf_pref_set_i32(
                domain,
                "ShowUIOnStartup",
                if self.show_ui_on_startup { 1 } else { 0 },
            );
            cf_pref_set_i32(
                domain,
                "DontCheckUpdate",
                if self.dont_check_update { 1 } else { 0 },
            );

            // ConvertTool keys
            cf_pref_set_i32(
                domain,
                "convertToolDontAlertWhenCompleted",
                if self.convert_tool.dont_alert_when_completed {
                    0
                } else {
                    1
                },
            );
            cf_pref_set_i32(
                domain,
                "convertToolToAllCaps",
                if self.convert_tool.to_all_caps { 1 } else { 0 },
            );
            cf_pref_set_i32(
                domain,
                "convertToolToAllNonCaps",
                if self.convert_tool.to_all_non_caps {
                    1
                } else {
                    0
                },
            );
            cf_pref_set_i32(
                domain,
                "convertToolToCapsFirstLetter",
                if self.convert_tool.to_caps_first_letter {
                    1
                } else {
                    0
                },
            );
            cf_pref_set_i32(
                domain,
                "convertToolToCapsEachWord",
                if self.convert_tool.to_caps_each_word {
                    1
                } else {
                    0
                },
            );
            cf_pref_set_i32(
                domain,
                "convertToolRemoveMark",
                if self.convert_tool.remove_mark { 1 } else { 0 },
            );
            cf_pref_set_i32(
                domain,
                "convertToolFromCode",
                self.convert_tool.from_code as i32,
            );
            cf_pref_set_i32(
                domain,
                "convertToolToCode",
                self.convert_tool.to_code as i32,
            );
            cf_pref_set_i32(domain, "convertToolHotKey", self.convert_tool.hot_key);

            cf_pref_set_bytes(domain, "macroData", &self.macro_data);
            cf_pref_set_bytes(domain, "smartSwitchKey", &self.smart_switch_data);

            CFPreferencesAppSynchronize(domain);
            CFRelease(domain);
        }
    }

    pub fn load_from_cf_preferences(&mut self, bundle_domain: &str) {
        #[cfg(target_os = "macos")]
        unsafe {
            let domain = cf_string(bundle_domain);
            if let Some(v) = cf_pref_get_i32(domain, "InputMethod") {
                self.engine.language = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "InputType") {
                self.engine.input_type = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "FreeMark") {
                self.engine.free_mark = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "CodeTable") {
                self.engine.code_table = v.max(0);
            }
            if let Some(v) = cf_pref_get_i32(domain, "SwitchKeyStatus") {
                if v != 0 {
                    self.engine.switch_key_status = v;
                }
            }
            if let Some(v) = cf_pref_get_i32(domain, "Spelling") {
                self.engine.check_spelling = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "ModernOrthography") {
                self.engine.use_modern_orthography = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "QuickTelex") {
                self.engine.quick_telex = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "RestoreIfInvalidWord") {
                self.engine.restore_if_wrong_spelling = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "FixRecommendBrowser") {
                self.engine.fix_recommend_browser = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "UseMacro") {
                self.engine.use_macro = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "UseMacroInEnglishMode") {
                self.engine.use_macro_in_english_mode = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "vAutoCapsMacro") {
                self.engine.auto_caps_macro = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "SendKeyStepByStep") {
                self.send_key_step_by_step = v != 0;
            }
            if let Some(v) = cf_pref_get_i32(domain, "UseSmartSwitchKey") {
                self.engine.use_smart_switch_key = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "UpperCaseFirstChar") {
                self.engine.upper_case_first_char = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "vTempOffSpelling") {
                self.engine.temp_off_spelling = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "vAllowConsonantZFWJ") {
                self.engine.allow_consonant_zfwj = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "vQuickStartConsonant") {
                self.engine.quick_start_consonant = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "vQuickEndConsonant") {
                self.engine.quick_end_consonant = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "vRememberCode") {
                self.engine.remember_code = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "vOtherLanguage") {
                self.engine.other_language = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "vTempOffOpenKey") {
                self.engine.temp_off_open_key = v;
            }
            if let Some(v) = cf_pref_get_i32(domain, "vFixChromiumBrowser") {
                self.fix_chromium_browser = v != 0;
            }
            if let Some(v) = cf_pref_get_i32(domain, "vPerformLayoutCompat") {
                self.perform_layout_compat = v != 0;
            }
            if let Some(v) = cf_pref_get_i32(domain, "RunOnStartup") {
                self.run_on_startup = v != 0;
            }
            if let Some(v) = cf_pref_get_i32(domain, "GrayIcon") {
                self.gray_icon = v != 0;
            }
            if let Some(v) = cf_pref_get_i32(domain, "vShowIconOnDock") {
                self.show_on_dock = v != 0;
            }
            if let Some(v) = cf_pref_get_i32(domain, "ShowUIOnStartup") {
                self.show_ui_on_startup = v != 0;
            }
            if let Some(v) = cf_pref_get_i32(domain, "DontCheckUpdate") {
                self.dont_check_update = v != 0;
            }

            if let Some(v) = cf_pref_get_i32(domain, "convertToolDontAlertWhenCompleted") {
                self.convert_tool.dont_alert_when_completed = v == 0;
            }
            if let Some(v) = cf_pref_get_i32(domain, "convertToolToAllCaps") {
                self.convert_tool.to_all_caps = v != 0;
            }
            if let Some(v) = cf_pref_get_i32(domain, "convertToolToAllNonCaps") {
                self.convert_tool.to_all_non_caps = v != 0;
            }
            if let Some(v) = cf_pref_get_i32(domain, "convertToolToCapsFirstLetter") {
                self.convert_tool.to_caps_first_letter = v != 0;
            }
            if let Some(v) = cf_pref_get_i32(domain, "convertToolToCapsEachWord") {
                self.convert_tool.to_caps_each_word = v != 0;
            }
            if let Some(v) = cf_pref_get_i32(domain, "convertToolRemoveMark") {
                self.convert_tool.remove_mark = v != 0;
            }
            if let Some(v) = cf_pref_get_i32(domain, "convertToolFromCode") {
                self.convert_tool.from_code = v.max(0) as u8;
            }
            if let Some(v) = cf_pref_get_i32(domain, "convertToolToCode") {
                self.convert_tool.to_code = v.max(0) as u8;
            }
            if let Some(v) = cf_pref_get_i32(domain, "convertToolHotKey") {
                self.convert_tool.hot_key = if v == 0 { EMPTY_HOTKEY } else { v };
            }

            if let Some(bytes) = cf_pref_get_bytes(domain, "macroData") {
                self.macro_data = bytes;
            }
            if let Some(bytes) = cf_pref_get_bytes(domain, "smartSwitchKey") {
                self.smart_switch_data = bytes;
            }

            CFRelease(domain);
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let content = format!(
            "vLanguage={}\n\
             vInputType={}\n\
             vFreeMark={}\n\
             vCodeTable={}\n\
             vSwitchKeyStatus={}\n\
             vCheckSpelling={}\n\
             vUseModernOrthography={}\n\
             vQuickTelex={}\n\
             vRestoreIfWrongSpelling={}\n\
             vFixRecommendBrowser={}\n\
             vUseMacro={}\n\
             vUseMacroInEnglishMode={}\n\
             vAutoCapsMacro={}\n\
             vUseSmartSwitchKey={}\n\
             vUpperCaseFirstChar={}\n\
             vTempOffSpelling={}\n\
             vAllowConsonantZFWJ={}\n\
             vQuickStartConsonant={}\n\
             vQuickEndConsonant={}\n\
             vRememberCode={}\n\
             vOtherLanguage={}\n\
             vTempOffOpenKey={}\n\
             vSendKeyStepByStep={}\n\
             vFixChromiumBrowser={}\n\
             vPerformLayoutCompat={}\n",
            self.engine.language,
            self.engine.input_type,
            self.engine.free_mark,
            self.engine.code_table,
            self.engine.switch_key_status,
            self.engine.check_spelling,
            self.engine.use_modern_orthography,
            self.engine.quick_telex,
            self.engine.restore_if_wrong_spelling,
            self.engine.fix_recommend_browser,
            self.engine.use_macro,
            self.engine.use_macro_in_english_mode,
            self.engine.auto_caps_macro,
            self.engine.use_smart_switch_key,
            self.engine.upper_case_first_char,
            self.engine.temp_off_spelling,
            self.engine.allow_consonant_zfwj,
            self.engine.quick_start_consonant,
            self.engine.quick_end_consonant,
            self.engine.remember_code,
            self.engine.other_language,
            self.engine.temp_off_open_key,
            if self.send_key_step_by_step { 1 } else { 0 },
            if self.fix_chromium_browser { 1 } else { 0 },
            if self.perform_layout_compat { 1 } else { 0 },
        );
        fs::write(path, content)
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let content = fs::read_to_string(path)?;
        for line in content.lines() {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let Ok(val) = v.trim().parse::<i32>() else {
                continue;
            };
            match k.trim() {
                "vLanguage" => self.engine.language = val,
                "vInputType" => self.engine.input_type = val,
                "vFreeMark" => self.engine.free_mark = val,
                "vCodeTable" => self.engine.code_table = val.max(0),
                "vSwitchKeyStatus" => self.engine.switch_key_status = val,
                "vCheckSpelling" => self.engine.check_spelling = val,
                "vUseModernOrthography" => self.engine.use_modern_orthography = val,
                "vQuickTelex" => self.engine.quick_telex = val,
                "vRestoreIfWrongSpelling" => self.engine.restore_if_wrong_spelling = val,
                "vFixRecommendBrowser" => self.engine.fix_recommend_browser = val,
                "vUseMacro" => self.engine.use_macro = val,
                "vUseMacroInEnglishMode" => self.engine.use_macro_in_english_mode = val,
                "vAutoCapsMacro" => self.engine.auto_caps_macro = val,
                "vUseSmartSwitchKey" => self.engine.use_smart_switch_key = val,
                "vUpperCaseFirstChar" => self.engine.upper_case_first_char = val,
                "vTempOffSpelling" => self.engine.temp_off_spelling = val,
                "vAllowConsonantZFWJ" => self.engine.allow_consonant_zfwj = val,
                "vQuickStartConsonant" => self.engine.quick_start_consonant = val,
                "vQuickEndConsonant" => self.engine.quick_end_consonant = val,
                "vRememberCode" => self.engine.remember_code = val,
                "vOtherLanguage" => self.engine.other_language = val,
                "vTempOffOpenKey" => self.engine.temp_off_open_key = val,
                "vSendKeyStepByStep" => self.send_key_step_by_step = val != 0,
                "vFixChromiumBrowser" => self.fix_chromium_browser = val != 0,
                "vPerformLayoutCompat" => self.perform_layout_compat = val != 0,
                _ => {}
            }
        }
        Ok(())
    }
}

pub fn convert_key_string_to_key_code(key_string: &str, fallback: u16) -> u16 {
    let lower = key_string.to_ascii_lowercase();
    match lower.as_str() {
        "`" | "~" => KEY_BACKQUOTE,
        "1" | "!" => KEY_1,
        "2" | "@" => KEY_2,
        "3" | "#" => KEY_3,
        "4" | "$" => KEY_4,
        "5" | "%" => KEY_5,
        "6" | "^" => KEY_6,
        "7" | "&" => KEY_7,
        "8" | "*" => KEY_8,
        "9" | "(" => KEY_9,
        "0" | ")" => KEY_0,
        "-" | "_" => KEY_MINUS,
        "=" | "+" => KEY_EQUALS,
        "q" => KEY_Q,
        "w" => KEY_W,
        "e" => KEY_E,
        "r" => KEY_R,
        "t" => KEY_T,
        "y" => KEY_Y,
        "u" => KEY_U,
        "i" => KEY_I,
        "o" => KEY_O,
        "p" => KEY_P,
        "[" | "{" => KEY_LEFT_BRACKET,
        "]" | "}" => KEY_RIGHT_BRACKET,
        "\\" | "|" => KEY_BACK_SLASH,
        "a" => KEY_A,
        "s" => KEY_S,
        "d" => KEY_D,
        "f" => KEY_F,
        "g" => KEY_G,
        "h" => KEY_H,
        "j" => KEY_J,
        "k" => KEY_K,
        "l" => KEY_L,
        ";" | ":" => KEY_SEMICOLON,
        "'" | "\"" => KEY_QUOTE,
        "z" => KEY_Z,
        "x" => KEY_X,
        "c" => KEY_C,
        "v" => KEY_V,
        "b" => KEY_B,
        "n" => KEY_N,
        "m" => KEY_M,
        "," | "<" => KEY_COMMA,
        "." | ">" => KEY_DOT,
        "/" | "?" => KEY_SLASH,
        _ => fallback,
    }
}
