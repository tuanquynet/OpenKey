use crate::types::*;

pub const DOU_KEY: [[u16; 2]; 2] = [
    [KEY_A, 0xE2], // a -> â
    [KEY_E, 0xEA], // e -> ê
];

pub const UNICODE_COMPOUND_MARK: [u16; 5] = [0x0301, 0x0300, 0x0309, 0x0303, 0x0323];

pub fn vowel_table(key: u16) -> &'static [&'static [u16]] {
    match key {
        KEY_A => &[
            &[KEY_A, KEY_N, KEY_G],
            &[KEY_A, KEY_G | END_CONSONANT_MASK],
            &[KEY_A, KEY_N],
            &[KEY_A, KEY_M],
            &[KEY_A, KEY_U],
            &[KEY_A, KEY_Y],
            &[KEY_A, KEY_T],
            &[KEY_A, KEY_P],
            &[KEY_A],
            &[KEY_A, KEY_C],
        ],
        KEY_O => &[
            &[KEY_O, KEY_N, KEY_G],
            &[KEY_O, KEY_G | END_CONSONANT_MASK],
            &[KEY_O, KEY_N],
            &[KEY_O, KEY_M],
            &[KEY_O, KEY_I],
            &[KEY_O, KEY_C],
            &[KEY_O, KEY_T],
            &[KEY_O, KEY_P],
            &[KEY_O],
        ],
        KEY_E => &[
            &[KEY_E, KEY_N, KEY_H],
            &[KEY_E, KEY_H | END_CONSONANT_MASK],
            &[KEY_E, KEY_N, KEY_G],
            &[KEY_E, KEY_G | END_CONSONANT_MASK],
            &[KEY_E, KEY_C, KEY_H],
            &[KEY_E, KEY_K | END_CONSONANT_MASK],
            &[KEY_E, KEY_C],
            &[KEY_E, KEY_T],
            &[KEY_E, KEY_Y],
            &[KEY_E, KEY_U],
            &[KEY_E, KEY_P],
            &[KEY_E, KEY_C],
            &[KEY_E, KEY_N],
            &[KEY_E, KEY_M],
            &[KEY_E],
        ],
        KEY_W => &[
            &[KEY_O, KEY_N],
            &[KEY_U, KEY_O, KEY_N, KEY_G],
            &[KEY_U, KEY_O, KEY_G | END_CONSONANT_MASK],
            &[KEY_U, KEY_O, KEY_N],
            &[KEY_U, KEY_O, KEY_I],
            &[KEY_U, KEY_O, KEY_C],
            &[KEY_O, KEY_I],
            &[KEY_O, KEY_P],
            &[KEY_O, KEY_M],
            &[KEY_O, KEY_A],
            &[KEY_O, KEY_T],
            &[KEY_U, KEY_N, KEY_G],
            &[KEY_U, KEY_G | END_CONSONANT_MASK],
            &[KEY_A, KEY_N, KEY_G],
            &[KEY_A, KEY_G | END_CONSONANT_MASK],
            &[KEY_U, KEY_N],
            &[KEY_U, KEY_M],
            &[KEY_U, KEY_C],
            &[KEY_U, KEY_A],
            &[KEY_U, KEY_I],
            &[KEY_U, KEY_T],
            &[KEY_U],
            &[KEY_A, KEY_P],
            &[KEY_A, KEY_T],
            &[KEY_A, KEY_M],
            &[KEY_A, KEY_N],
            &[KEY_A],
            &[KEY_A, KEY_C],
            &[KEY_A, KEY_C, KEY_H],
            &[KEY_A, KEY_K | END_CONSONANT_MASK],
            &[KEY_O],
            &[KEY_U, KEY_U],
        ],
        _ => &[],
    }
}

pub fn vowel_combine_table(key: u16) -> &'static [&'static [u32]] {
    const KA: u32 = KEY_A as u32;
    const KE: u32 = KEY_E as u32;
    const KI: u32 = KEY_I as u32;
    const KO: u32 = KEY_O as u32;
    const KU: u32 = KEY_U as u32;
    const KY: u32 = KEY_Y as u32;

    match key {
        KEY_A => &[
            &[0, KA, KI],
            &[0, KA, KO],
            &[0, KA, KU],
            &[0, KA | TONE_MASK, KU],
            &[0, KA, KY],
            &[0, KA | TONE_MASK, KY],
        ],
        KEY_E => &[
            &[0, KE, KO],
            &[0, KE | TONE_MASK, KU],
        ],
        KEY_I => &[
            &[1, KI, KE | TONE_MASK, KU],
            &[0, KI, KA],
            &[1, KI, KE | TONE_MASK],
            &[0, KI, KU],
        ],
        KEY_O => &[
            &[0, KO, KA, KI],
            &[0, KO, KA, KO],
            &[0, KO, KA, KY],
            &[0, KO, KE, KO],
            &[1, KO, KA],
            &[1, KO, KA | TONEW_MASK],
            &[1, KO, KE],
            &[0, KO, KI],
            &[0, KO | TONE_MASK, KI],
            &[0, KO | TONEW_MASK, KI],
            &[1, KO, KO],
            &[1, KO | TONE_MASK, KO | TONE_MASK],
        ],
        KEY_U => &[
            &[0, KU, KY, KU],
            &[1, KU, KY, KE | TONE_MASK],
            &[0, KU, KY, KA],
            &[0, KU | TONEW_MASK, KO | TONEW_MASK, KU],
            &[0, KU | TONEW_MASK, KO | TONEW_MASK, KI],
            &[0, KU, KO | TONE_MASK, KI],
            &[0, KU, KA | TONE_MASK, KY],
            &[1, KU, KA, KO],
            &[1, KU, KA],
            &[1, KU, KA | TONEW_MASK],
            &[1, KU, KA | TONE_MASK],
            &[0, KU | TONEW_MASK, KA],
            &[1, KU, KE | TONE_MASK],
            &[0, KU, KI],
            &[0, KU | TONEW_MASK, KI],
            &[1, KU, KO],
            &[1, KU, KO | TONE_MASK],
            &[0, KU, KO | TONEW_MASK],
            &[1, KU | TONEW_MASK, KO | TONEW_MASK],
            &[0, KU | TONEW_MASK, KU],
            &[1, KU, KY],
        ],
        KEY_Y => &[
            &[0, KY, KE | TONE_MASK, KU],
            &[1, KY, KE | TONE_MASK],
        ],
        _ => &[],
    }
}

