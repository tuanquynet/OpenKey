pub const MAX_BUFF: usize = 32;

// macOS Key Codes (from platforms/mac.h)
pub const KEY_ESC: u16 = 53;
pub const KEY_DELETE: u16 = 51;
pub const KEY_TAB: u16 = 48;
pub const KEY_ENTER: u16 = 76;
pub const KEY_RETURN: u16 = 36;
pub const KEY_SPACE: u16 = 49;
pub const KEY_LEFT: u16 = 123;
pub const KEY_RIGHT: u16 = 124;
pub const KEY_DOWN: u16 = 125;
pub const KEY_UP: u16 = 126;

pub const KEY_EMPTY: u16 = 256;
pub const KEY_A: u16 = 0;
pub const KEY_B: u16 = 11;
pub const KEY_C: u16 = 8;
pub const KEY_D: u16 = 2;
pub const KEY_E: u16 = 14;
pub const KEY_F: u16 = 3;
pub const KEY_G: u16 = 5;
pub const KEY_H: u16 = 4;
pub const KEY_I: u16 = 34;
pub const KEY_J: u16 = 38;
pub const KEY_K: u16 = 40;
pub const KEY_L: u16 = 37;
pub const KEY_M: u16 = 46;
pub const KEY_N: u16 = 45;
pub const KEY_O: u16 = 31;
pub const KEY_P: u16 = 35;
pub const KEY_Q: u16 = 12;
pub const KEY_R: u16 = 15;
pub const KEY_S: u16 = 1;
pub const KEY_T: u16 = 17;
pub const KEY_U: u16 = 32;
pub const KEY_V: u16 = 9;
pub const KEY_W: u16 = 13;
pub const KEY_X: u16 = 7;
pub const KEY_Y: u16 = 16;
pub const KEY_Z: u16 = 6;

pub const KEY_1: u16 = 18;
pub const KEY_2: u16 = 19;
pub const KEY_3: u16 = 20;
pub const KEY_4: u16 = 21;
pub const KEY_5: u16 = 23;
pub const KEY_6: u16 = 22;
pub const KEY_7: u16 = 26;
pub const KEY_8: u16 = 28;
pub const KEY_9: u16 = 25;
pub const KEY_0: u16 = 29;

pub const KEY_LEFT_BRACKET: u16 = 33;
pub const KEY_RIGHT_BRACKET: u16 = 30;

pub const KEY_LEFT_SHIFT: u16 = 57;
pub const KEY_RIGHT_SHIFT: u16 = 60;
pub const KEY_DOT: u16 = 47;

pub const KEY_BACKQUOTE: u16 = 50;
pub const KEY_MINUS: u16 = 27;
pub const KEY_EQUALS: u16 = 24;
pub const KEY_BACK_SLASH: u16 = 42;
pub const KEY_SEMICOLON: u16 = 41;
pub const KEY_QUOTE: u16 = 39;
pub const KEY_COMMA: u16 = 43;
pub const KEY_SLASH: u16 = 44;

// Bitmasks (from DataType.h)
pub const CAPS_MASK: u32 = 0x10000;
pub const TONE_MASK: u32 = 0x20000;
pub const TONEW_MASK: u32 = 0x40000;

pub const MARK1_MASK: u32 = 0x80000;
pub const MARK2_MASK: u32 = 0x100000;
pub const MARK3_MASK: u32 = 0x200000;
pub const MARK4_MASK: u32 = 0x400000;
pub const MARK5_MASK: u32 = 0x800000;
pub const MARK_MASK: u32 = 0xF80000;

pub const CHAR_MASK: u32 = 0xFFFF;
pub const STANDALONE_MASK: u32 = 0x1000000;
pub const CHAR_CODE_MASK: u32 = 0x2000000;
pub const PURE_CHARACTER_MASK: u32 = 0x80000000;

