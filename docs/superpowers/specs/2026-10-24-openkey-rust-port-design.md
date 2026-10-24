# OpenKey macOS Rust Port Design Spec

**Date:** 2026-10-24
**Scope:** Full Rust port of OpenKey for macOS (`openkey-engine`, `openkey-macos`, `openkey-app`)

---

## 1. Overview & Goals

Port OpenKey for macOS from C++ (`Sources/OpenKey/engine/`) and Objective-C/Objective-C++ (`Sources/OpenKey/macOS/`) to a pure Rust Cargo workspace at the repository root while preserving:

1. **100% Behavioral Parity:** Exact Vietnamese typing behavior across all 4 input types (`Telex`, `VNI`, `SimpleTelex1`, `SimpleTelex2`) and all 5 character encodings (`Unicode`, `TCVN3 (ABC)`, `VNI Windows`, `Unicode Compound`, `Vietnamese Locale CP 1258`), including spelling check, grammar check, modern/old orthography, quick telex, quick start/end consonant, restore-if-wrong-spelling, upper-case first character, macro expansion with auto-caps, smart switch key, and convert tool.
2. **100% Data Format Compatibility:** Byte-for-byte compatibility with OpenKey's binary persistence formats (`macroData` and `smartSwitchKey` blobs in `NSUserDefaults`), UniKey/OpenKey `.txt` macro import/export files, and `com.tuyenmai.openkey` preference keys.
3. **Owned, Thread-Safe State:** Eliminate the C++ engine's mutable globals (`vLanguage`, `TypingWord`, `_index`, `HookState`, `macroMap`, `_smartSwitchKeyData`) in favor of explicit, owned Rust structs (`Engine`, `EngineConfig`, `HookState`, `MacroStore`, `SmartSwitchStore`, `ConvertToolConfig`).
4. **Native macOS Integration:** Preserve `CGEventTap` keyboard/mouse interception (including automatic tap re-enable on `kCGEventTapDisabledByTimeout` and `kCGEventTapDisabledByUserInput`), backspace/Unicode event synthesis, Spotlight/browser autocomplete workarounds, `NSUserDefaults` configuration persistence, `AXIsProcessTrusted` check, and macOS status bar / CLI / control entrypoint.

---

## 2. Workspace Architecture & Decomposition

The repository hosts a root Cargo workspace (`Cargo.toml`) divided into three crates corresponding to the three sub-projects:

```
OpenKey/
├── Cargo.toml                  # Workspace root
├── crates/
│   ├── openkey-engine/         # Sub-project 1: Pure Rust Vietnamese engine (zero OS runtime deps)
│   │   ├── Cargo.toml
│   │   ├── build.rs            # Compiles original C++ engine ONLY for dev/test differential harness
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types.rs        # Key codes, bitmasks, HookCodeState, HookState, EngineConfig
│   │   │   ├── vietnamese.rs   # Vowel/consonant tables, code tables, character maps
│   │   │   ├── macro_store.rs  # MacroData, binary init/save, text file read/save, auto-caps lookup
│   │   │   ├── smart_switch.rs # Per-app input method & code table persistence
│   │   │   ├── convert_tool.rs # Code-table text conversion & casing transformations
│   │   │   └── engine.rs       # Core state machine (vKeyHandleEvent, vEnglishMode, checkSpelling, etc.)
│   │   └── tests/
│   │       └── cpp_parity.rs   # Differential test suite: Rust Engine vs C++ Engine via FFI
│   ├── openkey-macos/          # Sub-project 2: macOS system integration & CGEventTap runtime
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs       # AppConfig & NSUserDefaults / plist serialization
│   │       ├── hotkeys.rs      # Switch key & convert tool hotkey matching logic
│   │       ├── workaround.rs   # App-specific workarounds (Spotlight, Chromium, Sublime, Compound)
│   │       ├── sender.rs       # EventSender trait + CoreGraphics CGEvent keyboard/backspace synthesis
│   │       ├── handler.rs      # Platform-independent callback coordinator driving Engine + EventSender
│   │       └── event_tap.rs    # CGEventTap lifecycle, auto re-enable on timeout/user-input, accessibility
│   └── openkey-app/            # Sub-project 3: macOS application binary & UI/status-bar controller
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs         # Binary entrypoint (CLI + macOS runloop / status bar controller)
│           └── app.rs          # Application controller, status bar state, quick convert, config commands
```

