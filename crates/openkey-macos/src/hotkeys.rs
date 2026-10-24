use openkey_engine::types::*;

pub const EMPTY_HOTKEY: i32 = 0xFE0000FEu32 as i32;

// macOS CGEventFlags masks
pub const CG_FLAG_ALPHA_SHIFT: u64 = 0x00010000;
pub const CG_FLAG_SHIFT: u64 = 0x00020000;
pub const CG_FLAG_CONTROL: u64 = 0x00040000;
pub const CG_FLAG_OPTION: u64 = 0x00080000;
pub const CG_FLAG_COMMAND: u64 = 0x00100000;
pub const CG_FLAG_HELP: u64 = 0x00400000;
pub const CG_FLAG_SECONDARY_FN: u64 = 0x00800000;
pub const CG_FLAG_NUMERIC_PAD: u64 = 0x00200000;
pub const CG_FLAG_NON_COALESCED: u64 = 0x00000100;

#[inline]
pub fn has_other_control_key(flag: u64) -> bool {
    (flag
        & (CG_FLAG_COMMAND
            | CG_FLAG_CONTROL
            | CG_FLAG_OPTION
            | CG_FLAG_SECONDARY_FN
            | CG_FLAG_NUMERIC_PAD
            | CG_FLAG_HELP))
        != 0
}

#[inline]
pub fn check_hot_key(hot_key_data: i32, last_flag: u64, keycode: u16, check_key_code: bool) -> bool {
    if (hot_key_data & !0x8000) == EMPTY_HOTKEY {
        return false;
    }
    if has_control(hot_key_data) ^ ((last_flag & CG_FLAG_CONTROL) != 0) {
        return false;
    }
    if has_option(hot_key_data) ^ ((last_flag & CG_FLAG_OPTION) != 0) {
        return false;
    }
    if has_command(hot_key_data) ^ ((last_flag & CG_FLAG_COMMAND) != 0) {
        return false;
    }
    if has_shift(hot_key_data) ^ ((last_flag & CG_FLAG_SHIFT) != 0) {
        return false;
    }
    if check_key_code && get_switch_key(hot_key_data) != keycode {
        return false;
    }
    true
}

/// Formats `convertToolHotKey` into the status menu item title matching `setQuickConvertString` in `AppDelegate.m`.
pub fn format_quick_convert_menu_title(convert_tool_hot_key: i32) -> String {
    let mut hot_key = String::new();
    let mut has_add = false;
    if (convert_tool_hot_key & 0x100) != 0 {
        hot_key.push('⌃');
        has_add = true;
    }
    if (convert_tool_hot_key & 0x200) != 0 {
        if has_add {
            hot_key.push_str(" + ");
        }
        hot_key.push('⌥');
        has_add = true;
    }
    if (convert_tool_hot_key & 0x400) != 0 {
        if has_add {
            hot_key.push_str(" + ");
        }
        hot_key.push('⌘');
        has_add = true;
    }
    if (convert_tool_hot_key & 0x800) != 0 {
        if has_add {
            hot_key.push_str(" + ");
        }
        hot_key.push('⇧');
        has_add = true;
    }

    let k = ((convert_tool_hot_key as u32) >> 24) & 0xFF;
    if k != 0xFE && k != 0 {
        if has_add {
            hot_key.push_str(" + ");
        }
        if k == (KEY_SPACE as u32) {
            hot_key.push_str("␣ ");
        } else {
            hot_key.push((k as u8 as char).to_ascii_uppercase());
        }
    }

    if has_add {
        format!("Chuyển mã nhanh - [{}]", hot_key.to_uppercase())
    } else {
        "Chuyển mã nhanh".to_string()
    }
}