pub const END_CONSONANT_MASK: u16 = 0x4000;
pub const CONSONANT_ALLOW_MASK: u16 = 0x8000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum KeyEvent {
    Keyboard = 0,
    Mouse = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum KeyEventState {
    KeyDown = 0,
    KeyUp = 1,
    MouseDown = 2,
    MouseUp = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum InputType {
    Telex = 0,
    Vni = 1,
    SimpleTelex1 = 2,
    SimpleTelex2 = 3,
}

impl InputType {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => InputType::Vni,
            2 => InputType::SimpleTelex1,
            3 => InputType::SimpleTelex2,
            _ => InputType::Telex,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HookCodeState {
    DoNothing = 0,
    WillProcess = 1,
    BreakWord = 2,
    Restore = 3,
    ReplaceMacro = 4,
    RestoreAndStartNewSession = 5,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookState {
    pub code: u8,
    pub backspace_count: u8,
    pub new_char_count: u8,
    pub ext_code: u8,
    pub char_data: [u32; MAX_BUFF],
    pub macro_key: Vec<u32>,
    pub macro_data: Vec<u32>,
}

impl Default for HookState {
    fn default() -> Self {
        Self {
            code: HookCodeState::DoNothing as u8,
            backspace_count: 0,
            new_char_count: 0,
            ext_code: 0,
            char_data: [0; MAX_BUFF],
            macro_key: Vec::new(),
            macro_data: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineConfig {
    pub language: i32,
    pub input_type: i32,
    pub free_mark: i32,
    pub code_table: i32,
    pub switch_key_status: i32,
    pub check_spelling: i32,
    pub use_modern_orthography: i32,
    pub quick_telex: i32,
    pub restore_if_wrong_spelling: i32,
    pub fix_recommend_browser: i32,
    pub use_macro: i32,
    pub use_macro_in_english_mode: i32,
    pub auto_caps_macro: i32,
    pub use_smart_switch_key: i32,
    pub upper_case_first_char: i32,
    pub temp_off_spelling: i32,
    pub allow_consonant_zfwj: i32,
    pub quick_start_consonant: i32,
    pub quick_end_consonant: i32,
    pub remember_code: i32,
    pub other_language: i32,
    pub temp_off_open_key: i32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            language: 1,
            input_type: 0,
            free_mark: 0,
            code_table: 0,
            switch_key_status: 0x7A000206, // default Cmd+Shift or Option+Z
            check_spelling: 1,
            use_modern_orthography: 0,
            quick_telex: 0,
            restore_if_wrong_spelling: 1,
            fix_recommend_browser: 1,
            use_macro: 0,
            use_macro_in_english_mode: 0,
            auto_caps_macro: 0,
            use_smart_switch_key: 1,
            upper_case_first_char: 0,
            temp_off_spelling: 0,
            allow_consonant_zfwj: 0,
            quick_start_consonant: 0,
            quick_end_consonant: 0,
            remember_code: 1,
            other_language: 1,
            temp_off_open_key: 0,
        }
    }
}

#[inline]
pub fn is_consonant(key_code: u16) -> bool {
    !(key_code == KEY_A
        || key_code == KEY_E
        || key_code == KEY_U
        || key_code == KEY_Y
        || key_code == KEY_I
        || key_code == KEY_O)
}

#[inline]
pub fn is_number_key(code: u16) -> bool {
    code == KEY_1
        || code == KEY_2
        || code == KEY_3
        || code == KEY_4
        || code == KEY_5
        || code == KEY_6
        || code == KEY_7
        || code == KEY_8
        || code == KEY_9
        || code == KEY_0
}

#[inline]
pub fn is_double_code(code: i32) -> bool {
    code == 2 || code == 3
}

#[inline]
pub fn is_vni_code(code: i32) -> bool {
    code == 2
}

#[inline]
pub fn is_special_key(key_code: u16, input_type: i32) -> bool {
    match InputType::from_i32(input_type) {
        InputType::Telex => {
            key_code == KEY_W
                || key_code == KEY_E
                || key_code == KEY_R
                || key_code == KEY_O
                || key_code == KEY_LEFT_BRACKET
                || key_code == KEY_RIGHT_BRACKET
                || key_code == KEY_A
                || key_code == KEY_S
                || key_code == KEY_D
                || key_code == KEY_F
                || key_code == KEY_J
                || key_code == KEY_Z
                || key_code == KEY_X
        }
        InputType::Vni => {
            key_code == KEY_1
                || key_code == KEY_2
                || key_code == KEY_3
                || key_code == KEY_4
                || key_code == KEY_5
                || key_code == KEY_6
                || key_code == KEY_7
                || key_code == KEY_8
                || key_code == KEY_9
                || key_code == KEY_0
        }
        InputType::SimpleTelex1 | InputType::SimpleTelex2 => {
            key_code == KEY_W
                || key_code == KEY_E
                || key_code == KEY_R
                || key_code == KEY_O
                || key_code == KEY_A
                || key_code == KEY_S
                || key_code == KEY_D
                || key_code == KEY_F
                || key_code == KEY_J
                || key_code == KEY_Z
                || key_code == KEY_X
        }
    }
}

#[inline]
pub fn get_switch_key(data: i32) -> u16 {
    (data & 0xFF) as u16
}

#[inline]
pub fn has_control(data: i32) -> bool {
    (data & 0x100) != 0
}

#[inline]
pub fn has_option(data: i32) -> bool {
    (data & 0x200) != 0
}

#[inline]
pub fn has_command(data: i32) -> bool {
    (data & 0x400) != 0
}

#[inline]
pub fn has_shift(data: i32) -> bool {
    (data & 0x800) != 0
}

#[inline]
pub fn has_beep(data: i32) -> bool {
    (data & 0x8000) != 0
}
