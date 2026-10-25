use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SmartSwitchStore {
    data: BTreeMap<String, i8>,
    cache_key: String,
    cache_data: i8,
}

impl SmartSwitchStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init_from_bytes(&mut self, bytes: &[u8]) {
        self.data.clear();
        self.cache_key.clear();
        self.cache_data = 0;
        if bytes.len() < 2 {
            return;
        }
        let count = u16::from_le_bytes([bytes[0], bytes[1]]) as usize;
        let mut cursor = 2usize;
        for _ in 0..count {
            if cursor >= bytes.len() {
                break;
            }
            let bundle_id_size = bytes[cursor] as usize;
            cursor += 1;
            if cursor + bundle_id_size > bytes.len() {
                break;
            }
            let bundle_id = String::from_utf8_lossy(&bytes[cursor..cursor + bundle_id_size]).into_owned();
            cursor += bundle_id_size;
            if cursor >= bytes.len() {
                break;
            }
            let value = bytes[cursor] as i8;
            cursor += 1;
            self.data.insert(bundle_id, value);
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let count = self.data.len() as u16;
        out.push((count & 0xFF) as u8);
        out.push((count >> 8) as u8);
        for (bundle_id, &val) in &self.data {
            let b = bundle_id.as_bytes();
            out.push(b.len() as u8);
            out.extend_from_slice(b);
            out.push(val as u8);
        }
        out
    }

    pub fn get_app_input_method_status(&mut self, bundle_id: &str, current_input_method: i32) -> i32 {
        if !self.cache_key.is_empty() && self.cache_key == bundle_id {
            return self.cache_data as i32;
        }
        if let Some(&val) = self.data.get(bundle_id) {
            self.cache_key = bundle_id.to_string();
            self.cache_data = val;
            return val as i32;
        }
        self.cache_key = bundle_id.to_string();
        self.cache_data = current_input_method as i8;
        self.data.insert(bundle_id.to_string(), self.cache_data);
        -1
    }

    pub fn set_app_input_method_status(&mut self, bundle_id: &str, language: i32) {
        let val = language as i8;
        self.data.insert(bundle_id.to_string(), val);
        self.cache_key = bundle_id.to_string();
        self.cache_data = val;
    }
}
