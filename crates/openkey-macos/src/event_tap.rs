use std::ffi::c_void;
use std::ptr;

use openkey_engine::types::KEY_DELETE;

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

pub type CFTypeRef = *const c_void;
pub type CFMachPortRef = *mut c_void;
pub type CFRunLoopRef = *mut c_void;
pub type CFRunLoopSourceRef = *mut c_void;
pub type CFStringRef = *const c_void;
pub type CGEventRef = *mut c_void;
pub type CGEventTapProxy = *mut c_void;
pub type CGEventSourceRef = *mut c_void;
pub type CGEventMask = u64;

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
extern "C" {
    static kCFRunLoopCommonModes: CFStringRef;

    fn CFRelease(cf: CFTypeRef);
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

    fn AXIsProcessTrusted() -> bool;
}

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

    fn beep(&mut self) {}
    fn on_quick_convert(&mut self) {}
    fn on_input_method_changed(&mut self, _language: i32, _will_notify: bool) {}
    fn on_code_table_changed(&mut self, _code_table: i32) {}
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
    let keycode = if !event.is_null() {
        CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) as u16
    } else {
        0
    };

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
