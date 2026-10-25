#include "Engine.h"
#include <cstring>

int vLanguage = 1;
int vInputType = 0;
int vFreeMark = 0;
int vCodeTable = 0;
int vSwitchKeyStatus = 0x7A000206;
int vCheckSpelling = 1;
int vUseModernOrthography = 0;
int vQuickTelex = 0;
int vRestoreIfWrongSpelling = 1;
int vFixRecommendBrowser = 1;
int vUseMacro = 0;
int vUseMacroInEnglishMode = 0;
int vAutoCapsMacro = 0;
int vUseSmartSwitchKey = 1;
int vUpperCaseFirstChar = 0;
int vTempOffSpelling = 0;
int vAllowConsonantZFWJ = 0;
int vQuickStartConsonant = 0;
int vQuickEndConsonant = 0;
int vRememberCode = 1;
int vOtherLanguage = 1;
int vTempOffOpenKey = 0;

static vKeyHookState* g_hook_state = nullptr;

extern "C" {

struct CppEngineConfig {
    int language;
    int input_type;
    int free_mark;
    int code_table;
    int switch_key_status;
    int check_spelling;
    int use_modern_orthography;
    int quick_telex;
    int restore_if_wrong_spelling;
    int fix_recommend_browser;
    int use_macro;
    int use_macro_in_english_mode;
    int auto_caps_macro;
    int use_smart_switch_key;
    int upper_case_first_char;
    int temp_off_spelling;
    int allow_consonant_zfwj;
    int quick_start_consonant;
    int quick_end_consonant;
    int remember_code;
    int other_language;
    int temp_off_open_key;
};

struct CppHookSnapshot {
    uint8_t code;
    uint8_t backspace_count;
    uint8_t new_char_count;
    uint8_t ext_code;
    uint32_t char_data[MAX_BUFF];
    uint32_t macro_key_len;
    uint32_t macro_key[64];
    uint32_t macro_data_len;
    uint32_t macro_data[128];
};

void cpp_engine_init(const CppEngineConfig* cfg) {
    vLanguage = cfg->language;
    vInputType = cfg->input_type;
    vFreeMark = cfg->free_mark;
    vCodeTable = cfg->code_table;
    vSwitchKeyStatus = cfg->switch_key_status;
    vCheckSpelling = cfg->check_spelling;
    vUseModernOrthography = cfg->use_modern_orthography;
    vQuickTelex = cfg->quick_telex;
    vRestoreIfWrongSpelling = cfg->restore_if_wrong_spelling;
    vFixRecommendBrowser = cfg->fix_recommend_browser;
    vUseMacro = cfg->use_macro;
    vUseMacroInEnglishMode = cfg->use_macro_in_english_mode;
    vAutoCapsMacro = cfg->auto_caps_macro;
    vUseSmartSwitchKey = cfg->use_smart_switch_key;
    vUpperCaseFirstChar = cfg->upper_case_first_char;
    vTempOffSpelling = cfg->temp_off_spelling;
    vAllowConsonantZFWJ = cfg->allow_consonant_zfwj;
    vQuickStartConsonant = cfg->quick_start_consonant;
    vQuickEndConsonant = cfg->quick_end_consonant;
    vRememberCode = cfg->remember_code;
    vOtherLanguage = cfg->other_language;
    vTempOffOpenKey = cfg->temp_off_open_key;

    g_hook_state = static_cast<vKeyHookState*>(vKeyInit());
    g_hook_state->code = 0;
    g_hook_state->backspaceCount = 0;
    g_hook_state->newCharCount = 0;
    g_hook_state->extCode = 0;
    std::memset(g_hook_state->charData, 0, sizeof(g_hook_state->charData));
    g_hook_state->macroKey.clear();
    g_hook_state->macroData.clear();
    startNewSession();
    // Send a MouseDown break to clear internal static state (_spaceCount, _upperCaseStatus, _specialChar, _typingStates)
    vKeyHandleEvent(vKeyEvent::Mouse, vKeyEventState::MouseDown, 0, 0, false);
    g_hook_state->code = 0;
    g_hook_state->backspaceCount = 0;
    g_hook_state->newCharCount = 0;
    g_hook_state->extCode = 0;
    initMacroMap(nullptr, 0);
}

void cpp_engine_add_macro(const char* text, const char* content) {
    addMacro(std::string(text), std::string(content));
}

void cpp_engine_handle_event(int event, int state, uint16_t data, uint8_t caps_status, bool other_control_key, CppHookSnapshot* out) {
    vKeyHandleEvent(
        static_cast<vKeyEvent>(event),
        static_cast<vKeyEventState>(state),
        data,
        caps_status,
        other_control_key
    );
    out->code = g_hook_state->code;
    out->backspace_count = g_hook_state->backspaceCount;
    out->new_char_count = g_hook_state->newCharCount;
    out->ext_code = g_hook_state->extCode;
    std::memcpy(out->char_data, g_hook_state->charData, sizeof(out->char_data));

    out->macro_key_len = static_cast<uint32_t>(std::min<size_t>(g_hook_state->macroKey.size(), 64));
    for (uint32_t i = 0; i < out->macro_key_len; i++) {
        out->macro_key[i] = g_hook_state->macroKey[i];
    }

    out->macro_data_len = static_cast<uint32_t>(std::min<size_t>(g_hook_state->macroData.size(), 128));
    for (uint32_t i = 0; i < out->macro_data_len; i++) {
        out->macro_data[i] = g_hook_state->macroData[i];
    }
}

void cpp_engine_english_mode(int state, uint16_t data, bool is_caps, bool other_control_key, CppHookSnapshot* out) {
    vEnglishMode(
        static_cast<vKeyEventState>(state),
        data,
        is_caps,
        other_control_key
    );
    out->code = g_hook_state->code;
    out->backspace_count = g_hook_state->backspaceCount;
    out->new_char_count = g_hook_state->newCharCount;
    out->ext_code = g_hook_state->extCode;
    std::memcpy(out->char_data, g_hook_state->charData, sizeof(out->char_data));

    out->macro_key_len = static_cast<uint32_t>(std::min<size_t>(g_hook_state->macroKey.size(), 64));
    for (uint32_t i = 0; i < out->macro_key_len; i++) {
        out->macro_key[i] = g_hook_state->macroKey[i];
    }

    out->macro_data_len = static_cast<uint32_t>(std::min<size_t>(g_hook_state->macroData.size(), 128));
    for (uint32_t i = 0; i < out->macro_data_len; i++) {
        out->macro_data[i] = g_hook_state->macroData[i];
    }
}

void cpp_engine_start_new_session() {
    startNewSession();
}

uint32_t cpp_get_macro_save_data(uint8_t* out_buf, uint32_t max_len) {
    std::vector<Byte> data;
    getMacroSaveData(data);
    uint32_t n = static_cast<uint32_t>(std::min<size_t>(data.size(), max_len));
    if (n > 0) {
        std::memcpy(out_buf, data.data(), n);
    }
    return n;
}

void cpp_init_smart_switch_key() {
    initSmartSwitchKey(nullptr, 0);
}

int cpp_get_app_input_method_status(const char* bundle_id, int current_input_method) {
    return getAppInputMethodStatus(std::string(bundle_id), current_input_method);
}

void cpp_set_app_input_method_status(const char* bundle_id, int language) {
    setAppInputMethodStatus(std::string(bundle_id), language);
}

uint32_t cpp_get_smart_switch_save_data(uint8_t* out_buf, uint32_t max_len) {
    std::vector<Byte> data;
    getSmartSwitchKeySaveData(data);
    uint32_t n = static_cast<uint32_t>(std::min<size_t>(data.size(), max_len));
    if (n > 0) {
        std::memcpy(out_buf, data.data(), n);
    }
    return n;
}

uint32_t cpp_convert_util(
    const char* src,
    uint8_t from_code,
    uint8_t to_code,
    bool all_caps,
    bool non_caps,
    bool caps_first,
    bool caps_each_word,
    bool remove_mark,
    uint8_t* out_buf,
    uint32_t max_len
) {
    convertToolFromCode = from_code;
    convertToolToCode = to_code;
    convertToolToAllCaps = all_caps;
    convertToolToAllNonCaps = non_caps;
    convertToolToCapsFirstLetter = caps_first;
    convertToolToCapsEachWord = caps_each_word;
    convertToolRemoveMark = remove_mark;
    std::string res = convertUtil(std::string(src));
    uint32_t n = static_cast<uint32_t>(std::min<size_t>(res.size(), max_len));
    if (n > 0) {
        std::memcpy(out_buf, res.data(), n);
    }
    return n;
}

}
