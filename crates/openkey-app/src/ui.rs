use std::ffi::{c_char, c_void, CString};
use std::fs;
use std::path::{Path, PathBuf};
use std::ptr;

use openkey_macos::config::{MacConfig, OPENKEY_BUNDLE};
use openkey_macos::event_tap::quick_convert_pasteboard;
use openkey_macos::handler::MacEventHandler;
use openkey_macos::hotkeys::format_quick_convert_menu_title;

pub const DEFAULT_SWITCH_STATUS: i32 = 0x7A000206;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuBarSnapshot {
    pub vietnamese_enabled: bool,
    pub status_icon_name: &'static str,
    pub status_alternate_icon_name: &'static str,
    pub status_title: &'static str,
    pub is_template_icon: bool,
    pub selected_input_type: i32,
    pub selected_code_table: i32,
    pub quick_convert_title: String,
    pub run_on_startup: bool,
    pub show_on_dock: bool,
}

#[derive(Debug, Clone)]
pub struct AppUiController {
    pub domain: String,
    pub config: MacConfig,
    pub handler: MacEventHandler,
}

impl Default for AppUiController {
    fn default() -> Self {
        Self::with_domain(OPENKEY_BUNDLE)
    }
}

impl AppUiController {
    pub fn with_domain(domain: &str) -> Self {
        let config = MacConfig::default();
        let handler = MacEventHandler::new(config.clone());
        Self {
            domain: domain.to_string(),
            config,
            handler,
        }
    }

    pub fn load_from_domain(domain: &str) -> Self {
        let mut config = MacConfig::default();
        config.load_from_cf_preferences(domain);
        let handler = MacEventHandler::new(config.clone());
        Self {
            domain: domain.to_string(),
            config,
            handler,
        }
    }

    /// Resets configuration to OpenKey's factory defaults (`[AppDelegate loadDefaultConfig]`) and saves to `CFPreferences`.
    pub fn load_default_config(&mut self) {
        self.config = MacConfig::default();
        self.config.engine.language = 1;
        self.config.engine.input_type = 0;
        self.config.engine.free_mark = 0;
        self.config.engine.check_spelling = 1;
        self.config.engine.code_table = 0;
        self.config.engine.switch_key_status = DEFAULT_SWITCH_STATUS;
        self.config.engine.quick_telex = 0;
        self.config.engine.use_modern_orthography = 0;
        self.config.engine.restore_if_wrong_spelling = 0;
        self.config.engine.fix_recommend_browser = 1;
        self.config.engine.use_macro = 1;
        self.config.engine.use_macro_in_english_mode = 0;
        self.config.send_key_step_by_step = false;
        self.config.engine.use_smart_switch_key = 1;
        self.config.engine.upper_case_first_char = 0;
        self.config.engine.temp_off_spelling = 0;
        self.config.engine.allow_consonant_zfwj = 0;
        self.config.engine.quick_start_consonant = 0;
        self.config.engine.quick_end_consonant = 0;
        self.config.engine.remember_code = 1;
        self.config.engine.other_language = 1;
        self.config.engine.temp_off_open_key = 0;
        self.config.show_on_dock = false;
        self.config.fix_chromium_browser = false;
        self.config.perform_layout_compat = false;
        self.config.gray_icon = true;
        self.config.run_on_startup = true;

        self.handler = MacEventHandler::new(self.config.clone());
        self.save_preferences();
    }

    pub fn save_preferences(&mut self) {
        self.config.engine = self.handler.engine.config.clone();
        self.config.convert_tool = self.handler.engine.convert_tool.clone();
        self.config.macro_data = self.handler.engine.macros.to_bytes();
        self.config.smart_switch_data = self.handler.engine.smart_switch.to_bytes();
        self.config.save_to_cf_preferences(&self.domain);
    }

    pub fn menu_snapshot(&self) -> MenuBarSnapshot {
        let vn = self.handler.engine.config.language == 1;
        MenuBarSnapshot {
            vietnamese_enabled: vn,
            status_icon_name: if vn { "Status" } else { "StatusEng" },
            status_alternate_icon_name: if vn {
                "StatusHighlighted"
            } else {
                "StatusHighlightedEng"
            },
            status_title: if vn { "V" } else { "E" },
            is_template_icon: self.config.gray_icon,
            selected_input_type: self.handler.engine.config.input_type,
            selected_code_table: self.handler.engine.config.code_table,
            quick_convert_title: format_quick_convert_menu_title(
                self.handler.engine.convert_tool.hot_key,
            ),
            run_on_startup: self.config.run_on_startup,
            show_on_dock: self.config.show_on_dock,
        }
    }

    pub fn on_input_method_selected(&mut self) {
        let next = if self.handler.engine.config.language == 0 {
            1
        } else {
            0
        };
        self.handler.engine.config.language = next;
        if self.handler.engine.config.use_smart_switch_key != 0 {
            let packed = next | (self.handler.engine.config.code_table << 1);
            self.handler
                .engine
                .smart_switch
                .set_app_input_method_status(&self.handler.frontmost_app, packed);
        }
        self.handler.engine.start_new_session();
        self.save_preferences();
    }