pub const CONSONANT_D: &[&[u16]] = &[
    &[KEY_D, KEY_E, KEY_N, KEY_H],
    &[KEY_D, KEY_E, KEY_H | END_CONSONANT_MASK],
    &[KEY_D, KEY_E, KEY_N, KEY_G],
    &[KEY_D, KEY_E, KEY_G | END_CONSONANT_MASK],
    &[KEY_D, KEY_E, KEY_C, KEY_H],
    &[KEY_D, KEY_E, KEY_K | END_CONSONANT_MASK],
    &[KEY_D, KEY_E, KEY_N],
    &[KEY_D, KEY_E, KEY_C],
    &[KEY_D, KEY_E, KEY_M],
    &[KEY_D, KEY_E],
    &[KEY_D, KEY_E, KEY_T],
    &[KEY_D, KEY_E, KEY_U],
    &[KEY_D, KEY_E, KEY_O],
    &[KEY_D, KEY_E, KEY_P],
    &[KEY_D, KEY_U, KEY_N, KEY_G],
    &[KEY_D, KEY_U, KEY_G | END_CONSONANT_MASK],
    &[KEY_D, KEY_U, KEY_N],
    &[KEY_D, KEY_U, KEY_M],
    &[KEY_D, KEY_U, KEY_C],
    &[KEY_D, KEY_U, KEY_O],
    &[KEY_D, KEY_U, KEY_A],
    &[KEY_D, KEY_U, KEY_O, KEY_I],
    &[KEY_D, KEY_U, KEY_O, KEY_C],
    &[KEY_D, KEY_U, KEY_O, KEY_N],
    &[KEY_D, KEY_U, KEY_O, KEY_N, KEY_G],
    &[KEY_D, KEY_U, KEY_O, KEY_G | END_CONSONANT_MASK],
    &[KEY_D, KEY_U],
    &[KEY_D, KEY_U, KEY_P],
    &[KEY_D, KEY_U, KEY_T],
    &[KEY_D, KEY_U, KEY_I],
    &[KEY_D, KEY_I, KEY_C, KEY_H],
    &[KEY_D, KEY_I, KEY_K | END_CONSONANT_MASK],
    &[KEY_D, KEY_I, KEY_C],
    &[KEY_D, KEY_I, KEY_N, KEY_H],
    &[KEY_D, KEY_I, KEY_H | END_CONSONANT_MASK],
    &[KEY_D, KEY_I, KEY_N],
    &[KEY_D, KEY_I],
    &[KEY_D, KEY_I, KEY_A],
    &[KEY_D, KEY_I, KEY_E],
    &[KEY_D, KEY_I, KEY_E, KEY_C],
    &[KEY_D, KEY_I, KEY_E, KEY_U],
    &[KEY_D, KEY_I, KEY_E, KEY_N],
    &[KEY_D, KEY_I, KEY_E, KEY_M],
    &[KEY_D, KEY_I, KEY_E, KEY_P],
    &[KEY_D, KEY_I, KEY_T],
    &[KEY_D, KEY_O],
    &[KEY_D, KEY_O, KEY_A],
    &[KEY_D, KEY_O, KEY_A, KEY_N],
    &[KEY_D, KEY_O, KEY_A, KEY_N, KEY_G],
    &[KEY_D, KEY_O, KEY_A, KEY_G | END_CONSONANT_MASK],
    &[KEY_D, KEY_O, KEY_A, KEY_N, KEY_H],
    &[KEY_D, KEY_O, KEY_A, KEY_H | END_CONSONANT_MASK],
    &[KEY_D, KEY_O, KEY_A, KEY_M],
    &[KEY_D, KEY_O, KEY_E],
    &[KEY_D, KEY_O, KEY_I],
    &[KEY_D, KEY_O, KEY_P],
    &[KEY_D, KEY_O, KEY_C],
    &[KEY_D, KEY_O, KEY_N],
    &[KEY_D, KEY_O, KEY_N, KEY_G],
    &[KEY_D, KEY_O, KEY_G | END_CONSONANT_MASK],
    &[KEY_D, KEY_O, KEY_M],
    &[KEY_D, KEY_O, KEY_T],
    &[KEY_D, KEY_A],
    &[KEY_D, KEY_A, KEY_T],
    &[KEY_D, KEY_A, KEY_Y],
    &[KEY_D, KEY_A, KEY_U],
    &[KEY_D, KEY_A, KEY_I],
    &[KEY_D, KEY_A, KEY_O],
    &[KEY_D, KEY_A, KEY_P],
    &[KEY_D, KEY_A, KEY_C],
    &[KEY_D, KEY_A, KEY_C, KEY_H],
    &[KEY_D, KEY_A, KEY_K | END_CONSONANT_MASK],
    &[KEY_D, KEY_A, KEY_N],
    &[KEY_D, KEY_A, KEY_N, KEY_H],
    &[KEY_D, KEY_A, KEY_H | END_CONSONANT_MASK],
    &[KEY_D, KEY_A, KEY_N, KEY_G],
    &[KEY_D, KEY_A, KEY_G | END_CONSONANT_MASK],
    &[KEY_D, KEY_A, KEY_M],
    &[KEY_D],
];