---

## 3. Component Design

### 3.1 `openkey-engine` (Sub-project 1)

#### Data Types & Bitmasks (`types.rs`)
- Preserves OpenKey's exact 32-bit internal character word format:
  - Bits `0..=15` (`CHAR_MASK = 0xFFFF`): keycode or 16-bit code-table unit
  - Bit `16` (`CAPS_MASK = 0x10000`): uppercase flag
  - Bit `17` (`TONE_MASK = 0x20000`): circumflex (`^`) or `đ` flag
  - Bit `18` (`TONEW_MASK = 0x40000`): horn/breve (`ư`, `ơ`, `ă`) flag
  - Bits `19..=23` (`MARK1_MASK..MARK5_MASK`, `MARK_MASK = 0xF80000`): tone marks (Sắc, Huyền, Hỏi, Ngã, Nặng)
  - Bit `24` (`STANDALONE_MASK = 0x1000000`): standalone key (`w`, `[`, `]`)
  - Bit `25` (`CHAR_CODE_MASK = 0x2000000`): resolved code-table character vs raw keycode
  - Bit `31` (`PURE_CHARACTER_MASK = 0x80000000`): literal UTF-16/UCS-2 character
- `KeyEvent { Keyboard, Mouse }`, `KeyEventState { KeyDown, KeyUp, MouseDown, MouseUp }`, `InputType { Telex = 0, Vni = 1, SimpleTelex1 = 2, SimpleTelex2 = 3 }`, `HookCodeState { DoNothing = 0, WillProcess = 1, BreakWord = 2, Restore = 3, ReplaceMacro = 4, RestoreAndStartNewSession = 5 }`.
- `HookState`:
  - `code: u8`
  - `backspace_count: u8`
  - `new_char_count: u8`
  - `ext_code: u8` (`1` = word break, `2` = delete key, `3` = normal key, `4` = should not send empty char)
  - `char_data: [u32; MAX_BUFF]` (`MAX_BUFF = 32`)
  - `macro_key: Vec<u32>`
  - `macro_data: Vec<u32>`

#### Tables (`vietnamese.rs`)
- Exact static/lazy tables ported from `Vietnamese.cpp`:
  - `vowel_table(key)`, `vowel_combine_table(key)`, `vowel_for_mark_table(key)`
  - `CONSONANT_D`, `CONSONANT_TABLE`, `END_CONSONANT_TABLE`, `STANDALONE_W_BAD`, `DOUBLE_W_ALLOWED`
  - `QUICK_TELEX`, `QUICK_START_CONSONANT`, `QUICK_END_CONSONANT`
  - `CODE_TABLES[5]` (`Unicode`, `TCVN3`, `VNI Windows`, `Unicode Compound`, `CP 1258`) and `UNICODE_COMPOUND_MARK`
  - `character_map()` and `key_code_to_character(u32) -> u16`

#### Macro Store (`macro_store.rs`)
- `MacroStore` holding `BTreeMap<Vec<u32>, MacroData>` (ordered lexicographically just like `std::map<vector<Uint32>, MacroData>` in C++ so binary/file serialization order is identical).
- Binary serialization:
  - `init_from_bytes(bytes: &[u8], code_table: usize)`
  - `to_bytes(&self) -> Vec<u8>`
  - Format: `u16` little-endian `macro_count`, followed by `[u8 text_len][text bytes][u16 LE content_len][content bytes]` per entry.
- Text file serialization:
  - `save_to_file(path)` / `read_from_file(path, append, code_table)` with header `;Compatible OpenKey Macro Data file for UniKey*** version=1 ***`.
- Lookup:
  - `find_macro(&self, key: &mut Vec<u32>, code_table: usize, auto_caps: bool) -> Option<Vec<u32>>` implementing exact `modifyCaseUnicode` logic.

