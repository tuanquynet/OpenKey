use std::ffi::c_void;
use std::io::Write;
use std::process::{Command, Stdio};
use std::ptr;

use openkey_engine::convert_tool::{convert_util, ConvertToolConfig};
use openkey_engine::types::KEY_DELETE;

use crate::config::convert_key_string_to_key_code;
use crate::handler::{CallbackAction, MacEventHandler, MacEventType};
use crate::hotkeys::{CG_FLAG_NON_COALESCED, CG_FLAG_SHIFT};
use crate::sender::EventSender;

// macOS CoreGraphics / CoreFoundation / ApplicationServices constants
pub const K_CG_SESSION_EVENT_TAP: u32 = 1;
pub const K_CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
pub const K_CG_EVENT_TAP_OPTION_DEFAULT: u32 = 0;

pub const K_CG_EVENT_LEFT_MOUSE_DOWN: u32 = 1;
pub const K_CG_EVENT_RIGHT_MOUSE_DOWN: u32 = 3;
pub const K_CG_EVENT_LEFT_MOUSE_DRAGGED: u32 = 6;
pub const K_CG_EVENT_RIGHT_MOUSE_DRAGGED: u32 = 7;
pub const K_CG_EVENT_KEY_DOWN: u32 = 10;
pub const K_CG_EVENT_KEY_UP: u32 = 11;
pub const K_CG_EVENT_FLAGS_CHANGED: u32 = 12;
pub const K_CG_EVENT_TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFF_FFFE;
pub const K_CG_EVENT_TAP_DISABLED_BY_USER_INPUT: u32 = 0xFFFF_FFFF;

pub const K_CG_KEYBOARD_EVENT_KEYCODE: u32 = 9;
pub const K_CG_EVENT_SOURCE_STATE_ID: u32 = 45;
pub const K_CG_EVENT_SOURCE_STATE_PRIVATE: i32 = -1;

pub const K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: u32 = 1 << 0;
pub const K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS: u32 = 1 << 4;
pub const K_CG_NULL_WINDOW_ID: u32 = 0;

pub type CFTypeRef = *const c_void;
pub type CFArrayRef = *const c_void;
pub type CFDictionaryRef = *const c_void;
pub type CFMachPortRef = *mut c_void;
pub type CFRunLoopRef = *mut c_void;
pub type CFRunLoopSourceRef = *mut c_void;
pub type CFStringRef = *const c_void;
pub type CGEventRef = *mut c_void;
pub type CGEventTapProxy = *mut c_void;
pub type CGEventSourceRef = *mut c_void;
pub type CGEventMask = u64;
pub type TISInputSourceRef = *mut c_void;