// Note: C++ std::map<Uint16, vector<vector<Uint16>>> _vowelForMark iterates in key order:
// KEY_A (0), KEY_E (14), KEY_Y (16), KEY_O (31), KEY_U (32), KEY_I (34)
pub const VOWEL_FOR_MARK_KEYS: [u16; 6] = [KEY_A, KEY_E, KEY_Y, KEY_O, KEY_U, KEY_I];

pub fn vowel_for_mark_table(key: u16) -> &'static [&'static [u16]] {
    match key {
        KEY_A => &[
            &[KEY_A, KEY_N, KEY_G],
            &[KEY_A, KEY_G | END_CONSONANT_MASK],
            &[KEY_A, KEY_N],
            &[KEY_A, KEY_N, KEY_H],
            &[KEY_A, KEY_H | END_CONSONANT_MASK],
            &[KEY_A, KEY_M],
            &[KEY_A, KEY_U],
            &[KEY_A, KEY_Y],
            &[KEY_A, KEY_T],
            &[KEY_A, KEY_P],
            &[KEY_A],
            &[KEY_A, KEY_C],
            &[KEY_A, KEY_I],
            &[KEY_A, KEY_O],
            &[KEY_A, KEY_C, KEY_H],
            &[KEY_A, KEY_K | END_CONSONANT_MASK],
        ],
        KEY_O => &[
            &[KEY_O, KEY_O, KEY_N, KEY_G],
            &[KEY_O, KEY_O, KEY_G | END_CONSONANT_MASK],
            &[KEY_O, KEY_N, KEY_G],
            &[KEY_O, KEY_G | END_CONSONANT_MASK],
            &[KEY_O, KEY_O, KEY_N],
            &[KEY_O, KEY_O, KEY_C],
            &[KEY_O, KEY_O],
            &[KEY_O, KEY_N],
            &[KEY_O, KEY_M],
            &[KEY_O, KEY_I],
            &[KEY_O, KEY_C],
            &[KEY_O, KEY_T],
            &[KEY_O, KEY_P],
            &[KEY_O],
        ],
        KEY_E => &[
            &[KEY_E, KEY_N, KEY_H],
            &[KEY_E, KEY_H | END_CONSONANT_MASK],
            &[KEY_E, KEY_N, KEY_G],
            &[KEY_E, KEY_G | END_CONSONANT_MASK],
            &[KEY_E, KEY_C, KEY_H],
            &[KEY_E, KEY_K | END_CONSONANT_MASK],
            &[KEY_E, KEY_C],
            &[KEY_E, KEY_T],
            &[KEY_E, KEY_Y],
            &[KEY_E, KEY_U],
            &[KEY_E, KEY_P],
            &[KEY_E, KEY_C],
            &[KEY_E, KEY_N],
            &[KEY_E, KEY_M],
            &[KEY_E],
        ],
        KEY_I => &[
            &[KEY_I, KEY_N, KEY_H],
            &[KEY_I, KEY_H | END_CONSONANT_MASK],
            &[KEY_I, KEY_C, KEY_H],
            &[KEY_I, KEY_K | END_CONSONANT_MASK],
            &[KEY_I, KEY_N],
            &[KEY_I, KEY_T],
            &[KEY_I, KEY_U],
            &[KEY_I, KEY_U, KEY_P],
            &[KEY_I, KEY_N],
            &[KEY_I, KEY_M],
            &[KEY_I, KEY_P],
            &[KEY_I, KEY_A],
            &[KEY_I, KEY_C],
            &[KEY_I],
        ],
        KEY_U => &[
            &[KEY_U, KEY_N, KEY_G],
            &[KEY_U, KEY_G | END_CONSONANT_MASK],
            &[KEY_U, KEY_I],
            &[KEY_U, KEY_O],
            &[KEY_U, KEY_Y],
            &[KEY_U, KEY_Y, KEY_N],
            &[KEY_U, KEY_Y, KEY_T],
            &[KEY_U, KEY_Y, KEY_P],
            &[KEY_U, KEY_Y, KEY_C, KEY_H],
            &[KEY_U, KEY_Y, KEY_K | END_CONSONANT_MASK],
            &[KEY_U, KEY_Y, KEY_N, KEY_H],
            &[KEY_U, KEY_Y, KEY_H | END_CONSONANT_MASK],
            &[KEY_U, KEY_T],
            &[KEY_U, KEY_U],
            &[KEY_U, KEY_A],
            &[KEY_U, KEY_I],
            &[KEY_U, KEY_C],
            &[KEY_U, KEY_N],
            &[KEY_U, KEY_M],
            &[KEY_U, KEY_P],
            &[KEY_U],
        ],
        KEY_Y => &[&[KEY_Y]],
        _ => &[],
    }
}

