use openkey_engine::engine::Engine;
use openkey_engine::types::*;
use openkey_engine::vietnamese::{key_code_to_character, UNICODE_COMPOUND_MARK};

use crate::config::MacConfig;
use crate::hotkeys::*;
use crate::sender::EventSender;
use crate::workaround::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacEventType {
    TapDisabledByTimeout,
    TapDisabledByUserInput,
    KeyDown,
    KeyUp,
    FlagsChanged,
    LeftMouseDown,
    RightMouseDown,
    LeftMouseDragged,
    RightMouseDragged,
    Other(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackAction {
    PassThrough,
    Swallow,
}

#[derive(Debug, Clone)]
pub struct MacEventHandler {
    pub engine: Engine,
    pub send_key_step_by_step: bool,
    pub fix_chromium_browser: bool,
    pub perform_layout_compat: bool,
    pub sync_key: Vec<u8>,
    pub last_flag: u64,
    pub has_just_used_hotkey: bool,
    pub frontmost_app: String,
    pub spotlight_visible: bool,
    pub current_input_source_is_english: bool,
}

impl MacEventHandler {
    pub fn new(config: MacConfig) -> Self {
        let mut engine = Engine::with_config(config.engine);
        engine.convert_tool = config.convert_tool;
        if !config.macro_data.is_empty() {
            let ct = engine.config.code_table as usize;
            engine.macros.init_from_bytes(&config.macro_data, ct);
        }
        if !config.smart_switch_data.is_empty() {
            engine
                .smart_switch
                .init_from_bytes(&config.smart_switch_data);
        }
        Self {
            engine,
            send_key_step_by_step: config.send_key_step_by_step,
            fix_chromium_browser: config.fix_chromium_browser,
            perform_layout_compat: config.perform_layout_compat,
            sync_key: Vec::new(),
            last_flag: 0,
            has_just_used_hotkey: false,
            frontmost_app: "UnknownApp".to_string(),
            spotlight_visible: false,
            current_input_source_is_english: true,
        }
    }

    pub fn set_frontmost_app(&mut self, bundle_id: &str, spotlight_visible: bool) {
        self.frontmost_app = bundle_id.to_string();
        self.spotlight_visible = spotlight_visible;
    }

    pub fn set_current_input_source_is_english(&mut self, is_english: bool) {
        self.current_input_source_is_english = is_english;
    }

    pub fn request_new_session(&mut self) {
        self.engine
            .handle_event(KeyEvent::Mouse, KeyEventState::MouseDown, 0, 0, false);
        if is_double_code(self.engine.config.code_table) {
            self.sync_key.clear();
        }
    }

    pub fn switch_language<S: EventSender>(&mut self, sender: &mut S) {
        if self.engine.config.language == 0 {
            self.engine.config.language = 1;
        } else {
            self.engine.config.language = 0;
        }
        if has_beep(self.engine.config.switch_key_status) {
            sender.beep();
        }
        sender.on_input_method_changed(self.engine.config.language, true);
        if self.engine.config.use_smart_switch_key != 0 {
            let status = self.engine.config.language | (self.engine.config.code_table << 1);
            self.engine
                .smart_switch
                .set_app_input_method_status(&self.frontmost_app, status);
        }
        self.engine.start_new_session();
    }

    pub fn on_active_app_changed<S: EventSender>(&mut self, bundle_id: &str, sender: &mut S) {
        self.frontmost_app = bundle_id.to_string();
        let current_packed = self.engine.config.language | (self.engine.config.code_table << 1);
        let temp = self
            .engine
            .smart_switch
            .get_app_input_method_status(&self.frontmost_app, current_packed);
        if temp != -1 {
            let app_lang = temp & 0x01;
            let app_code = temp >> 1;
            if app_lang != self.engine.config.language {
                self.engine.config.language = app_lang;
                sender.on_input_method_changed(app_lang, false);
                self.engine.start_new_session();
            }
            if self.engine.config.remember_code != 0 && app_code != self.engine.config.code_table {
                self.engine.config.code_table = app_code;
                self.engine
                    .macros
                    .on_table_code_change(app_code as usize);
                sender.on_code_table_changed(app_code);
            }
        }
    }

    fn insert_key_length(&mut self, len: u8) {
        self.sync_key.push(len);
    }

    fn send_backspace<S: EventSender>(&mut self, sender: &mut S) {
        sender.send_backspace();
        let code_table = self.engine.config.code_table;
        if is_double_code(code_table) {
            if let Some(last) = self.sync_key.pop() {
                if last > 1 && !(code_table == 3 && contain_unicode_compound_app(&self.frontmost_app))
                {
                    sender.send_backspace();
                }
            }
        }
    }

    fn send_shift_and_left_arrow<S: EventSender>(&mut self, sender: &mut S) {
        sender.send_shift_left_arrow();
        let code_table = self.engine.config.code_table;
        if is_double_code(code_table) {
            if let Some(last) = self.sync_key.pop() {
                if last > 1 && !(code_table == 3 && contain_unicode_compound_app(&self.frontmost_app))
                {
                    sender.send_shift_left_arrow();
                }
            }
        }
    }

    fn send_empty_character<S: EventSender>(&mut self, sender: &mut S) {
        if is_double_code(self.engine.config.code_table) {
            self.insert_key_length(1);
        }
        let ch = empty_character_for_app(&self.frontmost_app);
        sender.send_empty_character(ch);
    }

    fn send_pure_character<S: EventSender>(&mut self, ch: u16, sender: &mut S) {
        sender.send_unicode_string(&[ch]);
        if is_double_code(self.engine.config.code_table) {
            self.insert_key_length(1);
        }
    }

    fn send_key_code<S: EventSender>(&mut self, data: u32, sender: &mut S) {
        let new_char = data as u16;
        let code_table = self.engine.config.code_table;
        if (data & CHAR_CODE_MASK) == 0 {
            if is_double_code(code_table) {
                self.insert_key_length(1);
            }
            sender.send_key_code(new_char, (data & CAPS_MASK) != 0);
        } else if code_table == 0 {
            sender.send_unicode_string(&[new_char]);
        } else if code_table == 1 || code_table == 2 || code_table == 4 {
            let hi = (new_char >> 8) & 0xFF;
            let lo = new_char & 0xFF;
            sender.send_unicode_string(&[lo]);
            if hi > 32 {
                if code_table == 2 {
                    self.insert_key_length(2);
                }
                sender.send_unicode_string(&[hi]);
            } else if code_table == 2 {
                self.insert_key_length(1);
            }
        } else if code_table == 3 {
            let hi = new_char >> 13;
            let lo = new_char & 0x1FFF;
            if hi > 0 {
                let mark = UNICODE_COMPOUND_MARK[(hi - 1) as usize];
                self.insert_key_length(2);
                sender.send_unicode_string(&[lo, mark]);
            } else {
                self.insert_key_length(1);
                sender.send_unicode_string(&[lo]);
            }
        }
    }

    fn send_new_char_string<S: EventSender>(
        &mut self,
        data_from_macro: bool,
        offset: usize,
        keycode: u16,
        flag: u64,
        sender: &mut S,
    ) {
        let mut new_char_string: Vec<u16> = Vec::with_capacity(20);
        let total_items = if data_from_macro {
            self.engine.hook_state.macro_data.len()
        } else {
            self.engine.hook_state.new_char_count as usize
        };
        let mut will_continue_sending = false;
        let mut will_send_control_key = false;
        let mut next_k = 0usize;
        let code_table = self.engine.config.code_table;

        if total_items > 0 {
            if data_from_macro {
                let mut k = offset;
                while k < total_items {
                    if new_char_string.len() >= 16 {
                        will_continue_sending = true;
                        next_k = k;
                        break;
                    }
                    let temp_char = self.engine.hook_state.macro_data[k];
                    self.append_char_unit(temp_char, code_table, &mut new_char_string);
                    k += 1;
                }
            } else {
                let start = (self.engine.hook_state.new_char_count as isize) - 1 - (offset as isize);
                let mut k = start;
                while k >= 0 {
                    if new_char_string.len() >= 16 {
                        will_continue_sending = true;
                        break;
                    }
                    let temp_char = self.engine.hook_state.char_data[k as usize];
                    self.append_char_unit(temp_char, code_table, &mut new_char_string);
                    k -= 1;
                }
            }
        }

        let h_code = self.engine.hook_state.code;
        if !will_continue_sending
            && (h_code == (HookCodeState::Restore as u8)
                || h_code == (HookCodeState::RestoreAndStartNewSession as u8))
        {
            if key_code_to_character(keycode as u32) != 0 {
                let is_caps = (flag & (CG_FLAG_ALPHA_SHIFT | CG_FLAG_SHIFT)) != 0;
                let ch = key_code_to_character(
                    (keycode as u32) | if is_caps { CAPS_MASK } else { 0 },
                );
                new_char_string.push(ch);
            } else {
                will_send_control_key = true;
            }
        }

        if !will_continue_sending && h_code == (HookCodeState::RestoreAndStartNewSession as u8) {
            self.engine.start_new_session();
        }

        let send_len = if will_continue_sending {
            16.min(new_char_string.len())
        } else {
            new_char_string.len()
        };
        if send_len > 0 {
            sender.send_unicode_string(&new_char_string[..send_len]);
        }

        if will_continue_sending {
            let next_offset = if data_from_macro { next_k } else { 16 };
            self.send_new_char_string(data_from_macro, next_offset, keycode, flag, sender);
        }

        if will_send_control_key {
            self.send_key_code(keycode as u32, sender);
        }
    }

    fn append_char_unit(&mut self, temp_char: u32, code_table: i32, out: &mut Vec<u16>) {
        if (temp_char & PURE_CHARACTER_MASK) != 0 {
            out.push(temp_char as u16);
            if is_double_code(code_table) {
                self.insert_key_length(1);
            }
        } else if (temp_char & CHAR_CODE_MASK) == 0 {
            if is_double_code(code_table) {
                self.insert_key_length(1);
            }
            out.push(key_code_to_character(temp_char));
        } else if code_table == 0 {
            out.push(temp_char as u16);
        } else if code_table == 1 || code_table == 2 || code_table == 4 {
            let new_char = temp_char as u16;
            let hi = (new_char >> 8) & 0xFF;
            let lo = new_char & 0xFF;
            out.push(lo);
            if hi > 32 {
                if code_table == 2 {
                    self.insert_key_length(2);
                }
                out.push(hi);
            } else if code_table == 2 {
                self.insert_key_length(1);
            }
        } else if code_table == 3 {
            let new_char = temp_char as u16;
            let hi = new_char >> 13;
            let lo = new_char & 0x1FFF;
            self.insert_key_length(if hi > 0 { 2 } else { 1 });
            out.push(lo);
            if hi > 0 {
                out.push(UNICODE_COMPOUND_MARK[(hi - 1) as usize]);
            }
        }
    }

    fn handle_macro<S: EventSender>(&mut self, keycode: u16, flag: u64, sender: &mut S) {
        if should_use_recommend_workaround(
            self.engine.config.fix_recommend_browser != 0,
            self.spotlight_visible,
            &self.frontmost_app,
        ) {
            self.send_empty_character(sender);
            self.engine.hook_state.backspace_count += 1;
        }

        let bpc = self.engine.hook_state.backspace_count as usize;
        for _ in 0..bpc {
            self.send_backspace(sender);
        }

        if !self.send_key_step_by_step {
            self.send_new_char_string(true, 0, keycode, flag, sender);
        } else {
            let macro_data = self.engine.hook_state.macro_data.clone();
            for item in macro_data {
                if (item & PURE_CHARACTER_MASK) != 0 {
                    self.send_pure_character(item as u16, sender);
                } else {
                    self.send_key_code(item, sender);
                }
            }
        }
        let shift_mask = if (flag & CG_FLAG_SHIFT) != 0 {
            CAPS_MASK
        } else {
            0
        };
        self.send_key_code((keycode as u32) | shift_mask, sender);
    }

    pub fn handle_raw_event<S: EventSender>(
        &mut self,
        event_type: MacEventType,
        keycode: u16,
        flag: u64,
        is_own_event_source: bool,
        sender: &mut S,
    ) -> CallbackAction {
        if event_type == MacEventType::TapDisabledByTimeout
            || event_type == MacEventType::TapDisabledByUserInput
        {
            sender.re_enable_event_tap();
            return CallbackAction::PassThrough;
        }

        if is_own_event_source {
            return CallbackAction::PassThrough;
        }

        let switch_key_status = self.engine.config.switch_key_status;
        let convert_hot_key = self.engine.convert_tool.hot_key;

        if event_type == MacEventType::KeyDown {
            if get_switch_key(switch_key_status) != keycode
                && get_switch_key(convert_hot_key) != keycode
            {
                self.last_flag = 0;
            } else {
                if get_switch_key(switch_key_status) == keycode
                    && check_hot_key(
                        switch_key_status,
                        self.last_flag,
                        keycode,
                        get_switch_key(switch_key_status) != 0xFE,
                    )
                {
                    self.switch_language(sender);
                    self.last_flag = 0;
                    self.has_just_used_hotkey = true;
                    return CallbackAction::Swallow;
                }
                if get_switch_key(convert_hot_key) == keycode
                    && check_hot_key(
                        convert_hot_key,
                        self.last_flag,
                        keycode,
                        get_switch_key(convert_hot_key) != 0xFE,
                    )
                {
                    sender.on_quick_convert();
                    self.last_flag = 0;
                    self.has_just_used_hotkey = true;
                    return CallbackAction::Swallow;
                }
            }
            self.has_just_used_hotkey = self.last_flag != 0;
        } else if event_type == MacEventType::FlagsChanged {
            if self.last_flag == 0 || self.last_flag < flag {
                self.last_flag = flag;
            } else if self.last_flag > flag {
                if check_hot_key(
                    switch_key_status,
                    self.last_flag,
                    keycode,
                    get_switch_key(switch_key_status) != 0xFE,
                ) {
                    self.last_flag = 0;
                    self.switch_language(sender);
                    self.has_just_used_hotkey = true;
                    return CallbackAction::Swallow;
                }
                if check_hot_key(
                    convert_hot_key,
                    self.last_flag,
                    keycode,
                    get_switch_key(convert_hot_key) != 0xFE,
                ) {
                    self.last_flag = 0;
                    sender.on_quick_convert();
                    self.has_just_used_hotkey = true;
                    return CallbackAction::Swallow;
                }
                if self.engine.config.temp_off_spelling != 0
                    && !self.has_just_used_hotkey
                    && (self.last_flag & CG_FLAG_CONTROL) != 0
                {
                    self.engine.temp_off_spell_checking();
                }
                if self.engine.config.temp_off_open_key != 0
                    && !self.has_just_used_hotkey
                    && (self.last_flag & CG_FLAG_COMMAND) != 0
                {
                    self.engine.temp_off_engine(true);
                }
                self.last_flag = 0;
                self.has_just_used_hotkey = false;
            }
        }

        match event_type {
            MacEventType::KeyDown
            | MacEventType::KeyUp
            | MacEventType::LeftMouseDown
            | MacEventType::RightMouseDown
            | MacEventType::LeftMouseDragged
            | MacEventType::RightMouseDragged => {}
            _ => return CallbackAction::PassThrough,
        }

        let other_control_key = has_other_control_key(flag);

        // English mode
        if self.engine.config.language == 0 {
            if self.engine.config.use_macro != 0
                && self.engine.config.use_macro_in_english_mode != 0
                && event_type == MacEventType::KeyDown
            {
                let is_caps = (flag & (CG_FLAG_SHIFT | CG_FLAG_ALPHA_SHIFT)) != 0;
                self.engine.english_mode(
                    KeyEventState::KeyDown,
                    keycode,
                    is_caps,
                    other_control_key,
                );
                if self.engine.hook_state.code == (HookCodeState::ReplaceMacro as u8) {
                    self.handle_macro(keycode, flag, sender);
                    return CallbackAction::Swallow;
                }
            }
            return CallbackAction::PassThrough;
        }

        // Mouse events
        if matches!(
            event_type,
            MacEventType::LeftMouseDown
                | MacEventType::RightMouseDown
                | MacEventType::LeftMouseDragged
                | MacEventType::RightMouseDragged
        ) {
            self.request_new_session();
            return CallbackAction::PassThrough;
        }

        // If "turn off Vietnamese when in other language" mode is on
        if self.engine.config.other_language != 0 && !self.current_input_source_is_english {
            return CallbackAction::PassThrough;
        }

        if event_type == MacEventType::KeyDown {
            let caps_status = if (flag & CG_FLAG_SHIFT) != 0 {
                1
            } else if (flag & CG_FLAG_ALPHA_SHIFT) != 0 {
                2
            } else {
                0
            };

            self.engine.handle_event(
                KeyEvent::Keyboard,
                KeyEventState::KeyDown,
                keycode,
                caps_status,
                other_control_key,
            );

            let h_code = self.engine.hook_state.code;
            let ext_code = self.engine.hook_state.ext_code;
            let code_table = self.engine.config.code_table;

            if h_code == (HookCodeState::DoNothing as u8) {
                if is_double_code(code_table) {
                    if ext_code == 1 {
                        self.sync_key.clear();
                    } else if ext_code == 2 {
                        if let Some(last) = self.sync_key.pop() {
                            if last > 1
                                && (code_table == 2
                                    || !contain_unicode_compound_app(&self.frontmost_app))
                            {
                                sender.send_backspace();
                            }
                        }
                    } else if ext_code == 3 {
                        self.insert_key_length(1);
                    }
                }
                return CallbackAction::PassThrough;
            } else if h_code == (HookCodeState::WillProcess as u8)
                || h_code == (HookCodeState::Restore as u8)
                || h_code == (HookCodeState::RestoreAndStartNewSession as u8)
            {
                if should_use_recommend_workaround(
                    self.engine.config.fix_recommend_browser != 0,
                    self.spotlight_visible,
                    &self.frontmost_app,
                ) && ext_code != 4
                {
                    if self.fix_chromium_browser
                        && is_exact_unicode_compound_app(&self.frontmost_app)
                    {
                        if self.engine.hook_state.backspace_count > 0 {
                            self.send_shift_and_left_arrow(sender);
                            if self.engine.hook_state.backspace_count == 1 {
                                self.engine.hook_state.backspace_count -= 1;
                            }
                        }
                    } else {
                        self.send_empty_character(sender);
                        self.engine.hook_state.backspace_count += 1;
                    }
                }

                if should_use_selection_replacement(self.spotlight_visible, &self.frontmost_app)
                    && self.engine.hook_state.backspace_count > 0
                {
                    let bpc = self.engine.hook_state.backspace_count as usize;
                    for _ in 0..bpc {
                        self.send_shift_and_left_arrow(sender);
                    }
                    self.engine.hook_state.backspace_count = 0;
                }

                let bpc = self.engine.hook_state.backspace_count as usize;
                if bpc > 0 && bpc < MAX_BUFF {
                    for _ in 0..bpc {
                        self.send_backspace(sender);
                    }
                }

                if !self.send_key_step_by_step {
                    self.send_new_char_string(false, 0, keycode, flag, sender);
                } else {
                    let ncc = self.engine.hook_state.new_char_count as usize;
                    if ncc > 0 && ncc <= MAX_BUFF {
                        for i in (0..ncc).rev() {
                            let cd = self.engine.hook_state.char_data[i];
                            self.send_key_code(cd, sender);
                        }
                    }
                    if h_code == (HookCodeState::Restore as u8)
                        || h_code == (HookCodeState::RestoreAndStartNewSession as u8)
                    {
                        let is_caps = (flag & (CG_FLAG_ALPHA_SHIFT | CG_FLAG_SHIFT)) != 0;
                        self.send_key_code(
                            (keycode as u32) | if is_caps { CAPS_MASK } else { 0 },
                            sender,
                        );
                    }
                    if h_code == (HookCodeState::RestoreAndStartNewSession as u8) {
                        self.engine.start_new_session();
                    }
                }
            } else if h_code == (HookCodeState::ReplaceMacro as u8) {
                self.handle_macro(keycode, flag, sender);
            }

            return CallbackAction::Swallow;
        }

        CallbackAction::PassThrough
    }
}