#### Smart Switch Key (`smart_switch.rs`)
- `SmartSwitchStore` holding `BTreeMap<String, i8>` plus single-entry cache (`cache_key`, `cache_data`) matching `SmartSwitchKey.cpp`.
- Binary serialization: `u16` LE count, followed by `[u8 len][bundle_id bytes][i8 value]`.
- `get_app_input_method_status(&mut self, bundle_id: &str, current_input_method: i32) -> i32` and `set_app_input_method_status(&mut self, bundle_id: &str, language: i32)`.

#### Convert Tool (`convert_tool.rs`)
- `ConvertToolConfig` struct (`dont_alert_when_completed`, `to_all_caps`, `to_all_non_caps`, `to_caps_first_letter`, `to_caps_each_word`, `remove_mark`, `from_code`, `to_code`, `hot_key`).
- `convert_util(source: &str, config: &ConvertToolConfig) -> String` operating on UTF-16 code units (`encode_utf16` / `String::from_utf16_lossy`) to match macOS/C++ `utf8ToWideString` (`std::codecvt_utf8_utf16`).

#### Core Engine (`engine.rs`)
- `Engine` struct containing:
  - `pub config: EngineConfig`
  - `pub hook_state: HookState`
  - `pub macros: MacroStore`
  - `pub smart_switch: SmartSwitchStore`
  - `pub convert_tool: ConvertToolConfig`
  - Internal typing buffers (`typing_word: [u32; 32]`, `index: u8`, `long_word_helper: Vec<u32>`, `typing_states: Vec<Vec<u32>>`, `key_states: [u32; 32]`, `state_index: u8`, `special_char: Vec<u32>`, `space_count: usize`, `upper_case_status: u8`, `temp_disable_key: bool`, `will_temp_off_engine: bool`, etc.).
- Public methods mirroring `Engine.h`:
  - `Engine::new() -> Self`
  - `handle_event(&mut self, event: KeyEvent, state: KeyEventState, data: u16, caps_status: u8, other_control_key: bool)`
  - `english_mode(&mut self, state: KeyEventState, data: u16, is_caps: bool, other_control_key: bool)`
  - `start_new_session(&mut self)`
  - `temp_off_spell_checking(&mut self)`
  - `set_check_spelling(&mut self)`
  - `temp_off_engine(&mut self, off: bool)`

---

### 3.2 `openkey-macos` (Sub-project 2)

Separates testable macOS event orchestration from raw CoreGraphics calls via a trait boundary:

1. **`EventSender` trait (`sender.rs`)**:
   - `send_backspace(&mut self)`
   - `send_shift_left_arrow(&mut self)`
   - `send_empty_character(&mut self, ch: u16)`
   - `send_unicode_string(&mut self, chars: &[u16])`
   - `send_key_code(&mut self, keycode: u16, shift: bool)`
   - `beep(&mut self)`
   - `trigger_quick_convert(&mut self)`
   - `notify_input_method_changed(&mut self, language: i32, code_table: i32)`
   - Implementations: `CoreGraphicsEventSender` (production `CGEventCreateKeyboardEvent` / `CGEventKeyboardSetUnicodeString` / `CGEventTapPostEvent`) and `RecordingEventSender` (for deterministic tests of `OpenKeyCallback` behavior).
2. **`MacEventHandler` (`handler.rs`)**:
   - Owns `Engine`, `MacRuntimeConfig` (`send_key_step_by_step`, `fix_chromium_browser`, `perform_layout_compat`), `sync_key: Vec<u8>`, `last_flag: u64`, `has_just_used_hotkey: bool`, `frontmost_app: String`.
   - Processes key-down, key-up, flags-changed, mouse-down/dragged, and tap-disabled events identically to `OpenKey.mm`:
     - Re-enables tap on `kCGEventTapDisabledByTimeout` / `kCGEventTapDisabledByUserInput`
     - Handles switch-language and quick-convert hotkeys (both modifier+key and modifier-only release)
     - Handles Ctrl release (`vTempOffSpelling`) and Cmd release (`vTempOffOpenKey`)
     - Handles double-code (`VNI Windows`, `Unicode Compound`) `_syncKey` backspace tracking
     - Handles browser autocomplete (`0x202F` / `0x200C` empty char or `Shift+Left` for Chromium) and Spotlight selection replacement (`Shift+Left`)
     - Chunks Unicode strings into 16-UTF16-unit batches (`SendNewCharString`) and handles `vRestore` / `vRestoreAndStartNewSession` trailing control keys.
