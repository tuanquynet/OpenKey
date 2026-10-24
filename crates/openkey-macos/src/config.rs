use std::fs;
use std::path::Path;

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
            macro_data: Vec::new(),
            smart_switch_data: Vec::new(),
        }
    }
}

impl MacConfig {
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