    pub fn on_input_type_selected(&mut self, index: i32) {
        self.handler.engine.config.input_type = index;
        self.save_preferences();
    }

    pub fn on_code_table_changed(&mut self, index: i32) {
        self.handler.engine.config.code_table = index;
        self.handler
            .engine
            .macros
            .on_table_code_change(index as usize);
        if self.handler.engine.config.remember_code != 0 {
            let packed = self.handler.engine.config.language | (index << 1);
            self.handler
                .engine
                .smart_switch
                .set_app_input_method_status(&self.handler.frontmost_app, packed);
        }
        self.save_preferences();
    }

    pub fn set_switch_key_control(&mut self, enabled: bool) {
        self.handler.engine.config.switch_key_status &= !0x100;
        if enabled {
            self.handler.engine.config.switch_key_status |= 1 << 8;
        }
        self.save_preferences();
    }

    pub fn set_switch_key_option(&mut self, enabled: bool) {
        self.handler.engine.config.switch_key_status &= !0x200;
        if enabled {
            self.handler.engine.config.switch_key_status |= 1 << 9;
        }
        self.save_preferences();
    }

    pub fn set_switch_key_command(&mut self, enabled: bool) {
        self.handler.engine.config.switch_key_status &= !0x400;
        if enabled {
            self.handler.engine.config.switch_key_status |= 1 << 10;
        }
        self.save_preferences();
    }

    pub fn set_switch_key_shift(&mut self, enabled: bool) {
        self.handler.engine.config.switch_key_status &= !0x800;
        if enabled {
            self.handler.engine.config.switch_key_status |= 1 << 11;
        }
        self.save_preferences();
    }

    pub fn set_switch_key_beep(&mut self, enabled: bool) {
        self.handler.engine.config.switch_key_status &= !0x8000;
        if enabled {
            self.handler.engine.config.switch_key_status |= 1 << 15;
        }
        self.save_preferences();
    }

    pub fn set_switch_key_char(&mut self, key_code: u16, character: u16) {
        let mut status = self.handler.engine.config.switch_key_status as u32;
        status &= 0xFFFF_FF00;
        status |= (key_code & 0xFF) as u32;
        status &= 0x00FF_FFFF;
        status |= ((character & 0xFF) as u32) << 24;
        self.handler.engine.config.switch_key_status = status as i32;
        self.save_preferences();
    }

    pub fn on_quick_convert(&self) -> bool {
        quick_convert_pasteboard(&self.handler.engine.convert_tool)
    }
}

/// Generates a complete macOS `OpenKey.app` bundle directory containing `Contents/Info.plist`,
/// `Contents/MacOS/OpenKey`, and `Contents/Resources/*` copied from `Sources/OpenKey/macOS/ModernKey/Resources/`.
pub fn create_macos_app_bundle(out_dir: &Path) -> Result<PathBuf, String> {
    let app_bundle = out_dir.join("OpenKey.app");
    let contents = app_bundle.join("Contents");
    let macos_dir = contents.join("MacOS");
    let resources_dir = contents.join("Resources");

    fs::create_dir_all(&macos_dir)
        .map_err(|e| format!("failed to create MacOS bundle dir: {}", e))?;
    fs::create_dir_all(&resources_dir)
        .map_err(|e| format!("failed to create Resources bundle dir: {}", e))?;

    let info_plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>OpenKey</string>
    <key>CFBundleIdentifier</key>
    <string>com.tuyenmai.openkey</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>OpenKey</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>2.0.5</string>
    <key>CFBundleVersion</key>
    <string>62</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>LSUIElement</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2019-2026 Tuyen Mai. All rights reserved.</string>
</dict>
</plist>
"#;
    fs::write(contents.join("Info.plist"), info_plist)
        .map_err(|e| format!("failed to write Info.plist: {}", e))?;

    // Copy current executable into Contents/MacOS/OpenKey if available
    if let Ok(exe) = std::env::current_exe() {
        let _ = fs::copy(&exe, macos_dir.join("OpenKey"));
    }

    // Copy icon resources from Sources/OpenKey/macOS/ModernKey/Resources
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let src_resources = repo_root.join("Sources/OpenKey/macOS/ModernKey/Resources");
    if src_resources.exists() {
        if let Ok(entries) = fs::read_dir(&src_resources) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Some(name) = p.file_name() {
                        let _ = fs::copy(&p, resources_dir.join(name));
                    }
                }
            }
        }
    }

    Ok(app_bundle)
}

#[cfg(target_os = "macos")]
#[link(name = "AppKit", kind = "framework")]
#[link(name = "objc")]
extern "C" {
    fn objc_getClass(name: *const c_char) -> *mut c_void;
    fn sel_registerName(name: *const c_char) -> *mut c_void;
    fn objc_msgSend();
}

#[cfg(target_os = "macos")]
unsafe fn cls(name: &str) -> *mut c_void {
    let c = CString::new(name).unwrap();
    objc_getClass(c.as_ptr())
}

