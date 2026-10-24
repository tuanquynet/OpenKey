use crate::vietnamese::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertToolConfig {
    pub dont_alert_when_completed: bool,
    pub to_all_caps: bool,
    pub to_all_non_caps: bool,
    pub to_caps_first_letter: bool,
    pub to_caps_each_word: bool,
    pub remove_mark: bool,
    pub from_code: u8,
    pub to_code: u8,
    pub hot_key: i32,
}

impl Default for ConvertToolConfig {
    fn default() -> Self {
        Self {
            dont_alert_when_completed: false,
            to_all_caps: false,
            to_all_non_caps: false,
            to_caps_first_letter: false,
            to_caps_each_word: false,
            remove_mark: false,
            from_code: 0,
            to_code: 0,
            hot_key: 0,
        }
    }
}

const BREAK_CHARS: [u16; 3] = ['.' as u16, '?' as u16, '!' as u16];

fn find_key_code(char_code: u32, code: usize) -> Option<(u32, usize)> {
    for &table_key in &CODE_TABLE_KEYS {
        if let Some(chars) = code_table_lookup(code, table_key) {
            for (z, &ch) in chars.iter().enumerate() {
                if char_code == (ch as u32) {
                    return Some((table_key, z));
                }
            }
        }
    }
    None
}

fn get_unicode_compound_mark_index(mark: u16) -> u16 {
    for (i, &m) in UNICODE_COMPOUND_MARK.iter().enumerate() {
        if mark == m {
            return ((i as u16) + 1) << 13;
        }
    }
    0
}

#[inline]
fn towupper_u16(ch: u16) -> u16 {
    if ch < 128 {
        (ch as u8 as char).to_ascii_uppercase() as u16
    } else {
        ch
    }
}

#[inline]
fn towlower_u16(ch: u16) -> u16 {
    if ch < 128 {
        (ch as u8 as char).to_ascii_lowercase() as u16
    } else {
        ch
    }
}

pub fn convert_util(source_string: &str, config: &ConvertToolConfig) -> String {
    let data: Vec<u16> = source_string.encode_utf16().collect();
    let mut temp: Vec<u16> = Vec::new();
    let mut has_break = false;
    let mut should_upper_case = config.to_caps_first_letter || config.to_caps_each_word;
    if config.to_all_non_caps {
        should_upper_case = false;
    }

    let from_code = config.from_code as usize;
    let to_code = config.to_code as usize;

    let mut i = 0usize;
    while i < data.len() {
        let mut p = 0usize;
        if i + 1 < data.len() {
            let t: u16 = match config.from_code {
                2 | 4 => {
                    p = 1;
                    data[i] | (data[i + 1] << 8)
                }
                3 => {
                    let target = get_unicode_compound_mark_index(data[i + 1]);
                    if target > 0 {
                        p = 1;
                        data[i] | target
                    } else {
                        data[i]
                    }
                }
                _ => data[i],
            };

            if let Some((j, k)) = find_key_code(t as u32, from_code) {
                i += p;
                let to_chars = code_table_lookup(to_code, j).unwrap_or(&[]);
                let mut target = to_chars.get(k).copied().unwrap_or(0);
                if (config.to_all_caps || should_upper_case) && k % 2 != 0 {
                    target = to_chars[k - 1];
                } else if (config.to_all_non_caps || !should_upper_case) && k % 2 == 0 {
                    if k + 1 < to_chars.len() {
                        target = to_chars[k + 1];
                    }
                }

                if config.remove_mark {
                    target = key_code_to_character((j & 0xFF) as u32);
                    if config.to_all_caps {
                        target = towupper_u16(target);
                    } else if config.to_all_non_caps {
                        target = towlower_u16(target);
                    }
                }

                match config.to_code {
                    0 | 1 => {
                        temp.push(target);
                    }
                    2 | 4 => {
                        let hi = (target >> 8) & 0xFF;
                        let lo = target & 0xFF;
                        temp.push(lo);
                        if hi > 32 {
                            temp.push(hi);
                        }
                    }
                    3 => {
                        if (target >> 13) > 0 {
                            temp.push(target & 0x1FFF);
                            temp.push(UNICODE_COMPOUND_MARK[((target >> 13) - 1) as usize]);
                        } else {
                            temp.push(target);
                        }
                    }
                    _ => {}
                }
                should_upper_case = false;
                has_break = false;
                i += 1;
                continue;
            }
        }

        let t = data[i];
        if let Some((j, k)) = find_key_code(t as u32, from_code) {
            let to_chars = code_table_lookup(to_code, j).unwrap_or(&[]);
            let mut target = to_chars.get(k).copied().unwrap_or(0);
            if (config.to_all_caps || should_upper_case) && k % 2 != 0 {
                target = to_chars[k - 1];
            } else if (config.to_all_non_caps || !should_upper_case) && k % 2 == 0 {
                if k + 1 < to_chars.len() {
                    target = to_chars[k + 1];
                }
            }

            if config.remove_mark {
                target = key_code_to_character((j & 0xFF) as u32);
                if config.to_all_caps {
                    target = towupper_u16(target);
                } else if config.to_all_non_caps {
                    target = towlower_u16(target);
                }
            }

            temp.push(target);
            should_upper_case = false;
            has_break = false;
            i += 1;
            continue;
        }

        if config.to_all_caps || should_upper_case {
            temp.push(towupper_u16(data[i]));
        } else if config.to_all_non_caps || !should_upper_case {
            temp.push(towlower_u16(data[i]));
        } else {
            temp.push(data[i]);
        }

        if t == ('\n' as u16) || (has_break && t == (' ' as u16)) {
            if config.to_caps_first_letter || config.to_caps_each_word {
                should_upper_case = true;
            }
        } else if t == (' ' as u16) && config.to_caps_each_word {
            should_upper_case = true;
        } else if BREAK_CHARS.contains(&t) {
            has_break = true;
        } else {
            should_upper_case = false;
            has_break = false;
        }

        i += 1;
    }

    String::from_utf16_lossy(&temp)
}