pub type CGEventTapCallBack = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef;

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
#[link(name = "CoreGraphics", kind = "framework")]
#[link(name = "ApplicationServices", kind = "framework")]
#[link(name = "Carbon", kind = "framework")]
#[link(name = "AudioToolbox", kind = "framework")]
#[link(name = "AppKit", kind = "framework")]
#[link(name = "objc")]
extern "C" {
    static kCFRunLoopCommonModes: CFStringRef;
    static kCGWindowOwnerName: CFStringRef;
    static kTISPropertyInputSourceLanguages: CFStringRef;
    static kAXTrustedCheckOptionPrompt: CFStringRef;
    static kCFBooleanTrue: CFTypeRef;

    fn CFRelease(cf: CFTypeRef);
    fn CFDictionaryCreate(
        allocator: CFTypeRef,
        keys: *const *const c_void,
        values: *const *const c_void,
        num_values: isize,
        key_call_backs: *const c_void,
        value_call_backs: *const c_void,
    ) -> CFDictionaryRef;
    fn CFAbsoluteTimeGetCurrent() -> f64;
    fn CFRunLoopGetMain() -> CFRunLoopRef;
    fn CFRunLoopTimerCreate(
        allocator: CFTypeRef,
        fire_date: f64,
        interval: f64,
        flags: usize,
        order: isize,
        callout: unsafe extern "C" fn(*mut c_void, *mut c_void),
        context: *mut CFRunLoopTimerContext,
    ) -> *mut c_void;
    fn CFRunLoopAddTimer(rl: CFRunLoopRef, timer: *mut c_void, mode: CFStringRef);
    fn CFRunLoopTimerInvalidate(timer: *mut c_void);
    fn CFArrayGetCount(the_array: CFArrayRef) -> isize;
    fn CFArrayGetValueAtIndex(the_array: CFArrayRef, idx: isize) -> *const c_void;
    fn CFDictionaryGetValue(the_dict: CFDictionaryRef, key: *const c_void) -> *const c_void;
    fn CFStringCreateWithBytes(
        alloc: *const c_void,
        bytes: *const u8,
        num_bytes: isize,
        encoding: u32,
        is_external_representation: u8,
    ) -> CFStringRef;
    fn CFStringCompare(
        the_string1: CFStringRef,
        the_string2: CFStringRef,
        compare_options: usize,
    ) -> isize;
    fn CFStringHasPrefix(the_string: CFStringRef, prefix: CFStringRef) -> u8;

    fn CFMachPortInvalidate(port: CFMachPortRef);
    fn CFMachPortCreateRunLoopSource(
        allocator: CFTypeRef,
        port: CFMachPortRef,
        order: isize,
    ) -> CFRunLoopSourceRef;
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
    fn CFRunLoopRemoveSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
    fn CFRunLoopRun();

    fn CGWindowListCopyWindowInfo(option: u32, relative_to_window: u32) -> CFArrayRef;

    fn CGEventSourceCreate(state_id: i32) -> CGEventSourceRef;
    fn CGEventSourceGetSourceStateID(source: CGEventSourceRef) -> i32;
    fn CGEventCreateKeyboardEvent(
        source: CGEventSourceRef,
        virtual_key: u16,
        key_down: bool,
    ) -> CGEventRef;
    fn CGEventKeyboardSetUnicodeString(
        event: CGEventRef,
        string_length: usize,
        unicode_string: *const u16,
    );
    fn CGEventKeyboardGetUnicodeString(
        event: CGEventRef,
        max_string_length: usize,
        actual_string_length: *mut usize,
        unicode_string: *mut u16,
    );
    fn CGEventGetFlags(event: CGEventRef) -> u64;
    fn CGEventSetFlags(event: CGEventRef, flags: u64);
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    fn CGEventTapPostEvent(proxy: CGEventTapProxy, event: CGEventRef);
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: CGEventMask,
        callback: CGEventTapCallBack,
        user_info: *mut c_void,
    ) -> CFMachPortRef;
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);

    fn TISCopyCurrentKeyboardInputSource() -> TISInputSourceRef;
    fn TISGetInputSourceProperty(
        input_source: TISInputSourceRef,
        property_key: CFStringRef,
    ) -> *const c_void;

    fn AudioServicesPlayAlertSound(in_system_sound_id: u32);
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
    fn objc_getClass(name: *const std::ffi::c_char) -> *mut c_void;
    fn sel_registerName(name: *const std::ffi::c_char) -> *mut c_void;
    fn objc_msgSend();
    fn getuid() -> u32;
    fn proc_pidinfo(
        pid: i32,
        flavor: i32,
        arg: u64,
        buffer: *mut c_void,
        buffersize: i32,
    ) -> i32;
}

#[cfg(target_os = "macos")]
const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
#[cfg(target_os = "macos")]
const K_SYSTEM_SOUND_ID_USER_PREFERRED_ALERT: u32 = 0x0000_1000;