#[cfg(target_os = "macos")]
unsafe fn sel(name: &str) -> *mut c_void {
    let c = CString::new(name).unwrap();
    sel_registerName(c.as_ptr())
}

#[cfg(target_os = "macos")]
unsafe fn nsstring(s: &str) -> *mut c_void {
    let c = CString::new(s).unwrap();
    let f: unsafe extern "C" fn(*mut c_void, *mut c_void, *const c_char) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    f(cls("NSString"), sel("stringWithUTF8String:"), c.as_ptr())
}

/// Initializes the native Cocoa `NSStatusBar` status item and menu matching `createStatusBarMenu` in `AppDelegate.m`.
#[cfg(target_os = "macos")]
pub unsafe fn setup_cocoa_status_bar(ui: &AppUiController) -> *mut c_void {
    let msg0: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_f64: unsafe extern "C" fn(*mut c_void, *mut c_void, f64) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_ptr: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_bool: unsafe extern "C" fn(*mut c_void, *mut c_void, i8) =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_isize: unsafe extern "C" fn(*mut c_void, *mut c_void, isize) =
        std::mem::transmute(objc_msgSend as *const ());
    let msg3_item: unsafe extern "C" fn(
        *mut c_void,
        *mut c_void,
        *mut c_void,
        *mut c_void,
        *mut c_void,
    ) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());

    let app = msg0(cls("NSApplication"), sel("sharedApplication"));
    let policy: isize = if ui.config.show_on_dock { 0 } else { 1 }; // 0 = Regular, 1 = Accessory
    let set_policy: unsafe extern "C" fn(*mut c_void, *mut c_void, isize) -> i8 =
        std::mem::transmute(objc_msgSend as *const ());
    set_policy(app, sel("setActivationPolicy:"), policy);

    let status_bar = msg0(cls("NSStatusBar"), sel("systemStatusBar"));
    // NSVariableStatusItemLength = -1.0
    let status_item = msg1_f64(status_bar, sel("statusItemWithLength:"), -1.0);
    let snap = ui.menu_snapshot();

    let button = msg0(status_item, sel("button"));
    if !button.is_null() {
        msg1_ptr(button, sel("setTitle:"), nsstring(snap.status_title));
    }

    let menu_alloc = msg0(cls("NSMenu"), sel("alloc"));
    let menu = msg1_ptr(menu_alloc, sel("initWithTitle:"), nsstring("OpenKey"));
    msg1_bool(menu, sel("setAutoenablesItems:"), 0);

    let item_vn = msg3_item(
        menu,
        sel("addItemWithTitle:action:keyEquivalent:"),
        nsstring("Bật Tiếng Việt"),
        ptr::null_mut(),
        nsstring(""),
    );
    msg1_isize(
        item_vn,
        sel("setState:"),
        if snap.vietnamese_enabled { 1 } else { 0 },
    );

    let sep1 = msg0(cls("NSMenuItem"), sel("separatorItem"));
    msg1_ptr(menu, sel("addItem:"), sep1);

    for (idx, title) in [
        (0, "Unicode dựng sẵn"),
        (1, "TCVN3 (ABC)"),
        (2, "VNI Windows"),
        (3, "Unicode tổ hợp"),
        (4, "Vietnamese Locale CP 1258"),
    ] {
        let item = msg3_item(
            menu,
            sel("addItemWithTitle:action:keyEquivalent:"),
            nsstring(title),
            ptr::null_mut(),
            nsstring(""),
        );
        msg1_isize(
            item,
            sel("setState:"),
            if snap.selected_code_table == idx { 1 } else { 0 },
        );
    }

    let sep2 = msg0(cls("NSMenuItem"), sel("separatorItem"));
    msg1_ptr(menu, sel("addItem:"), sep2);

    for (idx, title) in [
        (0, "Kiểu gõ: Telex"),
        (1, "Kiểu gõ: VNI"),
        (2, "Kiểu gõ: Simple Telex 1"),
        (3, "Kiểu gõ: Simple Telex 2"),
    ] {
        let item = msg3_item(
            menu,
            sel("addItemWithTitle:action:keyEquivalent:"),
            nsstring(title),
            ptr::null_mut(),
            nsstring(""),
        );
        msg1_isize(
            item,
            sel("setState:"),
            if snap.selected_input_type == idx { 1 } else { 0 },
        );
    }

    let sep3 = msg0(cls("NSMenuItem"), sel("separatorItem"));
    msg1_ptr(menu, sel("addItem:"), sep3);

    msg3_item(
        menu,
        sel("addItemWithTitle:action:keyEquivalent:"),
        nsstring(&snap.quick_convert_title),
        ptr::null_mut(),
        nsstring(""),
    );
    msg3_item(
        menu,
        sel("addItemWithTitle:action:keyEquivalent:"),
        nsstring("Thoát"),
        sel("terminate:"),
        nsstring("q"),
    );

    msg1_ptr(status_item, sel("setMenu:"), menu);
    status_item
}
