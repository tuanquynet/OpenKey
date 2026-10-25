use std::collections::VecDeque;

use crate::convert_tool::ConvertToolConfig;
use crate::macro_store::MacroStore;
use crate::smart_switch::SmartSwitchStore;
use crate::types::*;
use crate::vietnamese::*;

const CHAR_KEY_CODES: &[u16] = &[
    KEY_BACKQUOTE,
    KEY_1,
    KEY_2,
    KEY_3,
    KEY_4,
    KEY_5,
    KEY_6,
    KEY_7,
    KEY_8,
    KEY_9,
    KEY_0,
    KEY_MINUS,
    KEY_EQUALS,
    KEY_LEFT_BRACKET,
    KEY_RIGHT_BRACKET,
    KEY_BACK_SLASH,
    KEY_SEMICOLON,
    KEY_QUOTE,
    KEY_COMMA,
    KEY_DOT,
    KEY_SLASH,
];

const BREAK_CODES: &[u16] = &[
    KEY_ESC,
    KEY_TAB,
    KEY_ENTER,
    KEY_RETURN,
    KEY_LEFT,
    KEY_RIGHT,
    KEY_DOWN,
    KEY_UP,
    KEY_COMMA,
    KEY_DOT,
    KEY_SLASH,
    KEY_SEMICOLON,
    KEY_QUOTE,
    KEY_BACK_SLASH,
    KEY_MINUS,
    KEY_EQUALS,
    KEY_BACKQUOTE,
    KEY_TAB,
];

const MACRO_BREAK_CODES: &[u16] = &[
    KEY_RETURN,
    KEY_COMMA,
    KEY_DOT,
    KEY_SLASH,
    KEY_SEMICOLON,
    KEY_QUOTE,
    KEY_BACK_SLASH,
    KEY_MINUS,
    KEY_EQUALS,
];

const PROCESSING_CHAR: [[u16; 11]; 4] = [
    [
        KEY_S, KEY_F, KEY_R, KEY_X, KEY_J, KEY_A, KEY_O, KEY_E, KEY_W, KEY_D, KEY_Z,
    ], // Telex
    [
        KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_6, KEY_7, KEY_8, KEY_9, KEY_0,
    ], // VNI
    [
        KEY_S, KEY_F, KEY_R, KEY_X, KEY_J, KEY_A, KEY_O, KEY_E, KEY_W, KEY_D, KEY_Z,
    ], // Simple Telex 1
    [
        KEY_S, KEY_F, KEY_R, KEY_X, KEY_J, KEY_A, KEY_O, KEY_E, KEY_W, KEY_D, KEY_Z,
    ], // Simple Telex 2
];

#[derive(Debug, Clone)]
pub struct Engine {
    pub config: EngineConfig,
    pub hook_state: HookState,
    pub macros: MacroStore,
    pub smart_switch: SmartSwitchStore,
    pub convert_tool: ConvertToolConfig,

    typing_word: [u32; MAX_BUFF],
    index: u8,
    long_word_helper: Vec<u32>,
    typing_states: VecDeque<Vec<u32>>,
    typing_states_data: Vec<u32>,

    key_states: [u32; MAX_BUFF],
    state_index: u8,

    temp_disable_key: bool,
    is_correct: bool,
    is_changed: bool,
    vowel_count: u8,
    vowel_start_index: u8,
    vowel_end_index: u8,
    vowel_will_set_mark: u8,
    is_restored_w: bool,
    key_for_aeo: u16,
    is_checked_grammar: bool,
    is_caps: bool,
    space_count: usize,
    has_handled_macro: bool,
    upper_case_status: u8,
    is_char_key_code: bool,
    special_char: Vec<u32>,
    use_spell_checking_before: bool,
    has_handled_quick_consonant: bool,
    will_temp_off_engine: bool,

