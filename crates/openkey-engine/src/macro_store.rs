use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::types::*;
use crate::vietnamese::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroData {
    pub macro_text: String,
    pub macro_content: String,
    pub macro_content_code: Vec<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MacroStore {
    map: BTreeMap<Vec<u32>, MacroData>,
}

fn convert_str_to_macro_codes(s: &str, code_table: usize) -> Vec<u32> {
    let mut out = Vec::new();
    let utf16_units: Vec<u16> = s.encode_utf16().collect();
    for &unit in &utf16_units {
        let t = unit as u32;
        if let Some(kc) = character_to_key_code(t) {
            out.push(kc);
            continue;
        }

        let mut found = false;
        for &table_key in &CODE_TABLE_KEYS {
            if let Some(unicode_chars) = code_table_lookup(0, table_key) {
                for (k, &ch) in unicode_chars.iter().enumerate() {
                    if unit == ch {
                        if let Some(target_chars) = code_table_lookup(code_table, table_key) {
                            if k < target_chars.len() {
                                out.push((target_chars[k] as u32) | CHAR_CODE_MASK);
                                found = true;
                                break;
                            }
                        }
                    }
                }
            }
            if found {
                break;
            }
        }
        if found {
            continue;
        }

        out.push(t | PURE_CHARACTER_MASK);
    }
    out
}

fn modify_case_unicode(code: &mut u32, code_table: usize, is_upper_case: bool) -> bool {
    let char_buff = *code;
    if (*code & CHAR_CODE_MASK) == 0 {
        if is_upper_case {
            *code &= CAPS_MASK;
        } else {
            *code &= !CAPS_MASK;
        }
        return *code != char_buff;
    }

    for &table_key in &CODE_TABLE_KEYS {
        if let Some(chars) = code_table_lookup(code_table, table_key) {
            for (mut k_macro, &ch) in chars.iter().enumerate() {
                if (*code as u16) == ch {
                    if k_macro % 2 == 0 && !is_upper_case {
                        if k_macro + 1 < chars.len() {
                            k_macro += 1;
                        }
                    } else if k_macro % 2 != 0 && is_upper_case {
                        k_macro -= 1;
                    }
                    *code = (chars[k_macro] as u32) | CHAR_CODE_MASK;
                    return *code != char_buff;
                }
            }
        }
    }
    false
}

impl MacroStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init_from_bytes(&mut self, bytes: &[u8], code_table: usize) {
        self.map.clear();
        if bytes.len() < 2 {
            return;
        }
        let macro_count = u16::from_le_bytes([bytes[0], bytes[1]]) as usize;
        let mut cursor = 2usize;
        for _ in 0..macro_count {
            if cursor >= bytes.len() {
                break;
            }
            let text_size = bytes[cursor] as usize;
            cursor += 1;
            if cursor + text_size > bytes.len() {
                break;
            }
            let macro_text = String::from_utf8_lossy(&bytes[cursor..cursor + text_size]).into_owned();
            cursor += text_size;

            if cursor + 2 > bytes.len() {
                break;
            }
            let content_size = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
            cursor += 2;
            if cursor + content_size > bytes.len() {
                break;
            }
            let macro_content =
                String::from_utf8_lossy(&bytes[cursor..cursor + content_size]).into_owned();
            cursor += content_size;

            let key = convert_str_to_macro_codes(&macro_text, code_table);
            let macro_content_code = convert_str_to_macro_codes(&macro_content, code_table);
            self.map.insert(
                key,
                MacroData {
                    macro_text,
                    macro_content,
                    macro_content_code,
                },
            );
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let total = self.map.len() as u16;
        out.push((total & 0xFF) as u8);
        out.push((total >> 8) as u8);
        for data in self.map.values() {
            let text_bytes = data.macro_text.as_bytes();
            out.push(text_bytes.len() as u8);
            out.extend_from_slice(text_bytes);

            let content_bytes = data.macro_content.as_bytes();
            let content_size = content_bytes.len() as u16;
            out.push((content_size & 0xFF) as u8);
            out.push((content_size >> 8) as u8);
            out.extend_from_slice(content_bytes);
        }
        out
    }

    pub fn find_macro(
        &self,
        key: &mut Vec<u32>,
        code_table: usize,
        auto_caps_macro: bool,
    ) -> Option<Vec<u32>> {
        for elem in key.iter_mut() {
            *elem = get_character_code(*elem, code_table);
        }
        if let Some(data) = self.map.get(key) {
            return Some(data.macro_content_code.clone());
        }
        if auto_caps_macro {
            let mut macro_flag = false;
            if key.len() > 1 && modify_case_unicode(&mut key[1], code_table, false) {
                macro_flag = true;
                for item in key.iter_mut().skip(2) {
                    modify_case_unicode(item, code_table, false);
                }
            }
            if !key.is_empty() && modify_case_unicode(&mut key[0], code_table, false) {
                if let Some(data) = self.map.get(key) {
                    let mut macro_content_code = data.macro_content_code.clone();
                    for (c, item) in macro_content_code.iter_mut().enumerate() {
                        if c == 0 || macro_flag {
                            let k_char = key_code_to_character(*item);
                            if k_char != 0 {
                                let upper = (k_char as u8 as char).to_ascii_uppercase() as u32;
                                if let Some(mapped) = character_to_key_code(upper) {
                                    *item = mapped;
                                }
                                continue;
                            }
                            if (*item & CHAR_CODE_MASK) != 0 {
                                modify_case_unicode(item, code_table, true);
                            }
                        }
                    }
                    return Some(macro_content_code);
                }
            }
        }
        None
    }

    pub fn has_macro(&self, macro_name: &str, code_table: usize) -> bool {
        let key = convert_str_to_macro_codes(macro_name, code_table);
        self.map.contains_key(&key)
    }

    pub fn get_all_macros(&self) -> Vec<(&Vec<u32>, &str, &str)> {
        self.map
            .iter()
            .map(|(k, v)| (k, v.macro_text.as_str(), v.macro_content.as_str()))
            .collect()
    }

    pub fn add_macro(&mut self, macro_text: &str, macro_content: &str, code_table: usize) -> bool {
        let key = convert_str_to_macro_codes(macro_text, code_table);
        let macro_content_code = convert_str_to_macro_codes(macro_content, code_table);
        if let Some(existing) = self.map.get_mut(&key) {
            existing.macro_content = macro_content.to_string();
            existing.macro_content_code = macro_content_code;
        } else {
            self.map.insert(
                key,
                MacroData {
                    macro_text: macro_text.to_string(),
                    macro_content: macro_content.to_string(),
                    macro_content_code,
                },
            );
        }
        true
    }

    pub fn delete_macro(&mut self, macro_text: &str, code_table: usize) -> bool {
        let key = convert_str_to_macro_codes(macro_text, code_table);
        self.map.remove(&key).is_some()
    }

    pub fn on_table_code_change(&mut self, code_table: usize) {
        for data in self.map.values_mut() {
            data.macro_content_code = convert_str_to_macro_codes(&data.macro_content, code_table);
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut file = fs::File::create(path)?;
        writeln!(
            file,
            ";Compatible OpenKey Macro Data file for UniKey*** version=1 ***"
        )?;
        for data in self.map.values() {
            writeln!(file, "{}:{}", data.macro_text, data.macro_content)?;
        }
        Ok(())
    }

    pub fn read_from_file<P: AsRef<Path>>(
        &mut self,
        path: P,
        append: bool,
        code_table: usize,
    ) -> std::io::Result<()> {
        let content_str = fs::read_to_string(path)?;
        if !append {
            self.map.clear();
        }
        for (line_idx, line) in content_str.lines().enumerate() {
            if line_idx == 0 {
                continue;
            }
            if let Some(pos) = line.find(':') {
                let mut name = line[..pos].to_string();
                let mut content = line[pos + 1..].to_string();
                while name.is_empty() && !content.is_empty() {
                    if let Some(next_pos) = content.find(':') {
                        name.push(':');
                        name.push_str(&content[..next_pos]);
                        content = content[next_pos + 1..].to_string();
                    } else {
                        break;
                    }
                }
                if !name.is_empty() && !self.has_macro(&name, code_table) {
                    self.add_macro(&name, &content, code_table);
                }
            }
        }
        Ok(())
    }
}