pub fn is_accessibility_trusted() -> bool {
    #[cfg(target_os = "macos")]
    unsafe {
        AXIsProcessTrusted()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

#[repr(C)]
pub struct CFRunLoopTimerContext {
    pub version: isize,
    pub info: *mut c_void,
    pub retain: *const c_void,
    pub release: *const c_void,
    pub copy_description: *const c_void,
}

/// Prompts macOS to display the native Accessibility permission dialog (`AXIsProcessTrustedWithOptions` with `kAXTrustedCheckOptionPrompt: kCFBooleanTrue`).
pub fn prompt_accessibility_permission() -> bool {
    #[cfg(target_os = "macos")]
    unsafe {
        let keys = [kAXTrustedCheckOptionPrompt as *const c_void];
        let values = [kCFBooleanTrue];
        let dict = CFDictionaryCreate(
            ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            ptr::null(),
            ptr::null(),
        );
        let trusted = AXIsProcessTrustedWithOptions(dict);
        if !dict.is_null() {
            CFRelease(dict);
        }
        trusted
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// Queries `[[NSWorkspace sharedWorkspace] frontmostApplication].bundleIdentifier`, matching `queryFrontMostApp()` in `OpenKey.mm`.
pub fn frontmost_application_bundle_id() -> String {
    #[cfg(target_os = "macos")]
    unsafe {
        let msg0: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
            std::mem::transmute(objc_msgSend as *const ());
        let utf8: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *const std::ffi::c_char =
            std::mem::transmute(objc_msgSend as *const ());

        let ws_cls = objc_getClass(b"NSWorkspace\0".as_ptr() as *const std::ffi::c_char);
        if ws_cls.is_null() {
            return "UnknownApp".to_string();
        }
        let shared_sel = sel_registerName(b"sharedWorkspace\0".as_ptr() as *const std::ffi::c_char);
        let ws = msg0(ws_cls, shared_sel);
        if ws.is_null() {
            return "UnknownApp".to_string();
        }
        let front_sel =
            sel_registerName(b"frontmostApplication\0".as_ptr() as *const std::ffi::c_char);
        let app = msg0(ws, front_sel);
        if app.is_null() {
            return "UnknownApp".to_string();
        }
        let bundle_sel =
            sel_registerName(b"bundleIdentifier\0".as_ptr() as *const std::ffi::c_char);
        let mut ns_str = msg0(app, bundle_sel);
        if ns_str.is_null() {
            let name_sel =
                sel_registerName(b"localizedName\0".as_ptr() as *const std::ffi::c_char);
            ns_str = msg0(app, name_sel);
        }
        if ns_str.is_null() {
            return "UnknownApp".to_string();
        }
        let utf8_sel = sel_registerName(b"UTF8String\0".as_ptr() as *const std::ffi::c_char);
        let c_ptr = utf8(ns_str, utf8_sel);
        if c_ptr.is_null() {
            "UnknownApp".to_string()
        } else {
            let s = std::ffi::CStr::from_ptr(c_ptr)
                .to_string_lossy()
                .into_owned();
            if s.is_empty() {
                "UnknownApp".to_string()
            } else {
                s
            }
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        "UnknownApp".to_string()
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ProcBsdInfo {
    pbi_flags: u32,
    pbi_status: u32,
    pbi_xstatus: u32,
    pbi_pid: u32,
    pbi_ppid: u32,
    pbi_uid: u32,
    pbi_gid: u32,
    pbi_ruid: u32,
    pbi_rgid: u32,
    pbi_svuid: u32,
    pbi_svgid: u32,
    rfu_1: u32,
    pbi_comm: [u8; 16],
    pbi_name: [u8; 32],
    pbi_nfiles: u32,
    pbi_pgid: u32,
    pbi_pjobc: u32,
    e_tdev: u32,
    e_tpgid: u32,
    pbi_nice: i32,
    pbi_start_tvsec: u64,
    pbi_start_tvusec: u64,
}

const PROC_PIDTBSDINFO: i32 = 3;

/// Returns the UID owning `pid` via `proc_pidinfo(pid, PROC_PIDTBSDINFO, ...)`.
pub fn get_process_uid(pid: i32) -> Option<u32> {
    #[cfg(target_os = "macos")]
    unsafe {
        let mut info: ProcBsdInfo = std::mem::zeroed();
        let expected = std::mem::size_of::<ProcBsdInfo>() as i32;
        let ret = proc_pidinfo(
            pid,
            PROC_PIDTBSDINFO,
            0,
            (&mut info as *mut ProcBsdInfo) as *mut c_void,
            expected,
        );
        if ret == expected {
            Some(info.pbi_uid)
        } else {
            None
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = pid;
        None
    }
}

/// Checks whether `pid` belongs to the current macOS user (`getuid()`), supporting Fast User Switching (`AppDelegate.m`).
pub fn is_process_owned_by_current_user(pid: i32) -> bool {
    #[cfg(target_os = "macos")]
    unsafe {
        let current_uid = getuid();
        get_process_uid(pid) == Some(current_uid)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = pid;
        false
    }
}

/// Checks whether the macOS Spotlight window is currently visible on screen, matching `isSpotlightVisible()` in `OpenKey.mm`.
pub fn is_spotlight_visible() -> bool {
    #[cfg(target_os = "macos")]
    unsafe {
        let windows = CGWindowListCopyWindowInfo(
            K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY | K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS,
            K_CG_NULL_WINDOW_ID,
        );
        if windows.is_null() {
            return false;
        }
        let target = CFStringCreateWithBytes(
            ptr::null(),
            b"Spotlight".as_ptr(),
            9,
            K_CF_STRING_ENCODING_UTF8,
            0,
        );
        let count = CFArrayGetCount(windows);
        let mut found = false;
        for i in 0..count {
            let dict = CFArrayGetValueAtIndex(windows, i);
            if !dict.is_null() {
                let owner = CFDictionaryGetValue(dict, kCGWindowOwnerName);
                if !owner.is_null() && CFStringCompare(owner, target, 0) == 0 {
                    found = true;
                    break;
                }
            }
        }
        CFRelease(target);
        CFRelease(windows);
        found
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// Checks whether the current macOS keyboard input source primary language is English (`"en"`), matching `vOtherLanguage` check in `OpenKey.mm`.
pub fn is_current_input_source_english() -> bool {
    #[cfg(target_os = "macos")]
    unsafe {
        let isource = TISCopyCurrentKeyboardInputSource();
        if isource.is_null() {
            return true;
        }
        let languages = TISGetInputSourceProperty(isource, kTISPropertyInputSourceLanguages);
        let mut is_en = true;
        if !languages.is_null() && CFArrayGetCount(languages) > 0 {
            let lang_ref = CFArrayGetValueAtIndex(languages, 0);
            if !lang_ref.is_null() {
                let prefix = CFStringCreateWithBytes(
                    ptr::null(),
                    b"en".as_ptr(),
                    2,
                    K_CF_STRING_ENCODING_UTF8,
                    0,
                );
                is_en = CFStringHasPrefix(lang_ref, prefix) != 0;
                CFRelease(prefix);
            }
        }
        CFRelease(isource);
        is_en
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// Converts a `CGEventRef` key event to its layout-compatible virtual keycode when `vPerformLayoutCompat` is enabled.
pub fn convert_cg_event_to_layout_compat_keycode(event: CGEventRef, fallback: u16) -> u16 {
    #[cfg(target_os = "macos")]
    unsafe {
        if event.is_null() {
            return fallback;
        }
        let mut actual_len: usize = 0;
        let mut buf: [u16; 4] = [0; 4];
        CGEventKeyboardGetUnicodeString(event, buf.len(), &mut actual_len, buf.as_mut_ptr());
        if actual_len == 0 {
            return fallback;
        }
        let s = String::from_utf16_lossy(&buf[..actual_len]);
        convert_key_string_to_key_code(&s, fallback)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = event;
        fallback
    }
}

pub fn play_system_beep() {
    #[cfg(target_os = "macos")]
    unsafe {
        AudioServicesPlayAlertSound(K_SYSTEM_SOUND_ID_USER_PREFERRED_ALERT);
    }
}

/// Converts text on the macOS general pasteboard using `convert_util`, matching `[OpenKeyManager quickConvert]`.
pub fn quick_convert_pasteboard(config: &ConvertToolConfig) -> bool {
    let Ok(out) = Command::new("pbpaste").output() else {
        return false;
    };
    if !out.status.success() || out.stdout.is_empty() {
        return false;
    }
    let raw = String::from_utf8_lossy(&out.stdout);
    if raw.is_empty() {
        return false;
    }
    let converted = convert_util(&raw, config);
    let Ok(mut child) = Command::new("pbcopy").stdin(Stdio::piped()).spawn() else {
        return false;
    };
    if let Some(mut stdin) = child.stdin.take() {
        if stdin.write_all(converted.as_bytes()).is_err() {
            return false;
        }
    }
    child.wait().map(|s| s.success()).unwrap_or(false)
}

pub fn map_cg_event_type(raw_type: u32) -> MacEventType {
    match raw_type {
        K_CG_EVENT_TAP_DISABLED_BY_TIMEOUT => MacEventType::TapDisabledByTimeout,
        K_CG_EVENT_TAP_DISABLED_BY_USER_INPUT => MacEventType::TapDisabledByUserInput,
        K_CG_EVENT_KEY_DOWN => MacEventType::KeyDown,
        K_CG_EVENT_KEY_UP => MacEventType::KeyUp,
        K_CG_EVENT_FLAGS_CHANGED => MacEventType::FlagsChanged,
        K_CG_EVENT_LEFT_MOUSE_DOWN => MacEventType::LeftMouseDown,
        K_CG_EVENT_RIGHT_MOUSE_DOWN => MacEventType::RightMouseDown,
        K_CG_EVENT_LEFT_MOUSE_DRAGGED => MacEventType::LeftMouseDragged,
        K_CG_EVENT_RIGHT_MOUSE_DRAGGED => MacEventType::RightMouseDragged,
        other => MacEventType::Other(other),
    }
}

pub struct CoreGraphicsEventSender {
    pub event_tap: CFMachPortRef,
    pub proxy: CGEventTapProxy,
    pub event_source: CGEventSourceRef,
    pub backspace_down: CGEventRef,
    pub backspace_up: CGEventRef,
    pub convert_tool_config: ConvertToolConfig,
    pub on_status_changed: Option<fn(i32, i32)>,
}

impl CoreGraphicsEventSender {
    #[cfg(target_os = "macos")]
    pub unsafe fn new() -> Self {
        let event_source = CGEventSourceCreate(K_CG_EVENT_SOURCE_STATE_PRIVATE);
        let backspace_down = CGEventCreateKeyboardEvent(event_source, KEY_DELETE, true);
        let backspace_up = CGEventCreateKeyboardEvent(event_source, KEY_DELETE, false);
        Self {
            event_tap: ptr::null_mut(),
            proxy: ptr::null_mut(),
            event_source,
            backspace_down,
            backspace_up,
            convert_tool_config: ConvertToolConfig::default(),
            on_status_changed: None,
        }
    }
}

#[cfg(target_os = "macos")]
impl EventSender for CoreGraphicsEventSender {
    fn re_enable_event_tap(&mut self) {
        unsafe {
            if !self.event_tap.is_null() {
                CGEventTapEnable(self.event_tap, true);
            }
        }
    }

    fn send_backspace(&mut self) {
        unsafe {
            CGEventTapPostEvent(self.proxy, self.backspace_down);
            CGEventTapPostEvent(self.proxy, self.backspace_up);
        }
    }

    fn send_shift_left_arrow(&mut self) {
        use openkey_engine::types::KEY_LEFT;
        unsafe {
            let down = CGEventCreateKeyboardEvent(self.event_source, KEY_LEFT, true);
            let up = CGEventCreateKeyboardEvent(self.event_source, KEY_LEFT, false);
            let flags = CGEventGetFlags(down) | CG_FLAG_SHIFT;
            CGEventSetFlags(down, flags);
            CGEventSetFlags(up, flags);
            CGEventTapPostEvent(self.proxy, down);
            CGEventTapPostEvent(self.proxy, up);
            CFRelease(down);
            CFRelease(up);
        }
    }

    fn send_empty_character(&mut self, ch: u16) {
        self.send_unicode_string(&[ch]);
    }

    fn send_unicode_string(&mut self, chars: &[u16]) {
        unsafe {
            let down = CGEventCreateKeyboardEvent(self.event_source, 0, true);
            let up = CGEventCreateKeyboardEvent(self.event_source, 0, false);
            CGEventKeyboardSetUnicodeString(down, chars.len(), chars.as_ptr());
            CGEventKeyboardSetUnicodeString(up, chars.len(), chars.as_ptr());
            CGEventTapPostEvent(self.proxy, down);
            CGEventTapPostEvent(self.proxy, up);
            CFRelease(down);
            CFRelease(up);
        }
    }

    fn send_key_code(&mut self, keycode: u16, shift: bool) {
        unsafe {
            let down = CGEventCreateKeyboardEvent(self.event_source, keycode, true);
            let up = CGEventCreateKeyboardEvent(self.event_source, keycode, false);
            let mut flags = CGEventGetFlags(down);
            if shift {
                flags |= CG_FLAG_SHIFT;
            } else {
                flags &= !CG_FLAG_SHIFT;
            }
            flags |= CG_FLAG_NON_COALESCED;
            CGEventSetFlags(down, flags);
            CGEventSetFlags(up, flags);
            CGEventTapPostEvent(self.proxy, down);
            CGEventTapPostEvent(self.proxy, up);
            CFRelease(down);
            CFRelease(up);
        }
    }

    fn beep(&mut self) {
        play_system_beep();
    }

    fn on_quick_convert(&mut self) {
        let _ = quick_convert_pasteboard(&self.convert_tool_config);
    }

    fn on_input_method_changed(&mut self, language: i32, _will_notify: bool) {
        if let Some(cb) = self.on_status_changed {
            cb(language, -1);
        }
    }
    fn on_code_table_changed(&mut self, code_table: i32) {
        if let Some(cb) = self.on_status_changed {
            cb(-1, code_table);
        }
    }
}

pub struct EventTapContext {
    pub handler: MacEventHandler,
    pub sender: CoreGraphicsEventSender,
    pub run_loop_source: CFRunLoopSourceRef,
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn openkey_cg_event_callback(
    proxy: CGEventTapProxy,
    raw_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    if user_info.is_null() {
        return event;
    }
    let ctx = &mut *(user_info as *mut EventTapContext);
    ctx.sender.proxy = proxy;

    let ev_type = map_cg_event_type(raw_type);
    if ev_type == MacEventType::TapDisabledByTimeout
        || ev_type == MacEventType::TapDisabledByUserInput
    {
        ctx.sender.re_enable_event_tap();
        return event;
    }

    let is_own = if !event.is_null() && !ctx.sender.event_source.is_null() {
        CGEventGetIntegerValueField(event, K_CG_EVENT_SOURCE_STATE_ID)
            == (CGEventSourceGetSourceStateID(ctx.sender.event_source) as i64)
    } else {
        false
    };

    let flags = if !event.is_null() {
        CGEventGetFlags(event)
    } else {
        0
    };
    let mut keycode = if !event.is_null() {
        CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) as u16
    } else {
        0
    };

    if ev_type == MacEventType::KeyDown {
        let front_app = frontmost_application_bundle_id();
        if front_app != crate::config::OPENKEY_BUNDLE && front_app != ctx.handler.frontmost_app {
            if ctx.handler.engine.config.use_smart_switch_key != 0 {
                ctx.handler.on_active_app_changed(&front_app, &mut ctx.sender);
            } else {
                ctx.handler.frontmost_app = front_app;
            }
        }
        if ctx.handler.perform_layout_compat {
            keycode = convert_cg_event_to_layout_compat_keycode(event, keycode);
        }
        ctx.handler.spotlight_visible = is_spotlight_visible();
        if ctx.handler.engine.config.other_language != 0 {
            ctx.handler.current_input_source_is_english = is_current_input_source_english();
        }
    }
    match ctx
        .handler
        .handle_raw_event(ev_type, keycode, flags, is_own, &mut ctx.sender)
    {
        CallbackAction::PassThrough => event,
        CallbackAction::Swallow => ptr::null_mut(),
    }
}

impl EventTapContext {
    #[cfg(target_os = "macos")]
    pub fn start_event_tap(&mut self) -> Result<(), String> {
        unsafe {
            let mask: CGEventMask = (1 << K_CG_EVENT_KEY_DOWN)
                | (1 << K_CG_EVENT_KEY_UP)
                | (1 << K_CG_EVENT_FLAGS_CHANGED)
                | (1 << K_CG_EVENT_LEFT_MOUSE_DOWN)
                | (1 << K_CG_EVENT_RIGHT_MOUSE_DOWN)
                | (1 << K_CG_EVENT_LEFT_MOUSE_DRAGGED)
                | (1 << K_CG_EVENT_RIGHT_MOUSE_DRAGGED);

            let user_ptr = (self as *mut EventTapContext) as *mut c_void;
            let tap = CGEventTapCreate(
                K_CG_SESSION_EVENT_TAP,
                K_CG_HEAD_INSERT_EVENT_TAP,
                K_CG_EVENT_TAP_OPTION_DEFAULT,
                mask,
                openkey_cg_event_callback,
                user_ptr,
            );
            if tap.is_null() {
                return Err(
                    "Failed to create CGEventTap (Accessibility permission required)".to_string(),
                );
            }
            self.sender.event_tap = tap;
            let rls = CFMachPortCreateRunLoopSource(ptr::null(), tap, 0);
            self.run_loop_source = rls;
            CFRunLoopAddSource(CFRunLoopGetCurrent(), rls, kCFRunLoopCommonModes);
            CGEventTapEnable(tap, true);
            Ok(())
        }
    }
    #[cfg(target_os = "macos")]
    pub fn start_accessibility_retry_timer(&mut self) {
        unsafe extern "C" fn retry_timer_cb(timer: *mut c_void, info: *mut c_void) {
            if info.is_null() {
                return;
            }
            let ctx = &mut *(info as *mut EventTapContext);
            if !ctx.sender.event_tap.is_null() {
                CFRunLoopTimerInvalidate(timer);
                return;
            }
            if ctx.start_event_tap().is_ok() {
                CFRunLoopTimerInvalidate(timer);
            }
        }

        unsafe {
            let mut timer_ctx = CFRunLoopTimerContext {
                version: 0,
                info: (self as *mut EventTapContext) as *mut c_void,
                retain: ptr::null(),
                release: ptr::null(),
                copy_description: ptr::null(),
            };
            let timer = CFRunLoopTimerCreate(
                ptr::null(),
                CFAbsoluteTimeGetCurrent() + 1.0,
                1.0,
                0,
                0,
                retry_timer_cb,
                &mut timer_ctx,
            );
            if !timer.is_null() {
                CFRunLoopAddTimer(CFRunLoopGetMain(), timer, kCFRunLoopCommonModes);
                CFRelease(timer);
            }
        }
    }


    #[cfg(target_os = "macos")]
    pub fn run_loop(&self) {
        unsafe {
            CFRunLoopRun();
        }
    }

    #[cfg(target_os = "macos")]
    pub fn stop_event_tap(&mut self) {
        unsafe {
            if !self.run_loop_source.is_null() {
                CFRunLoopRemoveSource(
                    CFRunLoopGetCurrent(),
                    self.run_loop_source,
                    kCFRunLoopCommonModes,
                );
                CFRelease(self.run_loop_source);
                self.run_loop_source = ptr::null_mut();
            }
            if !self.sender.event_tap.is_null() {
                CFMachPortInvalidate(self.sender.event_tap);
                CFRelease(self.sender.event_tap);
                self.sender.event_tap = ptr::null_mut();
            }
        }
    }
}
