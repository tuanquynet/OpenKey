# OpenKey macOS Rust Port Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port OpenKey for macOS (C++ engine + Objective-C++ `CGEventTap` hook + CLI/app controller) to a pure Rust Cargo workspace with 100% C++ differential test parity.

**Architecture:** A 3-crate Cargo workspace (`crates/openkey-engine`, `crates/openkey-macos`, `crates/openkey-app`). `openkey-engine` encapsulates all Vietnamese typing state, macro tables, smart-switch state, and code-table conversion with zero mutable globals and verifies 100% parity against the original C++ engine compiled via `cc` in `[build-dependencies]` for tests. `openkey-macos` implements `MacEventHandler`, hotkey matching, Spotlight/browser workarounds, `NSUserDefaults`/file config, and `CGEventTap` with automatic tap re-enable. `openkey-app` provides the runnable binary and CLI commands.

**Tech Stack:** Rust 1.97+ (`edition = "2021"`), `cc` (test-only C++ differential harness), macOS `CoreFoundation` & `CoreGraphics` FFI (`CGEventTapCreate`, `CGEventTapEnable`, `CGEventKeyboardSetUnicodeString`).

## Global Constraints

- Rust edition `2021`, zero warnings (`cargo test --workspace`).
- Production builds of `openkey-engine`, `openkey-macos`, and `openkey-app` must be 100% pure Rust (the existing C++ engine in `Sources/OpenKey/engine/` is compiled only for the differential test harness in `openkey-engine`).
- All binary serialization formats (`MacroStore::to_bytes`, `SmartSwitchStore::to_bytes`) must be byte-for-byte identical to OpenKey C++.
- `CGEventTap` callback must automatically re-enable the tap on `kCGEventTapDisabledByTimeout` (`0xFFFFFFFE`) and `kCGEventTapDisabledByUserInput` (`0xFFFFFFFF`).

---

### Task 1: Workspace Scaffold, Types, Key Codes & Vietnamese Tables (`openkey-engine`)

**Files:**
- Create: `Cargo.toml`
- Create: `crates/openkey-engine/Cargo.toml`
- Create: `crates/openkey-engine/src/lib.rs`
- Create: `crates/openkey-engine/src/types.rs`
- Create: `crates/openkey-engine/src/vietnamese.rs`

**Interfaces:**
- Consumes: none
- Produces:
  - `openkey_engine::types::*` (`KEY_*` constants, `CAPS_MASK`, `TONE_MASK`, `TONEW_MASK`, `MARK1_MASK..MARK5_MASK`, `MARK_MASK`, `CHAR_MASK`, `STANDALONE_MASK`, `CHAR_CODE_MASK`, `PURE_CHARACTER_MASK`, `END_CONSONANT_MASK`, `CONSONANT_ALLOW_MASK`, `KeyEvent`, `KeyEventState`, `InputType`, `HookCodeState`, `HookState`, `EngineConfig`)
  - `openkey_engine::vietnamese::*` (`CODE_TABLES`, `UNICODE_COMPOUND_MARK`, `key_code_to_character`, `character_to_key_code`, `get_character_code`, `vowel_table`, `vowel_combine_table`, `vowel_for_mark_table`, `CONSONANT_D`, `CONSONANT_TABLE`, `END_CONSONANT_TABLE`, `STANDALONE_W_BAD`, `DOUBLE_W_ALLOWED`, `QUICK_TELEX`, `QUICK_START_CONSONANT`, `QUICK_END_CONSONANT`)

- [x] **Step 1: Create workspace `Cargo.toml` and `crates/openkey-engine` with failing table & character code unit tests**
- [x] **Step 2: Run `cargo test -p openkey-engine` to verify compilation/test failure before implementation**
- [x] **Step 3: Implement `types.rs` and `vietnamese.rs` with full code tables and character conversion**
- [x] **Step 4: Run `cargo test -p openkey-engine` to verify unit tests pass**
- [x] **Step 5: Commit**

---

### Task 2: SmartSwitchKey, MacroStore, and ConvertTool (`openkey-engine`)

**Files:**
- Create: `crates/openkey-engine/src/smart_switch.rs`
- Create: `crates/openkey-engine/src/macro_store.rs`
- Create: `crates/openkey-engine/src/convert_tool.rs`
- Modify: `crates/openkey-engine/src/lib.rs`

**Interfaces:**
- Consumes: `openkey_engine::types::*`, `openkey_engine::vietnamese::*`
- Produces:
  - `SmartSwitchStore::{new, init_from_bytes, to_bytes, get_app_input_method_status, set_app_input_method_status}`
  - `MacroStore::{new, init_from_bytes, to_bytes, find_macro, has_macro, get_all_macros, add_macro, delete_macro, on_table_code_change, save_to_file, read_from_file}`
  - `ConvertToolConfig`, `convert_util(source: &str, config: &ConvertToolConfig) -> String`