pub const CONSONANT_TABLE: &[&[u16]] = &[
    &[KEY_N, KEY_G, KEY_H],
    &[KEY_P, KEY_H],
    &[KEY_T, KEY_H],
    &[KEY_T, KEY_R],
    &[KEY_G, KEY_I],
    &[KEY_C, KEY_H],
    &[KEY_N, KEY_H],
    &[KEY_N, KEY_G],
    &[KEY_K, KEY_H],
    &[KEY_G, KEY_H],
    &[KEY_G],
    &[KEY_C],
    &[KEY_Q],
    &[KEY_K],
    &[KEY_T],
    &[KEY_R],
    &[KEY_H],
    &[KEY_B],
    &[KEY_M],
    &[KEY_V],
    &[KEY_N],
    &[KEY_L],
    &[KEY_X],
    &[KEY_P],
    &[KEY_S],
    &[KEY_D],
    &[KEY_F | CONSONANT_ALLOW_MASK],
    &[KEY_W | CONSONANT_ALLOW_MASK],
    &[KEY_Z | CONSONANT_ALLOW_MASK],
    &[KEY_J | CONSONANT_ALLOW_MASK],
    &[KEY_F | END_CONSONANT_MASK],
    &[KEY_W | END_CONSONANT_MASK],
    &[KEY_J | END_CONSONANT_MASK],
];

pub const END_CONSONANT_TABLE: &[&[u16]] = &[
    &[KEY_T],
    &[KEY_P],
    &[KEY_C],
    &[KEY_N],
    &[KEY_M],
    &[KEY_G | END_CONSONANT_MASK],
    &[KEY_K | END_CONSONANT_MASK],
    &[KEY_H | END_CONSONANT_MASK],
    &[KEY_C, KEY_H],
    &[KEY_N, KEY_H],
    &[KEY_N, KEY_G],
];

pub const STANDALONE_W_BAD: &[u16] = &[KEY_W, KEY_E, KEY_Y, KEY_F, KEY_J, KEY_K, KEY_Z];

pub const DOUBLE_W_ALLOWED: &[[u16; 2]] = &[
    [KEY_T, KEY_R],
    [KEY_T, KEY_H],
    [KEY_C, KEY_H],
    [KEY_N, KEY_H],
    [KEY_N, KEY_G],
    [KEY_K, KEY_H],
    [KEY_G, KEY_I],
    [KEY_P, KEY_H],
    [KEY_G, KEY_H],
];

pub fn quick_telex(key: u32) -> Option<[u16; 2]> {
    match key as u16 {
        KEY_C => Some([KEY_C, KEY_H]),
        KEY_G => Some([KEY_G, KEY_I]),
        KEY_K => Some([KEY_K, KEY_H]),
        KEY_N => Some([KEY_N, KEY_G]),
        KEY_Q => Some([KEY_Q, KEY_U]),
        KEY_P => Some([KEY_P, KEY_H]),
        KEY_T => Some([KEY_T, KEY_H]),
        KEY_U => Some([KEY_U, KEY_U]),
        _ => None,
    }
}

pub fn quick_start_consonant(key: u16) -> Option<[u16; 2]> {
    match key {
        KEY_F => Some([KEY_P, KEY_H]),
        KEY_J => Some([KEY_G, KEY_I]),
        KEY_W => Some([KEY_Q, KEY_U]),
        _ => None,
    }
}

pub fn quick_end_consonant(key: u16) -> Option<[u16; 2]> {
    match key {
        KEY_G => Some([KEY_N, KEY_G]),
        KEY_H => Some([KEY_N, KEY_H]),
        KEY_K => Some([KEY_C, KEY_H]),
        _ => None,
    }
}

// Note: C++ std::map<Uint32, vector<Uint16>> _codeTable[i] is ordered by Uint32 key ascending:
// KEY_A (0), KEY_D (2), KEY_E (14), KEY_Y (16), KEY_O (31), KEY_U (32), KEY_I (34),
// KEY_A|TONE_MASK (0x20000), KEY_E|TONE_MASK (0x2000E), KEY_O|TONE_MASK (0x2001F),
// KEY_A|TONEW_MASK (0x40000), KEY_O|TONEW_MASK (0x4001F), KEY_U|TONEW_MASK (0x40020)
pub const CODE_TABLE_KEYS: [u32; 13] = [
    KEY_A as u32,
    KEY_D as u32,
    KEY_E as u32,
    KEY_Y as u32,
    KEY_O as u32,
    KEY_U as u32,
    KEY_I as u32,
    (KEY_A as u32) | TONE_MASK,
    (KEY_E as u32) | TONE_MASK,
    (KEY_O as u32) | TONE_MASK,
    (KEY_A as u32) | TONEW_MASK,
    (KEY_O as u32) | TONEW_MASK,
    (KEY_U as u32) | TONEW_MASK,
];