3. **`event_tap.rs` & `config.rs`**:
   - CoreFoundation / CoreGraphics FFI bindings for `CGEventTapCreate`, `CGEventTapEnable`, `CFMachPortCreateRunLoopSource`, `CFRunLoopAddSource`, and `AXIsProcessTrustedWithOptions`.
   - Configuration load/save compatible with `com.tuyenmai.openkey` defaults and JSON/plist file fallback.

---

### 3.3 `openkey-app` (Sub-project 3)

Provides the runnable macOS binary (`openkey`) with:
- Daemon mode (`openkey run`): checks Accessibility permissions, loads config/macros/smart-switch data, starts the `CGEventTap` on the macOS `CFRunLoop`, and manages the status bar / hotkey lifecycle.
- CLI utilities for scripting and inspection:
  - `openkey convert --from <table_id> --to <table_id> [--remove-mark] [--all-caps] <text>`
  - `openkey config` (show/update settings)
  - `openkey macro` (list/add/remove/import/export UniKey-compatible macro files)

---

## 4. Error Handling & Edge Cases

- **Event Tap Disabled by macOS:** When `CGEventType` is `kCGEventTapDisabledByTimeout` (`0xFFFFFFFE`) or `kCGEventTapDisabledByUserInput` (`0xFFFFFFFF`), `MacEventHandler` invokes `re_enable_event_tap()` and returns the event unmodified.
- **Corrupted/Truncated Binary Blobs:** `MacroStore::init_from_bytes` and `SmartSwitchStore::init_from_bytes` bounds-check every cursor read against `bytes.len()` so truncated `NSUserDefaults` blobs never panic or read out-of-bounds.
- **Long Words (`>= 32` characters):** `Engine` shifts `typing_word` into `long_word_helper` on overflow and restores on `KEY_DELETE`, matching `Engine.cpp`.
- **UTF-16 Batching (`> 16` UTF-16 code units):** `MacEventHandler` splits `CGEventKeyboardSetUnicodeString` payloads into 16-unit chunks, matching `SendNewCharString` in `OpenKey.mm`.

---

## 5. Verification Strategy

1. **C++ Differential Test Suite (`crates/openkey-engine/tests/cpp_parity.rs`):**
   - `build.rs` compiles `Sources/OpenKey/engine/{Engine,Vietnamese,Macro,SmartSwitchKey,ConvertTool}.cpp` into a static library linked only during `cargo test` (`[dev-dependencies]`).
   - Side-by-side harness feeds identical keystroke streams (`KeyEvent`, `KeyEventState`, `keyCode`, `capsStatus`, `otherControlKey`) to both C++ and Rust engines under all combinations of input types (`Telex`, `VNI`, `SimpleTelex1`, `SimpleTelex2`), code tables (`0..=4`), orthography (`old` vs `modern`), quick telex, quick consonants, spell check, restore-if-wrong-spelling, and macros, asserting identical `HookState` (`code`, `backspace_count`, `new_char_count`, `ext_code`, `char_data`, `macro_data`).
2. **Unit & Integration Tests:**
   - Binary round-trip tests for `MacroStore` and `SmartSwitchStore` against C++ byte output.
   - `ConvertTool` parity tests across Unicode, VNI Windows, TCVN3, Unicode Compound, and CP 1258.
   - `openkey-macos` handler tests using `RecordingEventSender` to verify event tap re-enable, hotkey switching, Spotlight/Chromium workarounds, and 16-char Unicode batching.
   - `openkey-app` CLI smoke tests (`openkey convert`, `openkey macro`, `openkey config`).
