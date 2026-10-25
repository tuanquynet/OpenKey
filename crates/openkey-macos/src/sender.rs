use openkey_engine::types::*;
use openkey_engine::vietnamese::{character_to_key_code, key_code_to_character};

use crate::handler::{CallbackAction, MacEventHandler, MacEventType};
use crate::hotkeys::CG_FLAG_SHIFT;

pub trait EventSender {
    fn re_enable_event_tap(&mut self);
    fn send_backspace(&mut self);
    fn send_shift_left_arrow(&mut self);
    fn send_empty_character(&mut self, ch: u16);
    fn send_unicode_string(&mut self, chars: &[u16]);
    fn send_key_code(&mut self, keycode: u16, shift: bool);
    fn beep(&mut self);
    fn on_quick_convert(&mut self);
    fn on_input_method_changed(&mut self, language: i32, will_notify: bool);
    fn on_code_table_changed(&mut self, code_table: i32);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SentAction {
    ReEnableTap,
    Backspace,
    ShiftLeftArrow,
    EmptyChar(u16),
    UnicodeString(Vec<u16>),
    KeyCode { keycode: u16, shift: bool },
    Beep,
    QuickConvert,
    InputMethodChanged { language: i32, will_notify: bool },
    CodeTableChanged { code_table: i32 },
}

#[derive(Debug, Clone, Default)]
pub struct RecordingEventSender {
    pub actions: Vec<SentAction>,
    pub tap_reenabled_count: usize,
    pub beep_count: usize,
    pub quick_convert_count: usize,
    buffer: Vec<u16>,
    selected_tail: usize,
}

impl RecordingEventSender {
    fn delete_selected_or_one(&mut self) {
        if self.selected_tail > 0 {
            let new_len = self.buffer.len().saturating_sub(self.selected_tail);
            self.buffer.truncate(new_len);
            self.selected_tail = 0;
        } else {
            self.buffer.pop();
        }
    }

    fn insert_units(&mut self, units: &[u16]) {
        if self.selected_tail > 0 {
            let new_len = self.buffer.len().saturating_sub(self.selected_tail);
            self.buffer.truncate(new_len);
            self.selected_tail = 0;
        }
        self.buffer.extend_from_slice(units);
    }

    pub fn rendered_text(&self) -> String {
        String::from_utf16_lossy(&self.buffer)
    }

    pub fn simulate_key_sequence(&mut self, handler: &mut MacEventHandler, sequence: &str) {
        for ch in sequence.chars() {
            let (keycode, flags) = if ch == '\x08' {
                (KEY_DELETE, 0u64)
            } else if ch == '\n' {
                (KEY_RETURN, 0u64)
            } else if ch == '\t' {
                (KEY_TAB, 0u64)
            } else if ch == '\x1b' {
                (KEY_ESC, 0u64)
            } else {
                let mapped = character_to_key_code(ch as u32)
                    .unwrap_or_else(|| panic!("unmapped char {:?}", ch));
                let kc = (mapped & CHAR_MASK) as u16;
                let fl = if (mapped & CAPS_MASK) != 0 {
                    CG_FLAG_SHIFT
                } else {
                    0
                };
                (kc, fl)
            };

            let action = handler.handle_raw_event(MacEventType::KeyDown, keycode, flags, false, self);
            if action == CallbackAction::PassThrough {
                if keycode == KEY_DELETE {
                    self.delete_selected_or_one();
                } else if keycode == KEY_RETURN || keycode == KEY_ENTER {
                    self.insert_units(&['\n' as u16]);
                } else if keycode == KEY_TAB {
                    self.insert_units(&['\t' as u16]);
                } else {
                    let ch_u16 = key_code_to_character(
                        (keycode as u32)
                            | if (flags & CG_FLAG_SHIFT) != 0 {
                                CAPS_MASK
                            } else {
                                0
                            },
                    );
                    if ch_u16 != 0 {
                        self.insert_units(&[ch_u16]);
                    }
                }
            }
        }
    }
}

impl EventSender for RecordingEventSender {
    fn re_enable_event_tap(&mut self) {
        self.tap_reenabled_count += 1;
        self.actions.push(SentAction::ReEnableTap);
    }

    fn send_backspace(&mut self) {
        self.actions.push(SentAction::Backspace);
        self.delete_selected_or_one();
    }

    fn send_shift_left_arrow(&mut self) {
        self.actions.push(SentAction::ShiftLeftArrow);
        if self.selected_tail < self.buffer.len() {
            self.selected_tail += 1;
        }
    }

    fn send_empty_character(&mut self, ch: u16) {
        self.actions.push(SentAction::EmptyChar(ch));
        self.insert_units(&[ch]);
    }

    fn send_unicode_string(&mut self, chars: &[u16]) {
        self.actions.push(SentAction::UnicodeString(chars.to_vec()));
        self.insert_units(chars);
    }

    fn send_key_code(&mut self, keycode: u16, shift: bool) {
        self.actions.push(SentAction::KeyCode { keycode, shift });
        if keycode == KEY_DELETE {
            self.delete_selected_or_one();
        } else if keycode == KEY_RETURN || keycode == KEY_ENTER {
            self.insert_units(&['\n' as u16]);
        } else if keycode == KEY_TAB {
            self.insert_units(&['\t' as u16]);
        } else {
            let ch_u16 = key_code_to_character(
                (keycode as u32) | if shift { CAPS_MASK } else { 0 },
            );
            if ch_u16 != 0 {
                self.insert_units(&[ch_u16]);
            }
        }
    }

    fn beep(&mut self) {
        self.beep_count += 1;
        self.actions.push(SentAction::Beep);
    }

    fn on_quick_convert(&mut self) {
        self.quick_convert_count += 1;
        self.actions.push(SentAction::QuickConvert);
    }

    fn on_input_method_changed(&mut self, language: i32, will_notify: bool) {
        self.actions.push(SentAction::InputMethodChanged {
            language,
            will_notify,
        });
    }

    fn on_code_table_changed(&mut self, code_table: i32) {
        self.actions
            .push(SentAction::CodeTableChanged { code_table });
    }
}