pub fn code_table_lookup(table_index: usize, key: u32) -> Option<&'static [u16]> {
    const KA: u32 = KEY_A as u32;
    const KD: u32 = KEY_D as u32;
    const KE: u32 = KEY_E as u32;
    const KI: u32 = KEY_I as u32;
    const KO: u32 = KEY_O as u32;
    const KU: u32 = KEY_U as u32;
    const KY: u32 = KEY_Y as u32;

    const KA_T: u32 = KA | TONE_MASK;
    const KA_W: u32 = KA | TONEW_MASK;
    const KO_T: u32 = KO | TONE_MASK;
    const KO_W: u32 = KO | TONEW_MASK;
    const KU_W: u32 = KU | TONEW_MASK;
    const KE_T: u32 = KE | TONE_MASK;

    match table_index {
        0 => match key {
            KA => Some(&[0x00C2, 0x00E2, 0x0102, 0x0103, 0x00C1, 0x00E1, 0x00C0, 0x00E0, 0x1EA2, 0x1EA3, 0x00C3, 0x00E3, 0x1EA0, 0x1EA1]),
            KO => Some(&[0x00D4, 0x00F4, 0x01A0, 0x01A1, 0x00D3, 0x00F3, 0x00D2, 0x00F2, 0x1ECE, 0x1ECF, 0x00D5, 0x00F5, 0x1ECC, 0x1ECD]),
            KU => Some(&[0x0000, 0x0000, 0x01AF, 0x01B0, 0x00DA, 0x00FA, 0x00D9, 0x00F9, 0x1EE6, 0x1EE7, 0x0168, 0x0169, 0x1EE4, 0x1EE5]),
            KE => Some(&[0x00CA, 0x00EA, 0x0000, 0x0000, 0x00C9, 0x00E9, 0x00C8, 0x00E8, 0x1EBA, 0x1EBB, 0x1EBC, 0x1EBD, 0x1EB8, 0x1EB9]),
            KD => Some(&[0x0110, 0x0111]),
            KA_T => Some(&[0x1EA4, 0x1EA5, 0x1EA6, 0x1EA7, 0x1EA8, 0x1EA9, 0x1EAA, 0x1EAB, 0x1EAC, 0x1EAD]),
            KA_W => Some(&[0x1EAE, 0x1EAF, 0x1EB0, 0x1EB1, 0x1EB2, 0x1EB3, 0x1EB4, 0x1EB5, 0x1EB6, 0x1EB7]),
            KO_T => Some(&[0x1ED0, 0x1ED1, 0x1ED2, 0x1ED3, 0x1ED4, 0x1ED5, 0x1ED6, 0x1ED7, 0x1ED8, 0x1ED9]),
            KO_W => Some(&[0x1EDA, 0x1EDB, 0x1EDC, 0x1EDD, 0x1EDE, 0x1EDF, 0x1EE0, 0x1EE1, 0x1EE2, 0x1EE3]),
            KU_W => Some(&[0x1EE8, 0x1EE9, 0x1EEA, 0x1EEB, 0x1EEC, 0x1EED, 0x1EEE, 0x1EEF, 0x1EF0, 0x1EF1]),
            KE_T => Some(&[0x1EBE, 0x1EBF, 0x1EC0, 0x1EC1, 0x1EC2, 0x1EC3, 0x1EC4, 0x1EC5, 0x1EC6, 0x1EC7]),
            KI => Some(&[0x00CD, 0x00ED, 0x00CC, 0x00EC, 0x1EC8, 0x1EC9, 0x0128, 0x0129, 0x1ECA, 0x1ECB]),
            KY => Some(&[0x00DD, 0x00FD, 0x1EF2, 0x1EF3, 0x1EF6, 0x1EF7, 0x1EF8, 0x1EF9, 0x1EF4, 0x1EF5]),
            _ => None,
        },
        1 => match key {
            KA => Some(&[0xA2, 0xA9, 0xA1, 0xA8, 0xB8, 0xB8, 0xB5, 0xB5, 0xB6, 0xB6, 0xB7, 0xB7, 0xB9, 0xB9]),
            KO => Some(&[0xA4, 0xAB, 0xA5, 0xAC, 0xE3, 0xE3, 0xDF, 0xDF, 0xE1, 0xE1, 0xE2, 0xE2, 0xE4, 0xE4]),
            KU => Some(&[0x00, 0x00, 0xA6, 0xAD, 0xF3, 0xF3, 0xEF, 0xEF, 0xF1, 0xF1, 0xF2, 0xF2, 0xF4, 0xF4]),
            KE => Some(&[0xA3, 0xAA, 0x00, 0x00, 0xD0, 0xD0, 0xCC, 0xCC, 0xCE, 0xCE, 0xCF, 0xCF, 0xD1, 0xD1]),
            KD => Some(&[0xA7, 0xAE]),
            KA_T => Some(&[0xCA, 0xCA, 0xC7, 0xC7, 0xC8, 0xC8, 0xC9, 0xC9, 0xCB, 0xCB]),
            KA_W => Some(&[0xBE, 0xBE, 0xBB, 0xBB, 0xBC, 0xBC, 0xBD, 0xBD, 0xC6, 0xC6]),
            KO_T => Some(&[0xE8, 0xE8, 0xE5, 0xE5, 0xE6, 0xE6, 0xE7, 0xE7, 0xE9, 0xE9]),
            KO_W => Some(&[0xED, 0xED, 0xEA, 0xEA, 0xEB, 0xEB, 0xEC, 0xEC, 0xEE, 0xEE]),
            KU_W => Some(&[0xF8, 0xF8, 0xF5, 0xF5, 0xF6, 0xF6, 0xF7, 0xF7, 0xF9, 0xF9]),
            KE_T => Some(&[0xD5, 0xD5, 0xD2, 0xD2, 0xD3, 0xD3, 0xD4, 0xD4, 0xD6, 0xD6]),
            KI => Some(&[0xDD, 0xDD, 0xD7, 0xD7, 0xD8, 0xD8, 0xDC, 0xDC, 0xDE, 0xDE]),
            KY => Some(&[0xFD, 0xFD, 0xFA, 0xFA, 0xFB, 0xFB, 0xFC, 0xFC, 0xFE, 0xFE]),
            _ => None,
        },
        2 => match key {
            KA => Some(&[0xC241, 0xE261, 0xCA41, 0xEA61, 0xD941, 0xF961, 0xD841, 0xF861, 0xDB41, 0xFB61, 0xD541, 0xF561, 0xCF41, 0xEF61]),
            KO => Some(&[0xC24F, 0xE26F, 0x00D4, 0x00F4, 0xD94F, 0xF96F, 0xD84F, 0xF86F, 0xDB4F, 0xFB6F, 0xD54F, 0xF56F, 0xCF4F, 0xEF6F]),
            KU => Some(&[0x0000, 0x0000, 0x00D6, 0x00F6, 0xD955, 0xF975, 0xD855, 0xF875, 0xDB55, 0xFB75, 0xD555, 0xF575, 0xCF55, 0xEF75]),
            KE => Some(&[0xC245, 0xE265, 0x0000, 0x0000, 0xD945, 0xF965, 0xD845, 0xF865, 0xDB45, 0xFB65, 0xD545, 0xF565, 0xCF45, 0xEF65]),
            KD => Some(&[0x00D1, 0x00F1]),
            KA_T => Some(&[0xC141, 0xE161, 0xC041, 0xE061, 0xC541, 0xE561, 0xC341, 0xE361, 0xC441, 0xE461]),
            KA_W => Some(&[0xC941, 0xE961, 0xC841, 0xE861, 0xDA41, 0xFA61, 0xDC41, 0xFC61, 0xCB41, 0xEB61]),
            KO_T => Some(&[0xC14F, 0xE16F, 0xC04F, 0xE06F, 0xC54F, 0xE56F, 0xC34F, 0xE36F, 0xC44F, 0xE46F]),
            KO_W => Some(&[0xD9D4, 0xF9F4, 0xD8D4, 0xF8F4, 0xDBD4, 0xFBF4, 0xD5D4, 0xF5F4, 0xCFD4, 0xEFF4]),
            KU_W => Some(&[0xD9D6, 0xF9F6, 0xD8D6, 0xF8F6, 0xDBD6, 0xFBF6, 0xD5D6, 0xF5F6, 0xCFD6, 0xEFF6]),
            KE_T => Some(&[0xC145, 0xE165, 0xC045, 0xE065, 0xC545, 0xE565, 0xC345, 0xE365, 0xC445, 0xE465]),
            KI => Some(&[0x00CD, 0x00ED, 0x00CC, 0x00EC, 0x00C6, 0x00E6, 0x00D3, 0x00F3, 0x00D2, 0x00F2]),
            KY => Some(&[0xD959, 0xF979, 0xD859, 0xF879, 0xDB59, 0xFB79, 0xD559, 0xF579, 0x00CE, 0x00EE]),
            _ => None,
        },
        3 => match key {
            KA => Some(&[0x00C2, 0x00E2, 0x0102, 0x0103, 0x2041, 0x2061, 0x4041, 0x4061, 0x6041, 0x6061, 0x8041, 0x8061, 0xA041, 0xA061]),
            KO => Some(&[0x00D4, 0x00F4, 0x01A0, 0x01A1, 0x204F, 0x206F, 0x404F, 0x406F, 0x604F, 0x606F, 0x804F, 0x806F, 0xA04F, 0xA06F]),
            KU => Some(&[0x0000, 0x0000, 0x01AF, 0x01B0, 0x2055, 0x2075, 0x4055, 0x4075, 0x6055, 0x6075, 0x8055, 0x8075, 0xA055, 0xA075]),
            KE => Some(&[0x00CA, 0x00EA, 0x0000, 0x0000, 0x2045, 0x2065, 0x4045, 0x4065, 0x6045, 0x6065, 0x8045, 0x8065, 0xA045, 0xA065]),
            KD => Some(&[0x0110, 0x0111]),
            KA_T => Some(&[0x20C2, 0x20E2, 0x40C2, 0x40E2, 0x60C2, 0x60E2, 0x80C2, 0x80E2, 0xA0C2, 0xA0E2]),
            KA_W => Some(&[0x2102, 0x2103, 0x4102, 0x4103, 0x6102, 0x6103, 0x8102, 0x8103, 0xA102, 0xA103]),
            KO_T => Some(&[0x20D4, 0x20F4, 0x40D4, 0x40F4, 0x60D4, 0x60F4, 0x80D4, 0x80F4, 0xA0D4, 0xA0F4]),
            KO_W => Some(&[0x21A0, 0x21A1, 0x41A0, 0x41A1, 0x61A0, 0x61A1, 0x81A0, 0x81A1, 0xA1A0, 0xA1A1]),
            KU_W => Some(&[0x21AF, 0x21B0, 0x41AF, 0x41B0, 0x61AF, 0x61B0, 0x81AF, 0x81B0, 0xA1AF, 0xA1B0]),
            KE_T => Some(&[0x20CA, 0x20EA, 0x40CA, 0x40EA, 0x60CA, 0x60EA, 0x80CA, 0x80EA, 0xA0CA, 0xA0EA]),
            KI => Some(&[0x2049, 0x2069, 0x4049, 0x4069, 0x6049, 0x6069, 0x8049, 0x8069, 0xA049, 0xA069]),
            KY => Some(&[0x2059, 0x2079, 0x4059, 0x4079, 0x6059, 0x6079, 0x8059, 0x8079, 0xA059, 0xA079]),
            _ => None,
        },
        4 => match key {
            KA => Some(&[0x00C2, 0x00E2, 0x00C3, 0x00E3, 0xEC41, 0xEC61, 0xCC41, 0xCC61, 0xD241, 0xD261, 0xDE41, 0xDE61, 0xF241, 0xF261]),
            KO => Some(&[0x00D4, 0x00F4, 0x00D5, 0x00F5, 0xEC4F, 0xEC6F, 0xCC4F, 0xCC6F, 0xD24F, 0xD26F, 0xDE4F, 0xDE6F, 0xF24F, 0xF26F]),
            KU => Some(&[0x0000, 0x0000, 0x00DD, 0x00FD, 0xEC55, 0xEC75, 0xCC55, 0xCC75, 0xD255, 0xD275, 0xDE55, 0xDE75, 0xF255, 0xF275]),
            KE => Some(&[0x00CA, 0x00EA, 0x0000, 0x0000, 0xEC45, 0xEC65, 0xCC45, 0xCC65, 0xD245, 0xD265, 0xDE45, 0xDE65, 0xF245, 0xF265]),
            KD => Some(&[0x00D0, 0x00F0]),
            KA_T => Some(&[0xECC2, 0xECE2, 0xCCC2, 0xCCE2, 0xD2C2, 0xD2E2, 0xDEC2, 0xDEE2, 0xF2C2, 0xF2E2]),
            KA_W => Some(&[0xECC3, 0xECE3, 0xCCC3, 0xCCE3, 0xD2C3, 0xD2E3, 0xDEC3, 0xDEE3, 0xF2C3, 0xF2E3]),
            KO_T => Some(&[0xECD4, 0xECF4, 0xCCD4, 0xCCF4, 0xD2D4, 0xD2F4, 0xDED4, 0xDEF4, 0xF2D4, 0xF2F4]),
            KO_W => Some(&[0xECD5, 0xECF5, 0xCCD5, 0xCCF5, 0xD2D5, 0xD2F5, 0xDED5, 0xDEF5, 0xF2D5, 0xF2F5]),
            KU_W => Some(&[0xECDD, 0xECFD, 0xCCDD, 0xCCFD, 0xD2DD, 0xD2FD, 0xDEDD, 0xDEFD, 0xF2DD, 0xF2FD]),
            KE_T => Some(&[0xECCA, 0xECEA, 0xCCCA, 0xCCEA, 0xD2CA, 0xD2EA, 0xDECA, 0xDEEA, 0xF2CA, 0xF2EA]),
            KI => Some(&[0xEC49, 0xEC69, 0xCC49, 0xCC69, 0xD249, 0xD269, 0xDE49, 0xDE69, 0xF249, 0xF269]),
            KY => Some(&[0xEC59, 0xEC79, 0xCC59, 0xCC79, 0xD259, 0xD279, 0xDE59, 0xDE79, 0xF259, 0xF279]),
            _ => None,
        },
        _ => None,
    }
}