    spelling_ok: bool,
    spelling_flag: bool,
    spelling_vowel_ok: bool,
    spelling_end_index: u8,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self::with_config(EngineConfig::default())
    }

    pub fn with_config(config: EngineConfig) -> Self {
        let use_spell_checking_before = config.check_spelling != 0;
        Self {
            config,
            hook_state: HookState::default(),
            macros: MacroStore::new(),
            smart_switch: SmartSwitchStore::new(),
            convert_tool: ConvertToolConfig::default(),
            typing_word: [0; MAX_BUFF],
            index: 0,
            long_word_helper: Vec::new(),
            typing_states: VecDeque::new(),
            typing_states_data: Vec::new(),
            key_states: [0; MAX_BUFF],
            state_index: 0,
            temp_disable_key: false,
            is_correct: false,
            is_changed: false,
            vowel_count: 0,
            vowel_start_index: 0,
            vowel_end_index: 0,
            vowel_will_set_mark: 0,
            is_restored_w: false,
            key_for_aeo: 0,
            is_checked_grammar: false,
            is_caps: false,
            space_count: 0,
            has_handled_macro: false,
            upper_case_status: 0,
            is_char_key_code: false,
            special_char: Vec::new(),
            use_spell_checking_before,
            has_handled_quick_consonant: false,
            will_temp_off_engine: false,
            spelling_ok: false,
            spelling_flag: false,
            spelling_vowel_ok: false,
            spelling_end_index: 0,
        }
    }

    #[inline]
    fn input_type_idx(&self) -> usize {
        match self.config.input_type {
            1 => 1,
            2 => 2,
            3 => 3,
            _ => 0,
        }
    }

    #[inline]
    fn chr(&self, idx: usize) -> u16 {
        self.typing_word[idx] as u16
    }

    #[inline]
    fn get_char_code(&self, data: u32) -> u32 {
        get_character_code(data, self.config.code_table as usize)
    }

    #[inline]
    fn is_key_z(&self, key: u16) -> bool {
        PROCESSING_CHAR[self.input_type_idx()][10] == key
    }

    #[inline]
    fn is_key_d(&self, key: u16) -> bool {
        PROCESSING_CHAR[self.input_type_idx()][9] == key
    }

    #[inline]
    fn is_key_w(&self, key: u16) -> bool {
        let it = self.input_type_idx();
        if it != (InputType::Vni as usize) {
            PROCESSING_CHAR[it][8] == key
        } else {
            PROCESSING_CHAR[it][8] == key || PROCESSING_CHAR[it][7] == key
        }
    }

    #[inline]
    fn is_key_double(&self, key: u16) -> bool {
        let it = self.input_type_idx();
        if it != (InputType::Vni as usize) {
            PROCESSING_CHAR[it][5] == key
                || PROCESSING_CHAR[it][6] == key
                || PROCESSING_CHAR[it][7] == key
        } else {
            PROCESSING_CHAR[it][6] == key
        }
    }

    #[inline]
    fn is_key_s(&self, key: u16) -> bool {
        PROCESSING_CHAR[self.input_type_idx()][0] == key
    }

    #[inline]
    fn is_key_f(&self, key: u16) -> bool {
        PROCESSING_CHAR[self.input_type_idx()][1] == key
    }

    #[inline]
    fn is_key_r(&self, key: u16) -> bool {
        PROCESSING_CHAR[self.input_type_idx()][2] == key
    }

    #[inline]
    fn is_key_x(&self, key: u16) -> bool {
        PROCESSING_CHAR[self.input_type_idx()][3] == key
    }

    #[inline]
    fn is_key_j(&self, key: u16) -> bool {
        PROCESSING_CHAR[self.input_type_idx()][4] == key
    }

    #[inline]
    fn is_mark_key(&self, key_code: u16) -> bool {
        if self.config.input_type != (InputType::Vni as i32) {
            key_code == KEY_S
                || key_code == KEY_F
                || key_code == KEY_R
                || key_code == KEY_J
                || key_code == KEY_X
        } else {
            key_code == KEY_1
                || key_code == KEY_2
                || key_code == KEY_3
                || key_code == KEY_5
                || key_code == KEY_4
        }
    }

    #[inline]
    fn is_bracket_key(key: u16) -> bool {
        key == KEY_LEFT_BRACKET || key == KEY_RIGHT_BRACKET
    }

    #[inline]
    fn is_quick_telex_key(&self, code: u16) -> bool {
        self.index > 0
            && (code == KEY_C
                || code == KEY_G
                || code == KEY_K
                || code == KEY_N
                || code == KEY_Q
                || code == KEY_P
                || code == KEY_T)
            && (self.typing_word[(self.index - 1) as usize] as u16) == code
    }

    #[inline]
    fn is_word_break(event: KeyEvent, _state: KeyEventState, data: u16) -> bool {
        if event == KeyEvent::Mouse {
            return true;
        }
        BREAK_CODES.contains(&data)
    }

    #[inline]
    fn is_macro_break_code(data: u16) -> bool {
        MACRO_BREAK_CODES.contains(&data)
    }

    #[inline]
    fn set_key_data(&mut self, index: u8, key_code: u16, is_caps: bool) {
        let idx = index as usize;
        if idx >= MAX_BUFF {
            return;
        }
        self.typing_word[idx] = (key_code as u32) | if is_caps { CAPS_MASK } else { 0 };
    }

    pub fn init(&mut self) {
        self.index = 0;
        self.state_index = 0;
        self.use_spell_checking_before = self.config.check_spelling != 0;
        self.typing_states_data.clear();
        self.typing_states.clear();
        self.long_word_helper.clear();
    }

    pub fn start_new_session(&mut self) {
        self.index = 0;
        self.hook_state.backspace_count = 0;
        self.hook_state.new_char_count = 0;
        self.temp_disable_key = false;
        self.state_index = 0;
        self.has_handled_macro = false;
        self.has_handled_quick_consonant = false;
        self.long_word_helper.clear();
    }

    pub fn temp_off_spell_checking(&mut self) {
        if self.use_spell_checking_before {
            self.config.check_spelling = if self.config.check_spelling != 0 { 0 } else { 1 };
        }
    }

    pub fn set_check_spelling(&mut self) {
        self.use_spell_checking_before = self.config.check_spelling != 0;
    }

    pub fn temp_off_engine(&mut self, off: bool) {
        self.will_temp_off_engine = off;
    }

    fn check_spelling(&mut self, force_check_vowel: bool) {
        self.spelling_ok = false;
        self.spelling_vowel_ok = true;
        self.spelling_end_index = self.index;

        if self.index > 0 && self.chr((self.index - 1) as usize) == KEY_RIGHT_BRACKET {
            self.spelling_end_index = self.index - 1;
        }

        let end_idx = self.spelling_end_index as usize;
        if end_idx > 0 {
            let mut j = 0usize;
            if is_consonant(self.chr(0)) {
                for cons in CONSONANT_TABLE {
                    self.spelling_flag = false;
                    if end_idx < cons.len() {
                        self.spelling_flag = true;
                    }
                    j = 0;
                    while j < cons.len() {
                        if end_idx > j {
                            let mask_qs = if self.config.quick_start_consonant != 0 {
                                END_CONSONANT_MASK
                            } else {
                                0
                            };
                            let mask_allow = if self.config.allow_consonant_zfwj != 0 {
                                CONSONANT_ALLOW_MASK
                            } else {
                                0
                            };
                            if (cons[j] & !mask_qs) != self.chr(j)
                                && (cons[j] & !mask_allow) != self.chr(j)
                            {
                                self.spelling_flag = true;
                                break;
                            }
                        }
                        j += 1;
                    }
                    if self.spelling_flag {
                        continue;
                    }
                    break;
                }
            }

            if j == end_idx {
                self.spelling_ok = true;
            }

            let mut k = j;
            self.vowel_start_index = k as u8;
            // August 23rd, 2019: fix case "que't"
            if self.chr(self.vowel_start_index as usize) == KEY_U
                && k > 0
                && k + 1 < end_idx
                && self.chr((self.vowel_start_index - 1) as usize) == KEY_Q
            {
                k += 1;
                j = k;
                self.vowel_start_index = k as u8;
            } else if self.index >= 2
                && self.chr(0) == KEY_G
                && self.chr(1) == KEY_I
                && is_consonant(self.chr(2))
            {
                self.vowel_start_index = 1;
                k = 1;
                j = 1;
            }

            for _ in 0..3 {
                if k < end_idx && !is_consonant(self.chr(k)) {
                    k += 1;
                    self.vowel_end_index = k as u8;
                }
            }

            if k > j {
                self.spelling_vowel_ok = false;
                if k - j > 1 && force_check_vowel {
                    let vowel_set = vowel_combine_table(self.chr(j));
                    for vs in vowel_set {
                        self.spelling_flag = false;
                        let mut ii = 1usize;
                        while ii < vs.len() {
                            if j + ii - 1 < end_idx {
                                let idx = j + ii - 1;
                                let combined = (self.chr(idx) as u32)
                                    | (self.typing_word[idx] & TONEW_MASK)
                                    | (self.typing_word[idx] & TONE_MASK);
                                if vs[ii] != combined {
                                    self.spelling_flag = true;
                                    break;
                                }
                            }
                            ii += 1;
                        }
                        if self.spelling_flag
                            || (k < end_idx && vs[0] == 0)
                            || (j + ii - 1 < end_idx && !is_consonant(self.chr(j + ii - 1)))
                        {
                            continue;
                        }
                        self.spelling_vowel_ok = true;
                        break;
                    }
                } else if !is_consonant(self.chr(j)) {
                    self.spelling_vowel_ok = true;
                }

                for end_cons in END_CONSONANT_TABLE {
                    self.spelling_flag = false;
                    let mut jc = 0usize;
                    while jc < end_cons.len() {
                        if end_idx > k + jc {
                            let mask_qe = if self.config.quick_end_consonant != 0 {
                                END_CONSONANT_MASK
                            } else {
                                0
                            };
                            if (end_cons[jc] & !mask_qe) != self.chr(k + jc) {
                                self.spelling_flag = true;
                                break;
                            }
                        }
                        jc += 1;
                    }
                    if self.spelling_flag {
                        continue;
                    }
                    if k + jc >= end_idx {
                        self.spelling_ok = true;
                        break;
                    }
                }

                if self.spelling_ok {
                    let idx_usize = self.index as usize;
                    if idx_usize >= 3
                        && self.chr(idx_usize - 1) == KEY_H
                        && self.chr(idx_usize - 2) == KEY_C
                        && !((self.typing_word[idx_usize - 3] & MARK1_MASK) != 0
                            || (self.typing_word[idx_usize - 3] & MARK5_MASK) != 0
                            || (self.typing_word[idx_usize - 3] & MARK_MASK) == 0)
                    {
                        self.spelling_ok = false;
                    } else if idx_usize >= 2
                        && self.chr(idx_usize - 1) == KEY_T
                        && !((self.typing_word[idx_usize - 2] & MARK1_MASK) != 0
                            || (self.typing_word[idx_usize - 2] & MARK5_MASK) != 0
                            || (self.typing_word[idx_usize - 2] & MARK_MASK) == 0)
                    {
                        self.spelling_ok = false;
                    }
                }
            }
        } else {
            self.spelling_ok = true;
        }
        self.temp_disable_key = !(self.spelling_ok && self.spelling_vowel_ok);
    }

    fn check_grammar(&mut self, delta_back_space: i32) {
        if self.index <= 1 || (self.index as usize) >= MAX_BUFF {
            return;
        }

        self.find_and_calculate_vowel(true);
        if self.vowel_count == 0 {
            return;
        }

        self.is_checked_grammar = false;
        let l = self.vowel_start_index as usize;

        if self.index >= 3 {
            for i in (0..(self.index as usize)).rev() {
                let c = self.chr(i);
                if c == KEY_N
                    || c == KEY_C
                    || c == KEY_I
                    || c == KEY_M
                    || c == KEY_P
                    || c == KEY_T
                {
                    if i >= 2 && self.chr(i - 1) == KEY_O && self.chr(i - 2) == KEY_U {
                        if ((self.typing_word[i - 1] & TONEW_MASK) != 0)
                            ^ ((self.typing_word[i - 2] & TONEW_MASK) != 0)
                        {
                            self.typing_word[i - 2] |= TONEW_MASK;
                            self.typing_word[i - 1] |= TONEW_MASK;
                            self.is_checked_grammar = true;
                            break;
                        }
                    }
                }
            }
        }

        if self.index >= 2 {
            let vei = self.vowel_end_index as usize;
            for i in l..=vei {
                if (self.typing_word[i] & MARK_MASK) != 0 {
                    let mark = self.typing_word[i] & MARK_MASK;
                    self.typing_word[i] &= !MARK_MASK;
                    self.insert_mark(mark, false);
                    if i != (self.vowel_will_set_mark as usize) {
                        self.is_checked_grammar = true;
                    }
                    break;
                }
            }
        }

        if self.is_checked_grammar {
            if self.hook_state.code == (HookCodeState::DoNothing as u8) {
                self.hook_state.code = HookCodeState::WillProcess as u8;
            }
            self.hook_state.backspace_count = 0;

            let idx_usize = self.index as usize;
            for i in (l..idx_usize).rev() {
                self.hook_state.backspace_count += 1;
                self.hook_state.char_data[idx_usize - 1 - i] =
                    self.get_char_code(self.typing_word[i]);
            }
            self.hook_state.new_char_count = self.hook_state.backspace_count;
            self.hook_state.backspace_count =
                ((self.hook_state.backspace_count as i32) + delta_back_space) as u8;
            self.hook_state.ext_code = 4;
        }
    }

    fn insert_key(&mut self, key_code: u16, is_caps: bool, is_check_spelling: bool) {
        if (self.index as usize) >= MAX_BUFF {
            self.long_word_helper.push(self.typing_word[0]);
            for iii in 0..(MAX_BUFF - 1) {
                self.typing_word[iii] = self.typing_word[iii + 1];
            }
            let pos = self.index - 1;
            self.set_key_data(pos, key_code, is_caps);
        } else {
            let pos = self.index;
            self.index += 1;
            self.set_key_data(pos, key_code, is_caps);
        }

        if self.config.check_spelling != 0 && is_check_spelling {
            self.check_spelling(false);
        }

        if key_code == KEY_D
            && self.index >= 2
            && is_consonant(self.chr((self.index - 2) as usize))
        {
            self.temp_disable_key = false;
        }
    }

    fn insert_state(&mut self, key_code: u16, is_caps: bool) {
        let val = (key_code as u32) | if is_caps { CAPS_MASK } else { 0 };
        if (self.state_index as usize) >= MAX_BUFF {
            for iii in 0..(MAX_BUFF - 1) {
                self.key_states[iii] = self.key_states[iii + 1];
            }
            self.key_states[(self.state_index - 1) as usize] = val;
        } else {
            self.key_states[self.state_index as usize] = val;
            self.state_index += 1;
        }
    }

    fn save_word(&mut self) {
        if self.hook_state.code != (HookCodeState::ReplaceMacro as u8) {
            if self.index > 0 {
                if !self.long_word_helper.is_empty() {
                    self.typing_states_data.clear();
                    for (i, &w) in self.long_word_helper.iter().enumerate() {
                        if i != 0 && i % MAX_BUFF == 0 {
                            self.typing_states
                                .push_back(self.typing_states_data.clone());
                            self.typing_states_data.clear();
                        }
                        self.typing_states_data.push(w);
                    }
                    self.typing_states
                        .push_back(self.typing_states_data.clone());
                    self.long_word_helper.clear();
                }

                self.typing_states_data.clear();
                for i in 0..(self.index as usize) {
                    self.typing_states_data.push(self.typing_word[i]);
                }
                self.typing_states
                    .push_back(self.typing_states_data.clone());
            }
        } else {
            self.typing_states_data.clear();
            for (i, &w) in self.hook_state.macro_data.iter().enumerate() {
                if i != 0 && i % MAX_BUFF == 0 {
                    self.typing_states
                        .push_back(self.typing_states_data.clone());
                    self.typing_states_data.clear();
                }
                self.typing_states_data.push(w);
            }
            self.typing_states
                .push_back(self.typing_states_data.clone());
        }
    }

    fn save_word_repeated(&mut self, key_code: u32, count: usize) {
        self.typing_states_data.clear();
        for _ in 0..count {
            self.typing_states_data.push(key_code);
        }
        self.typing_states
            .push_back(self.typing_states_data.clone());
    }

    fn save_special_char(&mut self) {
        self.typing_states_data.clear();
        self.typing_states_data.extend_from_slice(&self.special_char);
        self.typing_states
            .push_back(self.typing_states_data.clone());
        self.special_char.clear();
    }

    fn restore_last_typing_state(&mut self) {
        if let Some(data) = self.typing_states.pop_back() {
            self.typing_states_data = data;
            if !self.typing_states_data.is_empty() {
                if self.typing_states_data[0] == (KEY_SPACE as u32) {
                    self.space_count = self.typing_states_data.len();
                    self.index = 0;
                } else if CHAR_KEY_CODES.contains(&(self.typing_states_data[0] as u16)) {
                    self.index = 0;
                    self.special_char = self.typing_states_data.clone();
                    self.check_spelling(false);
                } else {
                    for (i, &val) in self.typing_states_data.iter().enumerate() {
                        if i < MAX_BUFF {
                            self.typing_word[i] = val;
                        }
                    }
                    self.index = self.typing_states_data.len() as u8;
                }
            }
        }
    }

    fn check_correct_vowel(&mut self, charset: &[u16], k_out: &mut isize, mark_key: u16) {
        if self.index >= 2
            && self.chr((self.index - 1) as usize) == KEY_U
            && self.chr((self.index - 2) as usize) == KEY_Q
        {
            self.is_correct = false;
            return;
        }
        let mut k = (self.index as isize) - 1;
        for j in (0..charset.len()).rev() {
            let mask_qe = if self.config.quick_end_consonant != 0 {
                END_CONSONANT_MASK
            } else {
                0
            };
            if (charset[j] & !mask_qe) != self.chr(k as usize) {
                self.is_correct = false;
                *k_out = k;
                return;
            }
            k -= 1;
            if k < 0 {
                break;
            }
        }

        if self.is_correct
            && charset.len() > 1
            && (self.is_key_f(mark_key) || self.is_key_x(mark_key) || self.is_key_r(mark_key))
        {
            if charset[1] == KEY_C || charset[1] == KEY_T {
                self.is_correct = false;
            } else if charset.len() > 2 && charset[2] == KEY_T {
                self.is_correct = false;
            }
        }

        if self.is_correct && k >= 0 {
            if self.chr(k as usize) == self.chr((k + 1) as usize) {
                self.is_correct = false;
            }
        }
        *k_out = k;
    }

    fn find_and_calculate_vowel(&mut self, for_grammar: bool) {
        self.vowel_count = 0;
        self.vowel_start_index = 0;
        self.vowel_end_index = 0;
        for iii in (0..(self.index as usize)).rev() {
            if is_consonant(self.chr(iii)) {
                if self.vowel_count > 0 {
                    break;
                }
            } else {
                if self.vowel_count == 0 {
                    self.vowel_end_index = iii as u8;
                }
                if !for_grammar {
                    if (iii >= 1 && self.chr(iii) == KEY_I && self.chr(iii - 1) == KEY_G)
                        || (iii >= 1 && self.chr(iii) == KEY_U && self.chr(iii - 1) == KEY_Q)
                    {
                        break;
                    }
                }
                self.vowel_start_index = iii as u8;
                self.vowel_count += 1;
            }
        }
        if self.vowel_start_index >= 1
            && self.chr(self.vowel_start_index as usize) == KEY_U
            && self.chr((self.vowel_start_index - 1) as usize) == KEY_Q
        {
            self.vowel_start_index += 1;
            self.vowel_count = self.vowel_count.wrapping_sub(1);
        }
    }

    fn remove_mark(&mut self) {
        self.find_and_calculate_vowel(true);
        self.is_changed = false;
        let vsi = self.vowel_start_index as usize;
        let vei = self.vowel_end_index as usize;
        if self.index > 0 {
            for i in vsi..=vei {
                if (self.typing_word[i] & MARK_MASK) != 0 {
                    self.typing_word[i] &= !MARK_MASK;
                    self.is_changed = true;
                }
            }
        }
        if self.is_changed {
            self.hook_state.code = HookCodeState::WillProcess as u8;
            self.hook_state.backspace_count = 0;

            let idx_usize = self.index as usize;
            for i in (vsi..idx_usize).rev() {
                self.hook_state.backspace_count += 1;
                self.hook_state.char_data[idx_usize - 1 - i] =
                    self.get_char_code(self.typing_word[i]);
            }
            self.hook_state.new_char_count = self.hook_state.backspace_count;
        } else {
            self.hook_state.code = HookCodeState::DoNothing as u8;
        }
    }

    fn can_has_end_consonant(&self) -> bool {
        let vsi = self.vowel_start_index as usize;
        let vei = self.vowel_end_index as usize;
        let vo = vowel_combine_table(self.chr(vsi));
        for entry in vo {
            let mut kk = vsi;
            let mut matched_all = true;
            for &expected in &entry[1..] {
                if kk > vei
                    || ((self.chr(kk) as u32)
                        | (self.typing_word[kk] & TONE_MASK)
                        | (self.typing_word[kk] & TONEW_MASK))
                        != expected
                {
                    matched_all = false;
                    break;
                }
                kk += 1;
            }
            if matched_all {
                return entry[0] == 1;
            }
        }
        false
    }

    fn handle_modern_mark(&mut self) {
        let vsi = self.vowel_start_index as usize;
        let vei = self.vowel_end_index as usize;
        let idx = self.index as usize;

        self.vowel_will_set_mark = self.vowel_end_index;
        self.hook_state.backspace_count = self.index - self.vowel_end_index;

        // rule 2
        if self.vowel_count == 3
            && ((self.chr(vsi) == KEY_O && self.chr(vsi + 1) == KEY_A && self.chr(vsi + 2) == KEY_I)
                || (self.chr(vsi) == KEY_U
                    && self.chr(vsi + 1) == KEY_Y
                    && self.chr(vsi + 2) == KEY_U)
                || (self.chr(vsi) == KEY_O
                    && self.chr(vsi + 1) == KEY_E
                    && self.chr(vsi + 2) == KEY_O)
                || (self.chr(vsi) == KEY_U
                    && self.chr(vsi + 1) == KEY_Y
                    && self.chr(vsi + 2) == KEY_A))
        {
            self.vowel_will_set_mark = self.vowel_start_index + 1;
            self.hook_state.backspace_count = self.index - self.vowel_will_set_mark;
        } else if (self.chr(vsi) == KEY_O && self.chr(vsi + 1) == KEY_I)
            || (self.chr(vsi) == KEY_A && self.chr(vsi + 1) == KEY_I)
            || (self.chr(vsi) == KEY_U && self.chr(vsi + 1) == KEY_I)
        {
            self.vowel_will_set_mark = self.vowel_start_index;
            self.hook_state.backspace_count = self.index - self.vowel_will_set_mark;
        } else if vei >= 1 && self.chr(vei - 1) == KEY_A && self.chr(vei) == KEY_Y {
            self.vowel_will_set_mark = self.vowel_end_index - 1;
            self.hook_state.backspace_count = (self.index - self.vowel_end_index) + 1;
        } else if self.chr(vsi) == KEY_U && self.chr(vsi + 1) == KEY_O {
            self.vowel_will_set_mark = self.vowel_start_index + 1;
            self.hook_state.backspace_count = self.index - self.vowel_will_set_mark;
        } else if self.chr(vsi + 1) == KEY_O || self.chr(vsi + 1) == KEY_U {
            self.vowel_will_set_mark = self.vowel_end_index - 1;
            self.hook_state.backspace_count = (self.index - self.vowel_end_index) + 1;
        } else if self.chr(vsi) == KEY_O || self.chr(vsi) == KEY_U {
            self.vowel_will_set_mark = self.vowel_end_index;
            self.hook_state.backspace_count = self.index - self.vowel_end_index;
        }

        // rule 3.1
        if (self.chr(vsi) == KEY_I
            && (self.typing_word[vsi + 1] & ((KEY_E as u32) | TONE_MASK)) != 0)
            || (self.chr(vsi) == KEY_Y
                && (self.typing_word[vsi + 1] & ((KEY_E as u32) | TONE_MASK)) != 0)
            || (self.chr(vsi) == KEY_U && self.typing_word[vsi + 1] == ((KEY_O as u32) | TONE_MASK))
            || (self.typing_word[vsi] == ((KEY_U as u32) | TONEW_MASK)
                && self.typing_word[vsi + 1] == ((KEY_O as u32) | TONEW_MASK))
        {
            if vsi + 2 < idx {
                let c2 = self.chr(vsi + 2);
                if c2 == KEY_P
                    || c2 == KEY_T
                    || c2 == KEY_M
                    || c2 == KEY_N
                    || c2 == KEY_O
                    || c2 == KEY_U
                    || c2 == KEY_I
                    || c2 == KEY_C
                    || (vsi + 3 < idx && c2 == KEY_C && c2 == KEY_H)
                    || (vsi + 3 < idx && c2 == KEY_N && c2 == KEY_H)
                    || (vsi + 3 < idx && c2 == KEY_N && c2 == KEY_G)
                {
                    self.vowel_will_set_mark = self.vowel_start_index + 1;
                    self.hook_state.backspace_count = self.index - self.vowel_will_set_mark;
                } else {
                    self.vowel_will_set_mark = self.vowel_start_index;
                    self.hook_state.backspace_count = self.index - self.vowel_will_set_mark;
                }
            } else {
                self.vowel_will_set_mark = self.vowel_start_index;
                self.hook_state.backspace_count = self.index - self.vowel_will_set_mark;
            }
        } else if (self.chr(vsi) == KEY_I && self.chr(vsi) == KEY_A)
            || (self.chr(vsi) == KEY_Y && self.chr(vsi) == KEY_A)
            || (self.chr(vsi) == KEY_U && self.chr(vsi) == KEY_A)
            || (self.chr(vsi) == KEY_U
                && self.typing_word[vsi + 1] == ((KEY_U as u32) | TONEW_MASK))
        {
            self.vowel_will_set_mark = self.vowel_start_index;
            self.hook_state.backspace_count = self.index - self.vowel_will_set_mark;
        }

        // rule 4
        if self.vowel_count == 2 {
            if (self.chr(vsi) == KEY_I && self.chr(vsi + 1) == KEY_A)
                || (self.chr(vsi) == KEY_I && self.chr(vsi + 1) == KEY_U)
                || (self.chr(vsi) == KEY_I && self.chr(vsi + 1) == KEY_O)
            {
                if vsi == 0 || self.chr(vsi - 1) != KEY_G {
                    self.vowel_will_set_mark = self.vowel_start_index;
                    self.hook_state.backspace_count = self.index - self.vowel_will_set_mark;
                } else {
                    self.vowel_will_set_mark = self.vowel_start_index + 1;
                    self.hook_state.backspace_count = self.index - self.vowel_will_set_mark;
                }
            } else if self.chr(vsi) == KEY_U && self.chr(vsi + 1) == KEY_A {
                if vsi == 0 || self.chr(vsi - 1) != KEY_Q {
                    if vei + 1 >= idx || !self.can_has_end_consonant() {
                        self.vowel_will_set_mark = self.vowel_start_index;
                        self.hook_state.backspace_count = self.index - self.vowel_will_set_mark;
                    }
                } else {
                    self.vowel_will_set_mark = self.vowel_start_index + 1;
                    self.hook_state.backspace_count = self.index - self.vowel_will_set_mark;
                }
            } else if self.chr(vsi) == KEY_O && self.chr(vsi + 1) == KEY_O {
                self.vowel_will_set_mark = self.vowel_end_index;
                self.hook_state.backspace_count = self.index - self.vowel_will_set_mark;
            }
        }
    }

    fn handle_old_mark(&mut self) {
        let vsi = self.vowel_start_index as usize;
        let vei = self.vowel_end_index as usize;
        let idx = self.index as usize;

        if self.vowel_count == 0 && self.chr(vei) == KEY_I {
            self.vowel_will_set_mark = self.vowel_end_index;
        } else {
            self.vowel_will_set_mark = self.vowel_start_index;
        }
        self.hook_state.backspace_count = self.index - self.vowel_will_set_mark;

        // rule 2
        if self.vowel_count == 3
            || (vei + 1 < idx && is_consonant(self.chr(vei + 1)) && self.can_has_end_consonant())
        {
            self.vowel_will_set_mark = self.vowel_start_index + 1;
            self.hook_state.backspace_count = self.index - self.vowel_will_set_mark;
        }

        // rule 3
        for ii in vsi..=vei {
            if (self.chr(ii) == KEY_E && (self.typing_word[ii] & TONE_MASK) != 0)
                || (self.chr(ii) == KEY_O && (self.typing_word[ii] & TONEW_MASK) != 0)
            {
                self.vowel_will_set_mark = ii as u8;
                self.hook_state.backspace_count = self.index - self.vowel_will_set_mark;
                break;
            }
        }

        self.hook_state.new_char_count = self.hook_state.backspace_count;
    }

    fn insert_mark(&mut self, mark_mask: u32, can_modify_flag: bool) {
        self.vowel_count = 0;

        if can_modify_flag {
            self.hook_state.code = HookCodeState::WillProcess as u8;
        }
        self.hook_state.backspace_count = 0;
        self.hook_state.new_char_count = 0;

        self.find_and_calculate_vowel(false);
        self.vowel_will_set_mark = 0;

        if self.vowel_count == 1 {
            self.vowel_will_set_mark = self.vowel_end_index;
            self.hook_state.backspace_count = self.index - self.vowel_end_index;
        } else {
            if self.config.use_modern_orthography == 0 {
                self.handle_old_mark();
            } else {
                self.handle_modern_mark();
            }
            let vei = self.vowel_end_index as usize;
            if (self.typing_word[vei] & TONE_MASK) != 0 || (self.typing_word[vei] & TONEW_MASK) != 0
            {
                self.vowel_will_set_mark = self.vowel_end_index;
            }
        }

        let vsi = self.vowel_start_index as usize;
        let idx = self.index as usize;
        let vwsm = self.vowel_will_set_mark as usize;
        let mut kk = (idx as isize) - 1 - (vsi as isize);

        if (self.typing_word[vwsm] & mark_mask) != 0 {
            self.typing_word[vwsm] &= !MARK_MASK;
            if can_modify_flag {
                self.hook_state.code = HookCodeState::Restore as u8;
            }
            for ii in vsi..idx {
                self.typing_word[ii] &= !MARK_MASK;
                if kk >= 0 {
                    self.hook_state.char_data[kk as usize] =
                        self.get_char_code(self.typing_word[ii]);
                    kk -= 1;
                }
            }
            self.temp_disable_key = true;
        } else {
            self.typing_word[vwsm] &= !MARK_MASK;
            self.typing_word[vwsm] |= mark_mask;
            for ii in vsi..idx {
                if ii != vwsm {
                    self.typing_word[ii] &= !MARK_MASK;
                }
                if kk >= 0 {
                    self.hook_state.char_data[kk as usize] =
                        self.get_char_code(self.typing_word[ii]);
                    kk -= 1;
                }
            }
            self.hook_state.backspace_count = self.index - self.vowel_start_index;
        }
        self.hook_state.new_char_count = self.hook_state.backspace_count;
    }

    fn insert_d(&mut self, _data: u16, _is_caps: bool) {
        self.hook_state.code = HookCodeState::WillProcess as u8;
        self.hook_state.backspace_count = 0;
        let idx = self.index as usize;
        for ii in (0..idx).rev() {
            self.hook_state.backspace_count += 1;
            if self.chr(ii) == KEY_D {
                if (self.typing_word[ii] & TONE_MASK) != 0 {
                    self.hook_state.code = HookCodeState::Restore as u8;
                    self.typing_word[ii] &= !TONE_MASK;
                    self.hook_state.char_data[idx - 1 - ii] = self.typing_word[ii];
                    self.temp_disable_key = true;
                    break;
                } else {
                    self.typing_word[ii] |= TONE_MASK;
                    self.hook_state.char_data[idx - 1 - ii] =
                        self.get_char_code(self.typing_word[ii]);
                }
                break;
            } else {
                self.hook_state.char_data[idx - 1 - ii] = self.get_char_code(self.typing_word[ii]);
            }
        }
        self.hook_state.new_char_count = self.hook_state.backspace_count;
    }

    fn insert_aoe(&mut self, data: u16, _is_caps: bool) {
        self.find_and_calculate_vowel(false);

        let vsi = self.vowel_start_index as usize;
        let vei = self.vowel_end_index as usize;
        for ii in vsi..=vei {
            self.typing_word[ii] &= !TONEW_MASK;
        }

        self.hook_state.code = HookCodeState::WillProcess as u8;
        self.hook_state.backspace_count = 0;

        let idx = self.index as usize;
        for ii in (0..idx).rev() {
            self.hook_state.backspace_count += 1;
            if self.chr(ii) == data {
                if (self.typing_word[ii] & TONE_MASK) != 0 {
                    self.hook_state.code = HookCodeState::Restore as u8;
                    self.typing_word[ii] &= !TONE_MASK;
                    self.hook_state.char_data[idx - 1 - ii] = self.typing_word[ii];
                    if data != KEY_O {
                        self.temp_disable_key = true;
                    }
                    break;
                } else {
                    self.typing_word[ii] |= TONE_MASK;
                    if !self.is_key_d(data) {
                        self.typing_word[ii] &= !TONEW_MASK;
                    }
                    self.hook_state.char_data[idx - 1 - ii] =
                        self.get_char_code(self.typing_word[ii]);
                }
                break;
            } else {
                self.hook_state.char_data[idx - 1 - ii] = self.get_char_code(self.typing_word[ii]);
            }
        }
        self.hook_state.new_char_count = self.hook_state.backspace_count;
    }

    fn insert_w(&mut self, _data: u16, _is_caps: bool) {
        self.is_restored_w = false;

        self.find_and_calculate_vowel(false);

        let vsi = self.vowel_start_index as usize;
        let vei = self.vowel_end_index as usize;
        let idx = self.index as usize;

        for ii in vsi..=vei {
            self.typing_word[ii] &= !TONE_MASK;
        }

        if self.vowel_count > 1 {
            self.hook_state.backspace_count = self.index - self.vowel_start_index;
            self.hook_state.new_char_count = self.hook_state.backspace_count;

            if ((self.typing_word[vsi] & TONEW_MASK) != 0
                && (self.typing_word[vsi + 1] & TONEW_MASK) != 0)
                || ((self.typing_word[vsi] & TONEW_MASK) != 0 && self.chr(vsi + 1) == KEY_I)
                || ((self.typing_word[vsi] & TONEW_MASK) != 0 && self.chr(vsi + 1) == KEY_A)
            {
                self.hook_state.code = HookCodeState::Restore as u8;

                for ii in vsi..idx {
                    self.typing_word[ii] &= !TONEW_MASK;
                    self.hook_state.char_data[idx - 1 - ii] =
                        self.get_char_code(self.typing_word[ii]) & !STANDALONE_MASK;
                }
                self.is_restored_w = true;
                self.temp_disable_key = true;
            } else {
                self.hook_state.code = HookCodeState::WillProcess as u8;

                if self.chr(vsi) == KEY_U && self.chr(vsi + 1) == KEY_O {
                    if vsi >= 2
                        && self.typing_word[vsi - 2] == (KEY_T as u32)
                        && self.typing_word[vsi - 1] == (KEY_H as u32)
                    {
                        self.typing_word[vsi + 1] |= TONEW_MASK;
                        if vsi + 2 < idx && self.chr(vsi + 2) == KEY_N {
                            self.typing_word[vsi] |= TONEW_MASK;
                        }
                    } else if vsi >= 1 && self.typing_word[vsi - 1] == (KEY_Q as u32) {
                        self.typing_word[vsi + 1] |= TONEW_MASK;
                    } else {
                        self.typing_word[vsi] |= TONEW_MASK;
                        self.typing_word[vsi + 1] |= TONEW_MASK;
                    }
                } else if (self.chr(vsi) == KEY_U && self.chr(vsi + 1) == KEY_A)
                    || (self.chr(vsi) == KEY_U && self.chr(vsi + 1) == KEY_I)
                    || (self.chr(vsi) == KEY_U && self.chr(vsi + 1) == KEY_U)
                    || (self.chr(vsi) == KEY_O && self.chr(vsi + 1) == KEY_I)
                {
                    self.typing_word[vsi] |= TONEW_MASK;
                } else if (self.chr(vsi) == KEY_I && self.chr(vsi + 1) == KEY_O)
                    || (self.chr(vsi) == KEY_O && self.chr(vsi + 1) == KEY_A)
                {
                    self.typing_word[vsi + 1] |= TONEW_MASK;
                } else {
                    self.temp_disable_key = true;
                    self.is_changed = false;
                    self.hook_state.code = HookCodeState::DoNothing as u8;
                }

                for ii in vsi..idx {
                    self.hook_state.char_data[idx - 1 - ii] =
                        self.get_char_code(self.typing_word[ii]);
                }
            }
            return;
        }

        self.hook_state.code = HookCodeState::WillProcess as u8;
        self.hook_state.backspace_count = 0;

        for ii in (0..idx).rev() {
            if ii < vsi {
                break;
            }
            self.hook_state.backspace_count += 1;
            match self.chr(ii) {
                KEY_A | KEY_U | KEY_O => {
                    if (self.typing_word[ii] & TONEW_MASK) != 0 {
                        if (self.typing_word[ii] & STANDALONE_MASK) != 0 {
                            self.hook_state.code = HookCodeState::WillProcess as u8;
                            if self.chr(ii) == KEY_U {
                                self.typing_word[ii] = (KEY_W as u32)
                                    | if (self.typing_word[ii] & CAPS_MASK) != 0 {
                                        CAPS_MASK
                                    } else {
                                        0
                                    };
                            } else if self.chr(ii) == KEY_O {
                                self.hook_state.code = HookCodeState::Restore as u8;
                                self.typing_word[ii] = (KEY_O as u32)
                                    | if (self.typing_word[ii] & CAPS_MASK) != 0 {
                                        CAPS_MASK
                                    } else {
                                        0
                                    };
                                self.is_restored_w = true;
                            }
                            self.hook_state.char_data[idx - 1 - ii] = self.typing_word[ii];
                        } else {
                            self.hook_state.code = HookCodeState::Restore as u8;
                            self.typing_word[ii] &= !TONEW_MASK;
                            self.hook_state.char_data[idx - 1 - ii] = self.typing_word[ii];
                            self.is_restored_w = true;
                        }
                        self.temp_disable_key = true;
                    } else {
                        self.typing_word[ii] |= TONEW_MASK;
                        self.typing_word[ii] &= !TONE_MASK;
                        self.hook_state.char_data[idx - 1 - ii] =
                            self.get_char_code(self.typing_word[ii]);
                    }
                }
                _ => {
                    self.hook_state.char_data[idx - 1 - ii] =
                        self.get_char_code(self.typing_word[ii]);
                }
            }
        }
        self.hook_state.new_char_count = self.hook_state.backspace_count;
    }

    fn reverse_last_standalone_char(&mut self, key_code: u32, is_caps: bool) {
        self.hook_state.code = HookCodeState::WillProcess as u8;
        self.hook_state.backspace_count = 0;
        self.hook_state.new_char_count = 1;
        self.hook_state.ext_code = 4;
        let last = (self.index - 1) as usize;
        self.typing_word[last] =
            key_code | TONEW_MASK | STANDALONE_MASK | if is_caps { CAPS_MASK } else { 0 };
        self.hook_state.char_data[0] = self.get_char_code(self.typing_word[last]);
    }

    fn check_for_standalone_char(&mut self, data: u16, is_caps: bool, key_will_reverse: u32) {
        if self.index > 0
            && (self.chr((self.index - 1) as usize) as u32) == key_will_reverse
            && (self.typing_word[(self.index - 1) as usize] & TONEW_MASK) != 0
        {
            self.hook_state.code = HookCodeState::WillProcess as u8;
            self.hook_state.backspace_count = 1;
            self.hook_state.new_char_count = 1;
            let last = (self.index - 1) as usize;
            self.typing_word[last] = (data as u32) | if is_caps { CAPS_MASK } else { 0 };
            self.hook_state.char_data[0] = self.get_char_code(self.typing_word[last]);
            return;
        }

        if self.index > 0
            && self.chr((self.index - 1) as usize) == KEY_U
            && key_will_reverse == (KEY_O as u32)
        {
            self.insert_key(key_will_reverse as u16, is_caps, true);
            self.reverse_last_standalone_char(key_will_reverse, is_caps);
            return;
        }

        if self.index == 0 {
            self.insert_key(data, is_caps, false);
            self.reverse_last_standalone_char(key_will_reverse, is_caps);
            return;
        } else if self.index == 1 {
            if STANDALONE_W_BAD.contains(&self.chr(0)) {
                self.insert_key(data, is_caps, true);
                return;
            }
            self.insert_key(data, is_caps, false);
            self.reverse_last_standalone_char(key_will_reverse, is_caps);
            return;
        } else if self.index == 2 {
            for allowed in DOUBLE_W_ALLOWED {
                if self.chr(0) == allowed[0] && self.chr(1) == allowed[1] {
                    self.insert_key(data, is_caps, false);
                    self.reverse_last_standalone_char(key_will_reverse, is_caps);
                    return;
                }
            }
            self.insert_key(data, is_caps, true);
            return;
        }

        self.insert_key(data, is_caps, true);
    }

    fn upper_case_first_character(&mut self) {
        if (self.typing_word[0] & CAPS_MASK) == 0 {
            self.hook_state.code = HookCodeState::WillProcess as u8;
            self.hook_state.backspace_count = 0;
            self.hook_state.new_char_count = 1;
            self.typing_word[0] |= CAPS_MASK;
            self.hook_state.char_data[0] = self.get_char_code(self.typing_word[0]);
            self.upper_case_status = 0;
            if self.config.use_macro != 0 && !self.hook_state.macro_key.is_empty() {
                self.hook_state.macro_key[0] |= CAPS_MASK;
            }
        }
    }

    fn handle_main_key(&mut self, data: u16, is_caps: bool) {
        if self.is_key_z(data) {
            self.remove_mark();
            if !self.is_changed {
                self.insert_key(data, is_caps, true);
            }
            return;
        }

        if data == KEY_LEFT_BRACKET {
            self.check_for_standalone_char(data, is_caps, KEY_O as u32);
            return;
        }

        if data == KEY_RIGHT_BRACKET {
            self.check_for_standalone_char(data, is_caps, KEY_U as u32);
            return;
        }

        if self.is_key_d(data) {
            self.is_correct = false;
            self.is_changed = false;
            let mut k = self.index as isize;
            for cons_d in CONSONANT_D {
                if (self.index as usize) < cons_d.len() {
                    continue;
                }
                self.is_correct = true;
                self.check_correct_vowel(cons_d, &mut k, data);

                if !self.is_correct
                    && self.index >= 2
                    && self.chr((self.index - 1) as usize) == KEY_D
                    && is_consonant(self.chr((self.index - 2) as usize))
                {
                    self.is_correct = true;
                }
                if self.is_correct {
                    self.is_changed = true;
                    self.insert_d(data, is_caps);
                    break;
                }
            }

            if !self.is_changed {
                self.insert_key(data, is_caps, true);
            }
            return;
        }

        if self.is_mark_key(data) {
            for &vkey in &VOWEL_FOR_MARK_KEYS {
                let charset = vowel_for_mark_table(vkey);
                self.is_correct = false;
                self.is_changed = false;
                let mut k = self.index as isize;
                for entry in charset {
                    if (self.index as usize) < entry.len() {
                        continue;
                    }
                    self.is_correct = true;
                    self.check_correct_vowel(entry, &mut k, data);

                    if self.is_correct {
                        self.is_changed = true;
                        if self.is_key_s(data) {
                            self.insert_mark(MARK1_MASK, true);
                        } else if self.is_key_f(data) {
                            self.insert_mark(MARK2_MASK, true);
                        } else if self.is_key_r(data) {
                            self.insert_mark(MARK3_MASK, true);
                        } else if self.is_key_x(data) {
                            self.insert_mark(MARK4_MASK, true);
                        } else if self.is_key_j(data) {
                            self.insert_mark(MARK5_MASK, true);
                        }
                        break;
                    }
                }

                if self.is_correct {
                    break;
                }
            }

            if !self.is_changed {
                self.insert_key(data, is_caps, true);
            }
            return;
        }

        if self.config.input_type == (InputType::Vni as i32) {
            for i in (0..(self.index as usize)).rev() {
                let c = self.chr(i);
                if c == KEY_O || c == KEY_A || c == KEY_E {
                    self.vowel_end_index = i as u8;
                    break;
                }
            }
        }

        self.key_for_aeo = if self.config.input_type != (InputType::Vni as i32) {
            data
        } else if data == KEY_7 || data == KEY_8 {
            KEY_W
        } else if data == KEY_6 {
            self.typing_word[self.vowel_end_index as usize] as u16
        } else {
            data
        };

        let charset = vowel_table(self.key_for_aeo);
        self.is_correct = false;
        self.is_changed = false;
        let mut k = self.index as isize;
        for entry in charset {
            if (self.index as usize) < entry.len() {
                continue;
            }
            self.is_correct = true;
            self.check_correct_vowel(entry, &mut k, data);

            if self.is_correct {
                self.is_changed = true;
                if self.is_key_double(data) {
                    let k_aeo = self.key_for_aeo;
                    self.insert_aoe(k_aeo, is_caps);
                } else if self.is_key_w(data) {
                    if self.config.input_type == (InputType::Vni as i32) {
                        for j in (0..(self.index as usize)).rev() {
                            let c = self.chr(j);
                            if c == KEY_O || c == KEY_U || c == KEY_A || c == KEY_E {
                                self.vowel_end_index = j as u8;
                                break;
                            }
                        }
                        let vei = self.vowel_end_index as usize;
                        if (data == KEY_7
                            && self.chr(vei) == KEY_A
                            && (if vei >= 1 {
                                self.chr(vei - 1) != KEY_U
                            } else {
                                true
                            }))
                            || (data == KEY_8 && (self.chr(vei) == KEY_O || self.chr(vei) == KEY_U))
                        {
                            break;
                        }
                    }
                    let k_aeo = self.key_for_aeo;
                    self.insert_w(k_aeo, is_caps);
                }
                break;
            }
        }

        if !self.is_changed {
            if data == KEY_W && self.config.input_type != (InputType::SimpleTelex1 as i32) {
                self.check_for_standalone_char(data, is_caps, KEY_U as u32);
            } else {
                self.insert_key(data, is_caps, true);
            }
        }
    }

    fn handle_quick_telex(&mut self, data: u16, is_caps: bool) {
        let Some(pair) = quick_telex(data as u32) else {
            return;
        };
        let caps_flag = if is_caps { CAPS_MASK } else { 0 };
        self.hook_state.code = HookCodeState::WillProcess as u8;
        self.hook_state.backspace_count = 1;
        self.hook_state.new_char_count = 2;
        self.hook_state.char_data[1] = (pair[0] as u32) | caps_flag;
        self.hook_state.char_data[0] = (pair[1] as u32) | caps_flag;
        self.insert_key(pair[1], is_caps, false);
    }

    fn check_restore_if_wrong_spelling(&mut self, handle_code: u8) -> bool {
        let idx = self.index as usize;
        for ii in 0..idx {
            if !is_consonant(self.chr(ii))
                && ((self.typing_word[ii] & MARK_MASK) != 0
                    || (self.typing_word[ii] & TONE_MASK) != 0
                    || (self.typing_word[ii] & TONEW_MASK) != 0)
            {
                self.hook_state.code = handle_code;
                self.hook_state.backspace_count = self.index;
                self.hook_state.new_char_count = self.state_index;
                let s_idx = self.state_index as usize;
                for i in 0..s_idx {
                    self.typing_word[i] = self.key_states[i];
                    self.hook_state.char_data[s_idx - 1 - i] = self.typing_word[i];
                }
                self.index = self.state_index;
                return true;
            }
        }
        false
    }

    fn check_quick_consonant(&mut self) -> bool {
        if self.index <= 1 {
            return false;
        }
        let mut l = 0;
        if self.index > 0 {
            if self.config.quick_start_consonant != 0 {
                if let Some(pair) = quick_start_consonant(self.chr(0)) {
                    self.hook_state.code = HookCodeState::Restore as u8;
                    self.hook_state.backspace_count = self.index;
                    self.hook_state.new_char_count = self.index + 1;
                    if (self.index as usize) < MAX_BUFF - 1 {
                        self.index += 1;
                    }
                    for i in (2..(self.index as usize)).rev() {
                        self.typing_word[i] = self.typing_word[i - 1];
                    }
                    let caps1 = if (self.typing_word[0] & CAPS_MASK) != 0
                        && (self.typing_word[2] & CAPS_MASK) != 0
                    {
                        CAPS_MASK
                    } else {
                        0
                    };
                    let caps0 = if (self.typing_word[0] & CAPS_MASK) != 0 {
                        CAPS_MASK
                    } else {
                        0
                    };
                    self.typing_word[1] = (pair[1] as u32) | caps1;
                    self.typing_word[0] = (pair[0] as u32) | caps0;
                    l = 1;
                }
            }
            if self.config.quick_end_consonant != 0
                && self.index >= 2
                && !is_consonant(self.chr((self.index - 2) as usize))
            {
                if let Some(pair) = quick_end_consonant(self.chr((self.index - 1) as usize)) {
                    self.hook_state.code = HookCodeState::Restore as u8;
                    if l == 1 {
                        self.hook_state.new_char_count += 1;
                    } else {
                        self.hook_state.backspace_count = 1;
                        self.hook_state.new_char_count = 2;
                    }
                    if (self.index as usize) < MAX_BUFF - 1 {
                        self.index += 1;
                    }
                    let idx_usize = self.index as usize;
                    // Note: in C++ TypingWord[_index-2] isCHR(_index-2) which after _index++ is the old _index-1!
                    let Some(pair_after) = quick_end_consonant(self.chr(idx_usize - 2)) else {
                        return false;
                    };
                    let caps_end = if (self.typing_word[idx_usize - 2] & CAPS_MASK) != 0 {
                        CAPS_MASK
                    } else {
                        0
                    };
                    let _ = pair;
                    self.typing_word[idx_usize - 1] = (pair_after[1] as u32) | caps_end;
                    self.typing_word[idx_usize - 2] = (pair_after[0] as u32) | caps_end;
                    l = 1;
                }
            }
            if l == 1 {
                self.has_handled_quick_consonant = true;
                let idx_usize = self.index as usize;
                for i in (0..idx_usize).rev() {
                    self.hook_state.char_data[idx_usize - 1 - i] =
                        self.get_char_code(self.typing_word[i]);
                }
                return true;
            }
        }
        false
    }

    pub fn english_mode(
        &mut self,
        state: KeyEventState,
        data: u16,
        is_caps: bool,
        other_control_key: bool,
    ) {
        self.hook_state.code = HookCodeState::DoNothing as u8;
        if state == KeyEventState::MouseDown || (other_control_key && !is_caps) {
            self.hook_state.macro_key.clear();
            self.will_temp_off_engine = false;
        } else if data == KEY_SPACE {
            if !self.has_handled_macro {
                if let Some(macro_data) = self.macros.find_macro(
                    &mut self.hook_state.macro_key,
                    self.config.code_table as usize,
                    self.config.auto_caps_macro != 0,
                ) {
                    self.hook_state.macro_data = macro_data;
                    self.hook_state.code = HookCodeState::ReplaceMacro as u8;
                    self.hook_state.backspace_count = self.hook_state.macro_key.len() as u8;
                }
            }
            self.hook_state.macro_key.clear();
            self.will_temp_off_engine = false;
        } else if data == KEY_DELETE {
            if !self.hook_state.macro_key.is_empty() {
                self.hook_state.macro_key.pop();
            } else {
                self.will_temp_off_engine = false;
            }
        } else if Self::is_word_break(KeyEvent::Keyboard, state, data)
            && !CHAR_KEY_CODES.contains(&data)
        {
            self.hook_state.macro_key.clear();
            self.will_temp_off_engine = false;
        } else if !self.will_temp_off_engine {
            self.hook_state
                .macro_key
                .push((data as u32) | if is_caps { CAPS_MASK } else { 0 });
        }
    }

    pub fn handle_event(
        &mut self,
        event: KeyEvent,
        state: KeyEventState,
        data: u16,
        caps_status: u8,
        other_control_key: bool,
    ) {
        self.is_caps = caps_status == 1 || caps_status == 2;
        if (is_number_key(data) && caps_status == 1)
            || other_control_key
            || Self::is_word_break(event, state, data)
            || (self.index == 0 && is_number_key(data))
        {
            self.hook_state.code = HookCodeState::DoNothing as u8;
            self.hook_state.backspace_count = 0;
            self.hook_state.new_char_count = 0;
            self.hook_state.ext_code = 1; // word break

            let mut matched_macro = false;
            if self.config.use_macro != 0
                && Self::is_macro_break_code(data)
                && !self.has_handled_macro
            {
                if let Some(macro_data) = self.macros.find_macro(
                    &mut self.hook_state.macro_key,
                    self.config.code_table as usize,
                    self.config.auto_caps_macro != 0,
                ) {
                    self.hook_state.macro_data = macro_data;
                    self.hook_state.code = HookCodeState::ReplaceMacro as u8;
                    self.hook_state.backspace_count = self.hook_state.macro_key.len() as u8;
                    self.has_handled_macro = true;
                    matched_macro = true;
                }
            }

            if !matched_macro {
                if (self.config.quick_start_consonant != 0 || self.config.quick_end_consonant != 0)
                    && !self.temp_disable_key
                    && Self::is_macro_break_code(data)
                {
                    self.check_quick_consonant();
                } else if self.config.restore_if_wrong_spelling != 0
                    && Self::is_word_break(event, state, data)
                {
                    if !self.temp_disable_key && self.config.check_spelling != 0 {
                        self.check_spelling(true);
                    }
                    if self.temp_disable_key
                        && !self.check_restore_if_wrong_spelling(
                            HookCodeState::RestoreAndStartNewSession as u8,
                        )
                    {
                        self.hook_state.code = HookCodeState::DoNothing as u8;
                    }
                }
            }

            self.is_char_key_code = state == KeyEventState::KeyDown && CHAR_KEY_CODES.contains(&data);
            if !self.is_char_key_code {
                self.special_char.clear();
                self.typing_states.clear();
            } else {
                if self.space_count > 0 {
                    let sc = self.space_count;
                    self.save_word_repeated(KEY_SPACE as u32, sc);
                    self.space_count = 0;
                } else {
                    self.save_word();
                }
                self.special_char
                    .push((data as u32) | if self.is_caps { CAPS_MASK } else { 0 });
                self.hook_state.ext_code = 3;
            }

            if self.hook_state.code == (HookCodeState::DoNothing as u8) {
                self.start_new_session();
                self.config.check_spelling = if self.use_spell_checking_before { 1 } else { 0 };
                self.will_temp_off_engine = false;
            } else if self.hook_state.code == (HookCodeState::ReplaceMacro as u8)
                || self.has_handled_quick_consonant
            {
                self.index = 0;
            }

            if self.config.use_macro != 0 {
                if self.is_char_key_code {
                    self.hook_state
                        .macro_key
                        .push((data as u32) | if self.is_caps { CAPS_MASK } else { 0 });
                } else {
                    self.hook_state.macro_key.clear();
                }
            }

            if self.config.upper_case_first_char != 0 {
                if data == KEY_DOT {
                    self.upper_case_status = 1;
                } else if data == KEY_ENTER || data == KEY_RETURN {
                    self.upper_case_status = 2;
                } else {
                    self.upper_case_status = 0;
                }
            }
        } else if data == KEY_SPACE {
            if !self.temp_disable_key && self.config.check_spelling != 0 {
                self.check_spelling(true);
            }
            let mut matched_macro = false;
            if self.config.use_macro != 0 && !self.has_handled_macro {
                if let Some(macro_data) = self.macros.find_macro(
                    &mut self.hook_state.macro_key,
                    self.config.code_table as usize,
                    self.config.auto_caps_macro != 0,
                ) {
                    self.hook_state.macro_data = macro_data;
                    self.hook_state.code = HookCodeState::ReplaceMacro as u8;
                    self.hook_state.backspace_count = self.hook_state.macro_key.len() as u8;
                    self.space_count += 1;
                    self.has_handled_macro = true;
                    matched_macro = true;
                }
            }
            if !matched_macro {
                if (self.config.quick_start_consonant != 0 || self.config.quick_end_consonant != 0)
                    && !self.temp_disable_key
                    && self.check_quick_consonant()
                {
                    self.space_count += 1;
                } else if self.config.restore_if_wrong_spelling != 0
                    && self.temp_disable_key
                    && !self.has_handled_macro
                {
                    if !self.check_restore_if_wrong_spelling(HookCodeState::Restore as u8) {
                        self.hook_state.code = HookCodeState::DoNothing as u8;
                    }
                    self.space_count += 1;
                } else {
                    self.hook_state.code = HookCodeState::DoNothing as u8;
                    self.space_count += 1;
                }
            }
            if self.config.use_macro != 0 {
                self.hook_state.macro_key.clear();
            }
            if self.config.upper_case_first_char != 0 && self.upper_case_status == 1 {
                self.upper_case_status = 2;
            }
            if self.space_count == 1 {
                if !self.special_char.is_empty() {
                    self.save_special_char();
                } else {
                    self.save_word();
                }
            }
            self.config.check_spelling = if self.use_spell_checking_before { 1 } else { 0 };
            self.will_temp_off_engine = false;
        } else if data == KEY_DELETE {
            self.hook_state.code = HookCodeState::DoNothing as u8;
            self.hook_state.ext_code = 2; // delete
            if !self.special_char.is_empty() {
                self.special_char.pop();
                if self.special_char.is_empty() {
                    self.restore_last_typing_state();
                }
            } else if self.space_count > 0 {
                self.space_count -= 1;
                if self.space_count == 0 {
                    self.restore_last_typing_state();
                }
            } else {
                if self.state_index > 0 {
                    self.state_index -= 1;
                }
                if self.index > 0 {
                    self.index -= 1;
                    if let Some(last_long) = self.long_word_helper.pop() {
                        for i in (1..MAX_BUFF).rev() {
                            self.typing_word[i] = self.typing_word[i - 1];
                        }
                        self.typing_word[0] = last_long;
                        self.index += 1;
                    }
                    if self.config.check_spelling != 0 {
                        self.check_spelling(false);
                    }
                }
                if self.config.use_macro != 0 && !self.hook_state.macro_key.is_empty() {
                    self.hook_state.macro_key.pop();
                }

                self.hook_state.backspace_count = 0;
                self.hook_state.new_char_count = 0;
                self.hook_state.ext_code = 2;
                if self.index == 0 {
                    self.start_new_session();
                    self.special_char.clear();
                    self.restore_last_typing_state();
                } else {
                    self.check_grammar(1);
                }
            }
        } else {
            if self.will_temp_off_engine {
                self.hook_state.code = HookCodeState::DoNothing as u8;
                self.hook_state.ext_code = 3;
                return;
            }
            if self.space_count > 0 {
                self.hook_state.backspace_count = 0;
                self.hook_state.new_char_count = 0;
                self.hook_state.ext_code = 0;
                self.start_new_session();
                let sc = self.space_count;
                self.save_word_repeated(KEY_SPACE as u32, sc);
                self.space_count = 0;
            } else if !self.special_char.is_empty() {
                self.save_special_char();
            }

            let is_caps = self.is_caps;
            self.insert_state(data, is_caps);

            if !is_special_key(data, self.config.input_type) || self.temp_disable_key {
                if self.config.quick_telex != 0 && self.is_quick_telex_key(data) {
                    self.handle_quick_telex(data, is_caps);
                    return;
                } else {
                    self.hook_state.code = HookCodeState::DoNothing as u8;
                    self.hook_state.backspace_count = 0;
                    self.hook_state.new_char_count = 0;
                    self.hook_state.ext_code = 3;
                    self.insert_key(data, is_caps, true);
                }
            } else {
                self.hook_state.code = HookCodeState::DoNothing as u8;
                self.hook_state.ext_code = 3;
                self.handle_main_key(data, is_caps);
            }

            if self.config.free_mark == 0 && !self.is_key_d(data) {
                if self.hook_state.code == (HookCodeState::DoNothing as u8) {
                    self.check_grammar(-1);
                } else {
                    self.check_grammar(0);
                }
            }

            if self.hook_state.code == (HookCodeState::Restore as u8) {
                self.insert_key(data, is_caps, true);
                self.state_index = self.state_index.wrapping_sub(1);
            }

            if self.config.use_macro != 0 {
                if self.hook_state.code == (HookCodeState::DoNothing as u8) {
                    self.hook_state
                        .macro_key
                        .push((data as u32) | if is_caps { CAPS_MASK } else { 0 });
                } else if self.hook_state.code == (HookCodeState::WillProcess as u8)
                    || self.hook_state.code == (HookCodeState::Restore as u8)
                {
                    for _ in 0..(self.hook_state.backspace_count as usize) {
                        self.hook_state.macro_key.pop();
                    }
                    let start = (self.index as usize)
                        .saturating_sub(self.hook_state.backspace_count as usize);
                    let end = start + (self.hook_state.new_char_count as usize);
                    for i in start..end.min(MAX_BUFF) {
                        self.hook_state.macro_key.push(self.typing_word[i]);
                    }
                }
            }

            if self.config.upper_case_first_char != 0 {
                if self.index == 1 && self.upper_case_status == 2 {
                    self.upper_case_first_character();
                }
                self.upper_case_status = 0;
            }

            if Self::is_bracket_key(data)
                && (Self::is_bracket_key(self.hook_state.char_data[0] as u16)
                    || self.config.input_type == (InputType::SimpleTelex1 as i32)
                    || self.config.input_type == (InputType::SimpleTelex2 as i32))
            {
                let sub = if self.hook_state.code == (HookCodeState::WillProcess as u8) {
                    self.hook_state.backspace_count as isize
                } else {
                    0
                };
                if (self.index as isize) - sub > 0 {
                    self.index -= 1;
                    self.save_word();
                }
                self.index = 0;
                self.temp_disable_key = false;
                self.state_index = 0;
                self.hook_state.ext_code = 3;
                self.special_char
                    .push((data as u32) | if is_caps { CAPS_MASK } else { 0 });
            }
        }
    }
}