- [x] **Step 1: Write unit tests for `SmartSwitchStore`, `MacroStore` (binary round-trip, file import/export, auto-caps expansion), and `convert_util` (Unicode <-> VNI <-> TCVN3 <-> Unicode Compound, remove mark, casing)**
- [x] **Step 2: Run `cargo test -p openkey-engine` to verify failure**
- [x] **Step 3: Implement `smart_switch.rs`, `macro_store.rs`, and `convert_tool.rs`**
- [x] **Step 4: Run `cargo test -p openkey-engine` to verify all tests pass**
- [x] **Step 5: Commit**

---

### Task 3: Core Vietnamese Typing Engine & C++ Differential Test Harness (`openkey-engine`)

**Files:**
- Create: `crates/openkey-engine/src/engine.rs`
- Create: `crates/openkey-engine/build.rs`
- Create: `crates/openkey-engine/src/cpp_ffi.cpp`
- Create: `crates/openkey-engine/tests/cpp_parity.rs`
- Modify: `crates/openkey-engine/Cargo.toml`
- Modify: `crates/openkey-engine/src/lib.rs`

**Interfaces:**
- Consumes: `types`, `vietnamese`, `macro_store`, `smart_switch`, `convert_tool`
- Produces:
  - `Engine::{new, handle_event, english_mode, start_new_session, temp_off_spell_checking, set_check_spelling, temp_off_engine}`

- [x] **Step 1: Create C++ FFI wrapper (`cpp_ffi.cpp`), `build.rs`, and `tests/cpp_parity.rs` comparing Rust `Engine` against C++ `vKeyHandleEvent` across Telex, VNI, SimpleTelex1/2, all 5 code tables, modern/old orthography, quick telex, quick consonants, restore-if-wrong-spelling, backspace/history restoration, upper-case first char, and macros**
- [x] **Step 2: Implement `crates/openkey-engine/src/engine.rs` ported faithfully from `Sources/OpenKey/engine/Engine.cpp`**
- [x] **Step 3: Run `cargo test -p openkey-engine` and fix any parity discrepancies until 100% of differential tests pass**
- [x] **Step 4: Commit**

---

### Task 4: macOS Event Handler, Workarounds, Config & CGEventTap (`openkey-macos`)

**Files:**
- Create: `crates/openkey-macos/Cargo.toml`
- Create: `crates/openkey-macos/src/lib.rs`
- Create: `crates/openkey-macos/src/config.rs`
- Create: `crates/openkey-macos/src/hotkeys.rs`
- Create: `crates/openkey-macos/src/workaround.rs`
- Create: `crates/openkey-macos/src/sender.rs`
- Create: `crates/openkey-macos/src/handler.rs`
- Create: `crates/openkey-macos/src/event_tap.rs`

**Interfaces:**
- Consumes: `openkey_engine::*`
- Produces:
  - `MacConfig` (with `load_from_defaults`, `save_to_defaults`, JSON/plist serialization)
  - `EventSender` trait, `RecordingEventSender`, `CoreGraphicsEventSender`
  - `MacEventHandler` (handling `kCGEventTapDisabledByTimeout`, `kCGEventTapDisabledByUserInput`, `kCGEventKeyDown`, `kCGEventKeyUp`, `kCGEventFlagsChanged`, mouse events, `_syncKey` double-code backspaces, browser/Spotlight workarounds, 16-char UTF-16 chunking, and active app smart switching)
  - `EventTapController` (`init_event_tap`, `re_enable_event_tap`, `stop_event_tap`, `is_accessibility_trusted`)

- [x] **Step 1: Write comprehensive unit/integration tests for `MacEventHandler` (tap re-enable on timeout/user-input, hotkey language switch, Ctrl/Cmd temp-off, Spotlight selection replacement, Chromium shift-left workaround, VNI double backspace, 16-char Unicode batching, and smart switch key)**
- [x] **Step 2: Implement `config.rs`, `hotkeys.rs`, `workaround.rs`, `sender.rs`, `handler.rs`, and `event_tap.rs`**
- [x] **Step 3: Run `cargo test -p openkey-macos` to verify all tests pass**
- [x] **Step 4: Commit**

---

### Task 5: macOS Application & CLI (`openkey-app`)

**Files:**
- Create: `crates/openkey-app/Cargo.toml`
- Create: `crates/openkey-app/src/main.rs`
- Create: `crates/openkey-app/src/app.rs`

**Interfaces:**
- Consumes: `openkey_engine::*`, `openkey_macos::*`
- Produces:
  - `openkey` binary supporting `run` (macOS event tap daemon), `convert` (code-table text conversion CLI), `macro` (list/add/remove/import/export), `config` (show/set), and `type-sim` (simulate typing a key sequence and print resulting UTF-8 text)

- [x] **Step 1: Write CLI & controller tests in `crates/openkey-app/src/app.rs`**
- [x] **Step 2: Implement `crates/openkey-app/src/app.rs` and `crates/openkey-app/src/main.rs`**
- [x] **Step 3: Run `cargo test --workspace` and smoke-test the `openkey` binary (`convert`, `macro`, `type-sim`, `config`)**
- [x] **Step 4: Commit**