pub const CHARACTER_MAP: &[(char, u32)] = &[
    ('a', KEY_A as u32), ('A', (KEY_A as u32) | CAPS_MASK),
    ('b', KEY_B as u32), ('B', (KEY_B as u32) | CAPS_MASK),
    ('c', KEY_C as u32), ('C', (KEY_C as u32) | CAPS_MASK),
    ('d', KEY_D as u32), ('D', (KEY_D as u32) | CAPS_MASK),
    ('e', KEY_E as u32), ('E', (KEY_E as u32) | CAPS_MASK),
    ('f', KEY_F as u32), ('F', (KEY_F as u32) | CAPS_MASK),
    ('g', KEY_G as u32), ('G', (KEY_G as u32) | CAPS_MASK),
    ('h', KEY_H as u32), ('H', (KEY_H as u32) | CAPS_MASK),
    ('i', KEY_I as u32), ('I', (KEY_I as u32) | CAPS_MASK),
    ('j', KEY_J as u32), ('J', (KEY_J as u32) | CAPS_MASK),
    ('k', KEY_K as u32), ('K', (KEY_K as u32) | CAPS_MASK),
    ('l', KEY_L as u32), ('L', (KEY_L as u32) | CAPS_MASK),
    ('m', KEY_M as u32), ('M', (KEY_M as u32) | CAPS_MASK),
    ('n', KEY_N as u32), ('N', (KEY_N as u32) | CAPS_MASK),
    ('o', KEY_O as u32), ('O', (KEY_O as u32) | CAPS_MASK),
    ('p', KEY_P as u32), ('P', (KEY_P as u32) | CAPS_MASK),
    ('q', KEY_Q as u32), ('Q', (KEY_Q as u32) | CAPS_MASK),
    ('r', KEY_R as u32), ('R', (KEY_R as u32) | CAPS_MASK),
    ('s', KEY_S as u32), ('S', (KEY_S as u32) | CAPS_MASK),
    ('t', KEY_T as u32), ('T', (KEY_T as u32) | CAPS_MASK),
    ('u', KEY_U as u32), ('U', (KEY_U as u32) | CAPS_MASK),
    ('v', KEY_V as u32), ('V', (KEY_V as u32) | CAPS_MASK),
    ('w', KEY_W as u32), ('W', (KEY_W as u32) | CAPS_MASK),
    ('x', KEY_X as u32), ('X', (KEY_X as u32) | CAPS_MASK),
    ('y', KEY_Y as u32), ('Y', (KEY_Y as u32) | CAPS_MASK),
    ('z', KEY_Z as u32), ('Z', (KEY_Z as u32) | CAPS_MASK),
    ('1', KEY_1 as u32), ('!', (KEY_1 as u32) | CAPS_MASK),
    ('2', KEY_2 as u32), ('@', (KEY_2 as u32) | CAPS_MASK),
    ('3', KEY_3 as u32), ('#', (KEY_3 as u32) | CAPS_MASK),
    ('4', KEY_4 as u32), ('$', (KEY_4 as u32) | CAPS_MASK),
    ('5', KEY_5 as u32), ('%', (KEY_5 as u32) | CAPS_MASK),
    ('6', KEY_6 as u32), ('^', (KEY_6 as u32) | CAPS_MASK),
    ('7', KEY_7 as u32), ('&', (KEY_7 as u32) | CAPS_MASK),
    ('8', KEY_8 as u32), ('*', (KEY_8 as u32) | CAPS_MASK),
    ('9', KEY_9 as u32), ('(', (KEY_9 as u32) | CAPS_MASK),
    ('0', KEY_0 as u32), (')', (KEY_0 as u32) | CAPS_MASK),
    ('`', KEY_BACKQUOTE as u32), ('~', (KEY_BACKQUOTE as u32) | CAPS_MASK),
    ('-', KEY_MINUS as u32), ('_', (KEY_MINUS as u32) | CAPS_MASK),
    ('=', KEY_EQUALS as u32), ('+', (KEY_EQUALS as u32) | CAPS_MASK),
    ('[', KEY_LEFT_BRACKET as u32), ('{', (KEY_LEFT_BRACKET as u32) | CAPS_MASK),
    (']', KEY_RIGHT_BRACKET as u32), ('}', (KEY_RIGHT_BRACKET as u32) | CAPS_MASK),
    ('\\', KEY_BACK_SLASH as u32), ('|', (KEY_BACK_SLASH as u32) | CAPS_MASK),
    (';', KEY_SEMICOLON as u32), (':', (KEY_SEMICOLON as u32) | CAPS_MASK),
    ('\'', KEY_QUOTE as u32), ('"', (KEY_QUOTE as u32) | CAPS_MASK),
    (',', KEY_COMMA as u32), ('<', (KEY_COMMA as u32) | CAPS_MASK),
    ('.', KEY_DOT as u32), ('>', (KEY_DOT as u32) | CAPS_MASK),
    ('/', KEY_SLASH as u32), ('?', (KEY_SLASH as u32) | CAPS_MASK),
    (' ', KEY_SPACE as u32),
];

pub fn character_to_key_code(ch: u32) -> Option<u32> {
    for &(c, kc) in CHARACTER_MAP {
        if c as u32 == ch {
            return Some(kc);
        }
    }
    None
}

pub fn key_code_to_character(key_code: u32) -> u16 {
    for &(c, kc) in CHARACTER_MAP {
        if kc == key_code {
            return c as u16;
        }
    }
    0
}

pub fn get_character_code(data: u32, code_table: usize) -> u32 {
    let caps_elem: isize = if (data & CAPS_MASK) != 0 { 0 } else { 1 };
    let mut key: u32 = data & CHAR_MASK;
    if (data & MARK_MASK) != 0 {
        let mut mark_elem: isize = match data & MARK_MASK {
            MARK1_MASK => 0,
            MARK2_MASK => 2,
            MARK3_MASK => 4,
            MARK4_MASK => 6,
            MARK5_MASK => 8,
            _ => -2,
        };
        mark_elem += caps_elem;

        match key as u16 {
            KEY_A | KEY_O | KEY_U | KEY_E => {
                if (data & TONE_MASK) == 0 && (data & TONEW_MASK) == 0 {
                    mark_elem += 4;
                }
            }
            _ => {}
        }

        if (data & TONE_MASK) != 0 {
            key |= TONE_MASK;
        } else if (data & TONEW_MASK) != 0 {
            key |= TONEW_MASK;
        }

        let Some(entry) = code_table_lookup(code_table, key) else {
            return data;
        };
        if mark_elem < 0 || (mark_elem as usize) >= entry.len() {
            return data;
        }
        (entry[mark_elem as usize] as u32) | CHAR_CODE_MASK
    } else {
        let Some(entry) = code_table_lookup(code_table, key) else {
            return data;
        };
        if (data & TONE_MASK) != 0 {
            (entry[caps_elem as usize] as u32) | CHAR_CODE_MASK
        } else if (data & TONEW_MASK) != 0 {
            (entry[(caps_elem + 2) as usize] as u32) | CHAR_CODE_MASK
        } else {
            data
        }
    }
}
