use std::ffi::{c_char, c_void, CString};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicIsize, AtomicPtr, Ordering};

use openkey_macos::config::{MacConfig, OPENKEY_BUNDLE};
use openkey_macos::event_tap::{quick_convert_pasteboard, EventTapContext};
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
        self.config.send_key_step_by_step = self.handler.send_key_step_by_step;
        self.config.fix_chromium_browser = self.handler.fix_chromium_browser;
        self.config.perform_layout_compat = self.handler.perform_layout_compat;
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

    // ViewController.m option setters
    pub fn set_free_mark(&mut self, enabled: bool) {
        self.handler.engine.config.free_mark = if enabled { 1 } else { 0 };
        self.save_preferences();
    }

    pub fn set_modern_orthography(&mut self, enabled: bool) {
        self.handler.engine.config.use_modern_orthography = if enabled { 1 } else { 0 };
        self.save_preferences();
    }

    pub fn set_check_spelling(&mut self, enabled: bool) {
        self.handler.engine.config.check_spelling = if enabled { 1 } else { 0 };
        self.handler.engine.set_check_spelling();
        self.save_preferences();
    }

    pub fn set_quick_telex(&mut self, enabled: bool) {
        self.handler.engine.config.quick_telex = if enabled { 1 } else { 0 };
        self.save_preferences();
    }

    pub fn set_restore_if_invalid_word(&mut self, enabled: bool) {
        self.handler.engine.config.restore_if_wrong_spelling = if enabled { 1 } else { 0 };
        self.save_preferences();
    }

    pub fn set_temp_off_spell_checking(&mut self, enabled: bool) {
        self.handler.engine.config.temp_off_spelling = if enabled { 1 } else { 0 };
        self.save_preferences();
    }

    pub fn set_allow_zfwj(&mut self, enabled: bool) {
        self.handler.engine.config.allow_consonant_zfwj = if enabled { 1 } else { 0 };
        self.save_preferences();
    }

    pub fn set_fix_recommend_browser(&mut self, enabled: bool) {
        self.handler.engine.config.fix_recommend_browser = if enabled { 1 } else { 0 };
        self.save_preferences();
    }

    pub fn set_use_macro(&mut self, enabled: bool) {
        self.handler.engine.config.use_macro = if enabled { 1 } else { 0 };
        self.save_preferences();
    }

    pub fn set_use_macro_in_english_mode(&mut self, enabled: bool) {
        self.handler.engine.config.use_macro_in_english_mode = if enabled { 1 } else { 0 };
        self.save_preferences();
    }

    pub fn set_auto_caps_macro(&mut self, enabled: bool) {
        self.handler.engine.config.auto_caps_macro = if enabled { 1 } else { 0 };
        self.save_preferences();
    }

    pub fn set_send_key_step_by_step(&mut self, enabled: bool) {
        self.handler.send_key_step_by_step = enabled;
        self.save_preferences();
    }

    pub fn set_perform_layout_compat(&mut self, enabled: bool) {
        self.handler.perform_layout_compat = enabled;
        self.save_preferences();
    }

    pub fn set_use_smart_switch_key(&mut self, enabled: bool) {
        self.handler.engine.config.use_smart_switch_key = if enabled { 1 } else { 0 };
        self.save_preferences();
    }

    pub fn set_upper_case_first_char(&mut self, enabled: bool) {
        self.handler.engine.config.upper_case_first_char = if enabled { 1 } else { 0 };
        self.save_preferences();
    }

    pub fn set_quick_start_consonant(&mut self, enabled: bool) {
        self.handler.engine.config.quick_start_consonant = if enabled { 1 } else { 0 };
        self.save_preferences();
    }

    pub fn set_quick_end_consonant(&mut self, enabled: bool) {
        self.handler.engine.config.quick_end_consonant = if enabled { 1 } else { 0 };
        self.save_preferences();
    }

    pub fn set_temp_off_open_key(&mut self, enabled: bool) {
        self.handler.engine.config.temp_off_open_key = if enabled { 1 } else { 0 };
        self.save_preferences();
    }

    pub fn set_remember_table_code(&mut self, enabled: bool) {
        self.handler.engine.config.remember_code = if enabled { 1 } else { 0 };
        self.save_preferences();
    }

    pub fn set_other_language(&mut self, enabled: bool) {
        self.handler.engine.config.other_language = if enabled { 1 } else { 0 };
        self.save_preferences();
    }

    pub fn set_fix_chromium_browser(&mut self, enabled: bool) {
        self.handler.fix_chromium_browser = enabled;
        self.save_preferences();
    }

    pub fn set_show_icon_on_dock(&mut self, enabled: bool) {
        self.config.show_on_dock = enabled;
        self.save_preferences();
    }

    pub fn set_gray_icon(&mut self, enabled: bool) {
        self.config.gray_icon = enabled;
        self.save_preferences();
    }

    pub fn set_run_on_startup(&mut self, enabled: bool) {
        self.config.run_on_startup = enabled;
        self.save_preferences();
    }

    pub fn set_show_ui_on_startup(&mut self, enabled: bool) {
        self.config.show_ui_on_startup = enabled;
        self.save_preferences();
    }

    pub fn set_check_new_version_on_startup(&mut self, enabled: bool) {
        self.config.dont_check_update = !enabled;
        self.save_preferences();
    }

    // ConvertToolViewController.mm option setters
    pub fn turn_off_all_convert_caps_options(&mut self) {
        self.handler.engine.convert_tool.to_all_caps = false;
        self.handler.engine.convert_tool.to_all_non_caps = false;
        self.handler.engine.convert_tool.to_caps_first_letter = false;
        self.handler.engine.convert_tool.to_caps_each_word = false;
    }

    pub fn set_convert_to_all_caps(&mut self, enabled: bool) {
        self.turn_off_all_convert_caps_options();
        self.handler.engine.convert_tool.to_all_caps = enabled;
        self.save_preferences();
    }

    pub fn set_convert_to_non_caps(&mut self, enabled: bool) {
        self.turn_off_all_convert_caps_options();
        self.handler.engine.convert_tool.to_all_non_caps = enabled;
        self.save_preferences();
    }

    pub fn set_convert_to_caps_first_letter(&mut self, enabled: bool) {
        self.turn_off_all_convert_caps_options();
        self.handler.engine.convert_tool.to_caps_first_letter = enabled;
        self.save_preferences();
    }

    pub fn set_convert_to_caps_each_word(&mut self, enabled: bool) {
        self.turn_off_all_convert_caps_options();
        self.handler.engine.convert_tool.to_caps_each_word = enabled;
        self.save_preferences();
    }

    pub fn set_convert_remove_mark(&mut self, enabled: bool) {
        self.handler.engine.convert_tool.remove_mark = enabled;
        self.save_preferences();
    }

    pub fn set_convert_from_code(&mut self, code: u8) {
        self.handler.engine.convert_tool.from_code = code;
        self.save_preferences();
    }

    pub fn set_convert_to_code(&mut self, code: u8) {
        self.handler.engine.convert_tool.to_code = code;
        self.save_preferences();
    }

    pub fn reverse_convert_codes(&mut self) {
        let from = self.handler.engine.convert_tool.from_code;
        let to = self.handler.engine.convert_tool.to_code;
        self.handler.engine.convert_tool.from_code = to;
        self.handler.engine.convert_tool.to_code = from;
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
    <key>CFBundleDisplayName</key>
    <string>OpenKey</string>
    <key>CFBundleExecutable</key>
    <string>OpenKey</string>
    <key>CFBundleIconFile</key>
    <string>Icon</string>
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
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.utilities</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>LSMultipleInstancesProhibited</key>
    <true/>
    <key>LSUIElement</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2019-2026 Tuyen Mai. All rights reserved.</string>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
</dict>
</plist>
"#;
    fs::write(contents.join("Info.plist"), info_plist)
        .map_err(|e| format!("failed to write Info.plist: {}", e))?;
    let _ = fs::write(contents.join("PkgInfo"), b"APPL????");

    if let Ok(exe) = std::env::current_exe() {
        let sibling_bin = exe
            .parent()
            .map(|p| p.join("openkey"))
            .filter(|p| p.exists())
            .unwrap_or(exe);
        let dst_bin = macos_dir.join("OpenKey");
        let _ = fs::copy(&sibling_bin, &dst_bin);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = fs::metadata(&dst_bin) {
                let mut perms = meta.permissions();
                perms.set_mode(0o755);
                let _ = fs::set_permissions(&dst_bin, perms);
            }
        }
    }
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

    #[cfg(target_os = "macos")]
    {
        if let Some(bundle_str) = app_bundle.to_str() {
            let _ = std::process::Command::new("codesign")
                .args([
                    "--force",
                    "--deep",
                    "--identifier",
                    OPENKEY_BUNDLE,
                    "--sign",
                    "-",
                    bundle_str,
                ])
                .output();
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
    fn objc_allocateClassPair(
        superclass: *mut c_void,
        name: *const c_char,
        extra_bytes: usize,
    ) -> *mut c_void;
    fn class_addMethod(
        cls: *mut c_void,
        name: *mut c_void,
        imp: *const c_void,
        types: *const c_char,
    ) -> i8;
    fn objc_registerClassPair(cls: *mut c_void);
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

static ACTIVE_EVENT_TAP_CTX: AtomicPtr<EventTapContext> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_STATUS_ITEM: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

pub fn register_active_runtime(ctx: *mut EventTapContext, status_item: *mut c_void) {
    ACTIVE_EVENT_TAP_CTX.store(ctx, Ordering::SeqCst);
    ACTIVE_STATUS_ITEM.store(status_item, Ordering::SeqCst);
}

pub fn on_runtime_status_changed(language: i32, code_table: i32) {
    let mut ui = AppUiController::load_from_domain(OPENKEY_BUNDLE);
    if language >= 0 {
        ui.handler.engine.config.language = language;
    }
    if code_table >= 0 {
        ui.handler.engine.config.code_table = code_table;
    }
    let ctx_ptr = ACTIVE_EVENT_TAP_CTX.load(Ordering::SeqCst);
    if !ctx_ptr.is_null() {
        unsafe {
            let ctx = &mut *ctx_ptr;
            ui.handler.engine.smart_switch = ctx.handler.engine.smart_switch.clone();
            ui.handler.engine.macros = ctx.handler.engine.macros.clone();
        }
    }
    ui.save_preferences();
    #[cfg(target_os = "macos")]
    unsafe {
        let status_item = ACTIVE_STATUS_ITEM.load(Ordering::SeqCst);
        if !status_item.is_null() {
            populate_status_item(status_item, &ui);
        }
    }
}

#[cfg(target_os = "macos")]
unsafe fn sync_ui_to_active_runtime(ui: &mut AppUiController) {
    let ctx_ptr = ACTIVE_EVENT_TAP_CTX.load(Ordering::SeqCst);
    if !ctx_ptr.is_null() {
        let ctx = &mut *ctx_ptr;
        ctx.handler.engine.config = ui.handler.engine.config.clone();
        ctx.handler.engine.convert_tool = ui.handler.engine.convert_tool.clone();
        ctx.handler.send_key_step_by_step = ui.handler.send_key_step_by_step;
        ctx.handler.fix_chromium_browser = ui.handler.fix_chromium_browser;
        ctx.handler.perform_layout_compat = ui.handler.perform_layout_compat;
        ctx.sender.convert_tool_config = ui.handler.engine.convert_tool.clone();
        if ctx.sender.event_tap.is_null() {
            let _ = ctx.start_event_tap();
        }
    }
    let status_item = ACTIVE_STATUS_ITEM.load(Ordering::SeqCst);
    if !status_item.is_null() {
        populate_status_item(status_item, ui);
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn objc_on_input_method_selected(_this: *mut c_void, _cmd: *mut c_void, _sender: *mut c_void) {
    let mut ui = AppUiController::load_from_domain(OPENKEY_BUNDLE);
    ui.on_input_method_selected();
    sync_ui_to_active_runtime(&mut ui);
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn objc_on_input_type_selected(_this: *mut c_void, _cmd: *mut c_void, sender: *mut c_void) {
    if !sender.is_null() {
        let get_tag: unsafe extern "C" fn(*mut c_void, *mut c_void) -> isize =
            std::mem::transmute(objc_msgSend as *const ());
        let tag = get_tag(sender, sel("tag")) as i32;
        let mut ui = AppUiController::load_from_domain(OPENKEY_BUNDLE);
        ui.on_input_type_selected(tag);
        sync_ui_to_active_runtime(&mut ui);
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn objc_on_code_selected(_this: *mut c_void, _cmd: *mut c_void, sender: *mut c_void) {
    if !sender.is_null() {
        let get_tag: unsafe extern "C" fn(*mut c_void, *mut c_void) -> isize =
            std::mem::transmute(objc_msgSend as *const ());
        let tag = get_tag(sender, sel("tag")) as i32;
        let mut ui = AppUiController::load_from_domain(OPENKEY_BUNDLE);
        ui.on_code_table_changed(tag);
        sync_ui_to_active_runtime(&mut ui);
    }
}
#[cfg(target_os = "macos")]
unsafe extern "C" fn objc_on_quick_convert(_this: *mut c_void, _cmd: *mut c_void, _sender: *mut c_void) {
    let ui = AppUiController::load_from_domain(OPENKEY_BUNDLE);
    let _ = ui.on_quick_convert();
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NSPoint {
    pub x: f64,
    pub y: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NSSize {
    pub width: f64,
    pub height: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NSRect {
    pub origin: NSPoint,
    pub size: NSSize,
}

impl NSRect {
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            origin: NSPoint { x, y },
            size: NSSize { width, height },
        }
    }
}

static ACTIVE_CP_WINDOW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_CP_TAB_INDEX: AtomicIsize = AtomicIsize::new(0);
static ACTIVE_CP_TAB_BOXES: [AtomicPtr<c_void>; 4] = [
    AtomicPtr::new(std::ptr::null_mut()),
    AtomicPtr::new(std::ptr::null_mut()),
    AtomicPtr::new(std::ptr::null_mut()),
    AtomicPtr::new(std::ptr::null_mut()),
];
static ACTIVE_CP_TAB_BUTTONS: [AtomicPtr<c_void>; 4] = [
    AtomicPtr::new(std::ptr::null_mut()),
    AtomicPtr::new(std::ptr::null_mut()),
    AtomicPtr::new(std::ptr::null_mut()),
    AtomicPtr::new(std::ptr::null_mut()),
];
static ACTIVE_CP_RADIO_VIET: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ACTIVE_CP_RADIO_ENG: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

pub fn cocoa_control_panel_active_tab() -> usize {
    ACTIVE_CP_TAB_INDEX.load(Ordering::SeqCst).max(0) as usize
}

#[cfg(target_os = "macos")]
pub unsafe fn show_control_panel_tab(index: usize) {
    let idx = index.min(3);
    ACTIVE_CP_TAB_INDEX.store(idx as isize, Ordering::SeqCst);
    let msg1_bool: unsafe extern "C" fn(*mut c_void, *mut c_void, i8) =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_isize: unsafe extern "C" fn(*mut c_void, *mut c_void, isize) =
        std::mem::transmute(objc_msgSend as *const ());

    for i in 0..4 {
        let box_ptr = ACTIVE_CP_TAB_BOXES[i].load(Ordering::SeqCst);
        if !box_ptr.is_null() {
            msg1_bool(box_ptr, sel("setHidden:"), if i == idx { 0 } else { 1 });
        }
        let btn_ptr = ACTIVE_CP_TAB_BUTTONS[i].load(Ordering::SeqCst);
        if !btn_ptr.is_null() {
            msg1_isize(btn_ptr, sel("setState:"), if i == idx { 1 } else { 0 });
        }
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn objc_on_tab_button_clicked(_this: *mut c_void, _cmd: *mut c_void, sender: *mut c_void) {
    if sender.is_null() {
        return;
    }
    let get_isize: unsafe extern "C" fn(*mut c_void, *mut c_void) -> isize =
        std::mem::transmute(objc_msgSend as *const ());
    let tag = get_isize(sender, sel("tag")).max(0) as usize;
    show_control_panel_tab(tag);
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn objc_on_popup_input_type_changed(_this: *mut c_void, _cmd: *mut c_void, sender: *mut c_void) {
    if sender.is_null() {
        return;
    }
    let get_isize: unsafe extern "C" fn(*mut c_void, *mut c_void) -> isize =
        std::mem::transmute(objc_msgSend as *const ());
    let idx = get_isize(sender, sel("indexOfSelectedItem")) as i32;
    let mut ui = AppUiController::load_from_domain(OPENKEY_BUNDLE);
    ui.on_input_type_selected(idx);
    sync_ui_to_active_runtime(&mut ui);
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn objc_on_popup_code_changed(_this: *mut c_void, _cmd: *mut c_void, sender: *mut c_void) {
    if sender.is_null() {
        return;
    }
    let get_isize: unsafe extern "C" fn(*mut c_void, *mut c_void) -> isize =
        std::mem::transmute(objc_msgSend as *const ());
    let idx = get_isize(sender, sel("indexOfSelectedItem")) as i32;
    let mut ui = AppUiController::load_from_domain(OPENKEY_BUNDLE);
    ui.on_code_table_changed(idx);
    sync_ui_to_active_runtime(&mut ui);
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn objc_on_radio_language_changed(_this: *mut c_void, _cmd: *mut c_void, sender: *mut c_void) {
    if sender.is_null() {
        return;
    }
    let get_isize: unsafe extern "C" fn(*mut c_void, *mut c_void) -> isize =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_isize: unsafe extern "C" fn(*mut c_void, *mut c_void, isize) =
        std::mem::transmute(objc_msgSend as *const ());
    let lang = get_isize(sender, sel("tag")) as i32;
    let mut ui = AppUiController::load_from_domain(OPENKEY_BUNDLE);
    if ui.handler.engine.config.language != lang {
        ui.on_input_method_selected();
        sync_ui_to_active_runtime(&mut ui);
    }
    let v_btn = ACTIVE_CP_RADIO_VIET.load(Ordering::SeqCst);
    let e_btn = ACTIVE_CP_RADIO_ENG.load(Ordering::SeqCst);
    if !v_btn.is_null() {
        msg1_isize(v_btn, sel("setState:"), if lang == 1 { 1 } else { 0 });
    }
    if !e_btn.is_null() {
        msg1_isize(e_btn, sel("setState:"), if lang == 0 { 1 } else { 0 });
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn objc_on_ok_clicked(_this: *mut c_void, _cmd: *mut c_void, _sender: *mut c_void) {
    let win = ACTIVE_CP_WINDOW.load(Ordering::SeqCst);
    if !win.is_null() {
        let msg0: unsafe extern "C" fn(*mut c_void, *mut c_void) =
            std::mem::transmute(objc_msgSend as *const ());
        msg0(win, sel("close"));
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn objc_on_default_config_clicked(_this: *mut c_void, _cmd: *mut c_void, _sender: *mut c_void) {
    let mut ui = AppUiController::load_from_domain(OPENKEY_BUNDLE);
    ui.load_default_config();
    sync_ui_to_active_runtime(&mut ui);
    let old_win = ACTIVE_CP_WINDOW.load(Ordering::SeqCst);
    if !old_win.is_null() {
        let msg0: unsafe extern "C" fn(*mut c_void, *mut c_void) =
            std::mem::transmute(objc_msgSend as *const ());
        msg0(old_win, sel("close"));
    }
    let new_win = build_control_panel_window(&ui);
    show_window(new_win);
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn objc_on_checkbox_toggled(_this: *mut c_void, _cmd: *mut c_void, sender: *mut c_void) {
    if sender.is_null() {
        return;
    }
    let get_isize: unsafe extern "C" fn(*mut c_void, *mut c_void) -> isize =
        std::mem::transmute(objc_msgSend as *const ());
    let tag = get_isize(sender, sel("tag"));
    let state = get_isize(sender, sel("state")) != 0;
    let mut ui = AppUiController::load_from_domain(OPENKEY_BUNDLE);
    match tag {
        1 => ui.set_modern_orthography(state),
        2 => ui.set_check_spelling(state),
        3 => ui.set_restore_if_invalid_word(state),
        4 => ui.set_quick_telex(state),
        5 => ui.set_fix_recommend_browser(state),
        6 => ui.set_use_macro(state),
        7 => ui.set_use_macro_in_english_mode(state),
        8 => ui.set_auto_caps_macro(state),
        9 => ui.set_use_smart_switch_key(state),
        10 => ui.set_upper_case_first_char(state),
        11 => ui.set_allow_zfwj(state),
        12 => ui.set_quick_start_consonant(state),
        13 => ui.set_quick_end_consonant(state),
        14 => ui.set_remember_table_code(state),
        15 => ui.set_other_language(state),
        16 => ui.set_temp_off_spelling(state),
        17 => ui.set_temp_off_open_key(state),
        18 => ui.set_gray_icon(state),
        19 => ui.set_run_on_startup(state),
        20 => ui.set_convert_remove_mark(state),
        21 => ui.set_convert_to_all_caps(state),
        22 => ui.set_convert_to_non_caps(state),
        23 => ui.set_convert_to_caps_first_letter(state),
        24 => ui.set_convert_to_caps_each_word(state),
        25 => ui.set_show_icon_on_dock(state),
        26 => ui.set_show_ui_on_startup(state),
        27 => ui.set_perform_layout_compat(state),
        28 => ui.set_fix_chromium_browser(state),
        29 => ui.set_send_key_step_by_step(state),
        30 => ui.set_check_new_version_on_startup(state),
        101 => ui.set_switch_key_control(state),
        102 => ui.set_switch_key_option(state),
        103 => ui.set_switch_key_command(state),
        104 => ui.set_switch_key_shift(state),
        105 => ui.set_switch_key_beep(state),
        _ => {}
    }
    sync_ui_to_active_runtime(&mut ui);
}

impl AppUiController {
    pub fn set_temp_off_spelling(&mut self, enabled: bool) {
        self.set_temp_off_spell_checking(enabled);
    }
}

#[cfg(target_os = "macos")]
unsafe fn show_window(win: *mut c_void) {
    if win.is_null() {
        return;
    }
    let msg0: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_ptr: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_bool: unsafe extern "C" fn(*mut c_void, *mut c_void, i8) =
        std::mem::transmute(objc_msgSend as *const ());
    msg0(win, sel("center"));
    msg1_ptr(win, sel("makeKeyAndOrderFront:"), std::ptr::null_mut());
    let app = msg0(cls("NSApplication"), sel("sharedApplication"));
    msg1_bool(app, sel("activateIgnoringOtherApps:"), 1);
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn objc_on_control_panel_selected(_this: *mut c_void, _cmd: *mut c_void, _sender: *mut c_void) {
    let ui = AppUiController::load_from_domain(OPENKEY_BUNDLE);
    let win = build_control_panel_window(&ui);
    show_window(win);
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn objc_on_macro_selected(_this: *mut c_void, _cmd: *mut c_void, _sender: *mut c_void) {
    let ui = AppUiController::load_from_domain(OPENKEY_BUNDLE);
    show_window(build_macro_window(&ui));
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn objc_on_convert_tool_selected(_this: *mut c_void, _cmd: *mut c_void, _sender: *mut c_void) {
    let ui = AppUiController::load_from_domain(OPENKEY_BUNDLE);
    show_window(build_convert_tool_window(&ui));
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn objc_on_about_selected(_this: *mut c_void, _cmd: *mut c_void, _sender: *mut c_void) {
    let ui = AppUiController::load_from_domain(OPENKEY_BUNDLE);
    show_window(build_about_window(&ui));
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn objc_app_should_handle_reopen(
    _this: *mut c_void,
    _cmd: *mut c_void,
    _sender: *mut c_void,
    _has_visible_windows: i8,
) -> i8 {
    open_control_panel_window();
    1
}

#[cfg(target_os = "macos")]
pub unsafe fn open_control_panel_window() {
    let ui = AppUiController::load_from_domain(OPENKEY_BUNDLE);
    let existing = ACTIVE_CP_WINDOW.load(Ordering::SeqCst);
    if !existing.is_null() {
        show_window(existing);
    } else {
        let win = build_control_panel_window(&ui);
        show_window(win);
    }
}

/// Registers the `OpenKeyRustDelegate` Objective-C class with AppKit menu & window action selectors.
pub fn register_objc_delegate_class() -> *mut c_void {
    #[cfg(target_os = "macos")]
    unsafe {
        let existing = cls("OpenKeyRustDelegate");
        if !existing.is_null() {
            return existing;
        }
        let superclass = cls("NSObject");
        let name = CString::new("OpenKeyRustDelegate").unwrap();
        let new_cls = objc_allocateClassPair(superclass, name.as_ptr(), 0);
        if new_cls.is_null() {
            return cls("OpenKeyRustDelegate");
        }
        let sig = CString::new("v@:@").unwrap();
        for (sel_name, imp) in [
            ("onInputMethodSelected:", objc_on_input_method_selected as *const c_void),
            ("onInputTypeSelected:", objc_on_input_type_selected as *const c_void),
            ("onCodeSelected:", objc_on_code_selected as *const c_void),
            ("onQuickConvert:", objc_on_quick_convert as *const c_void),
            ("onCheckboxToggled:", objc_on_checkbox_toggled as *const c_void),
            ("onTabButtonClicked:", objc_on_tab_button_clicked as *const c_void),
            ("onPopupInputTypeChanged:", objc_on_popup_input_type_changed as *const c_void),
            ("onPopupCodeChanged:", objc_on_popup_code_changed as *const c_void),
            ("onRadioLanguageChanged:", objc_on_radio_language_changed as *const c_void),
            ("onOKClicked:", objc_on_ok_clicked as *const c_void),
            ("onDefaultConfigClicked:", objc_on_default_config_clicked as *const c_void),
            ("onControlPanelSelected:", objc_on_control_panel_selected as *const c_void),
            ("onMacroSelected:", objc_on_macro_selected as *const c_void),
            ("onConvertToolSelected:", objc_on_convert_tool_selected as *const c_void),
            ("onAboutSelected:", objc_on_about_selected as *const c_void),
        ] {
            class_addMethod(new_cls, sel(sel_name), imp, sig.as_ptr());
        }
        let reopen_sig = CString::new("c@:@c").unwrap();
        class_addMethod(
            new_cls,
            sel("applicationShouldHandleReopen:hasVisibleWindows:"),
            objc_app_should_handle_reopen as *const c_void,
            reopen_sig.as_ptr(),
        );
        objc_registerClassPair(new_cls);
        new_cls
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::ptr::null_mut()
    }
}

#[cfg(target_os = "macos")]
unsafe fn create_base_window(title: &str, width: f64, height: f64) -> (*mut c_void, *mut c_void, *mut c_void) {
    let msg0: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_ptr: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_bool: unsafe extern "C" fn(*mut c_void, *mut c_void, i8) =
        std::mem::transmute(objc_msgSend as *const ());
    let init_win: unsafe extern "C" fn(*mut c_void, *mut c_void, NSRect, usize, usize, i8) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());

    let _app = msg0(cls("NSApplication"), sel("sharedApplication"));
    let delegate_cls = register_objc_delegate_class();
    let delegate = msg0(msg0(delegate_cls, sel("alloc")), sel("init"));

    let win_alloc = msg0(cls("NSWindow"), sel("alloc"));
    let style_mask: usize = (1 << 0) | (1 << 1); // Titled | Closable
    let win = init_win(
        win_alloc,
        sel("initWithContentRect:styleMask:backing:defer:"),
        NSRect::new(0.0, 0.0, width, height),
        style_mask,
        2, // NSBackingStoreBuffered
        0,
    );
    msg1_bool(win, sel("setReleasedWhenClosed:"), 0);
    msg1_ptr(win, sel("setTitle:"), nsstring(title));
    let content_view = msg0(win, sel("contentView"));
    (win, content_view, delegate)
}

#[cfg(target_os = "macos")]
unsafe fn add_label(content_view: *mut c_void, text: &str, rect: NSRect) -> *mut c_void {
    let msg1_ptr_ret: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_rect: unsafe extern "C" fn(*mut c_void, *mut c_void, NSRect) =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_ptr: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) =
        std::mem::transmute(objc_msgSend as *const ());

    let label = msg1_ptr_ret(cls("NSTextField"), sel("labelWithString:"), nsstring(text));
    msg1_rect(label, sel("setFrame:"), rect);
    msg1_ptr(content_view, sel("addSubview:"), label);
    label
}

#[cfg(target_os = "macos")]
unsafe fn add_bezel_textfield(content_view: *mut c_void, text: &str, rect: NSRect) -> *mut c_void {
    let msg0: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let init_frame: unsafe extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_ptr: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_bool: unsafe extern "C" fn(*mut c_void, *mut c_void, i8) =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_isize: unsafe extern "C" fn(*mut c_void, *mut c_void, isize) =
        std::mem::transmute(objc_msgSend as *const ());

    let tf = init_frame(msg0(cls("NSTextField"), sel("alloc")), sel("initWithFrame:"), rect);
    msg1_ptr(tf, sel("setStringValue:"), nsstring(text));
    msg1_bool(tf, sel("setBezeled:"), 1);
    msg1_bool(tf, sel("setDrawsBackground:"), 1);
    msg1_bool(tf, sel("setEditable:"), 0);
    msg1_bool(tf, sel("setSelectable:"), 1);
    msg1_isize(tf, sel("setAlignment:"), 1); // NSTextAlignmentCenter
    msg1_ptr(content_view, sel("addSubview:"), tf);
    tf
}

#[cfg(target_os = "macos")]
unsafe fn add_checkbox(
    content_view: *mut c_void,
    delegate: *mut c_void,
    title: &str,
    checked: bool,
    tag: isize,
    rect: NSRect,
) -> *mut c_void {
    let msg3_btn: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_isize: unsafe extern "C" fn(*mut c_void, *mut c_void, isize) =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_rect: unsafe extern "C" fn(*mut c_void, *mut c_void, NSRect) =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_ptr: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) =
        std::mem::transmute(objc_msgSend as *const ());

    let btn = msg3_btn(
        cls("NSButton"),
        sel("checkboxWithTitle:target:action:"),
        nsstring(title),
        delegate,
        sel("onCheckboxToggled:"),
    );
    msg1_isize(btn, sel("setState:"), if checked { 1 } else { 0 });
    msg1_isize(btn, sel("setTag:"), tag);
    msg1_rect(btn, sel("setFrame:"), rect);
    msg1_ptr(content_view, sel("addSubview:"), btn);
    btn
}

#[cfg(target_os = "macos")]
unsafe fn add_radio_button(
    content_view: *mut c_void,
    delegate: *mut c_void,
    title: &str,
    selected: bool,
    tag: isize,
    rect: NSRect,
) -> *mut c_void {
    let msg3_btn: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_isize: unsafe extern "C" fn(*mut c_void, *mut c_void, isize) =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_rect: unsafe extern "C" fn(*mut c_void, *mut c_void, NSRect) =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_ptr: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) =
        std::mem::transmute(objc_msgSend as *const ());

    let btn = msg3_btn(
        cls("NSButton"),
        sel("radioButtonWithTitle:target:action:"),
        nsstring(title),
        delegate,
        sel("onRadioLanguageChanged:"),
    );
    msg1_isize(btn, sel("setState:"), if selected { 1 } else { 0 });
    msg1_isize(btn, sel("setTag:"), tag);
    msg1_rect(btn, sel("setFrame:"), rect);
    msg1_ptr(content_view, sel("addSubview:"), btn);
    btn
}

#[cfg(target_os = "macos")]
unsafe fn add_popup_button(
    content_view: *mut c_void,
    delegate: *mut c_void,
    items: &[&str],
    selected_idx: isize,
    action_sel: *mut c_void,
    rect: NSRect,
) -> *mut c_void {
    let msg0: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let init_popup: unsafe extern "C" fn(*mut c_void, *mut c_void, NSRect, i8) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_ptr: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_isize: unsafe extern "C" fn(*mut c_void, *mut c_void, isize) =
        std::mem::transmute(objc_msgSend as *const ());

    let popup = init_popup(
        msg0(cls("NSPopUpButton"), sel("alloc")),
        sel("initWithFrame:pullsDown:"),
        rect,
        0,
    );
    for &item in items {
        msg1_ptr(popup, sel("addItemWithTitle:"), nsstring(item));
    }
    msg1_isize(popup, sel("selectItemAtIndex:"), selected_idx.max(0));
    msg1_ptr(popup, sel("setTarget:"), delegate);
    msg1_ptr(popup, sel("setAction:"), action_sel);
    msg1_ptr(content_view, sel("addSubview:"), popup);
    popup
}

#[cfg(target_os = "macos")]
unsafe fn create_custom_box(parent_view: *mut c_void, rect: NSRect) -> (*mut c_void, *mut c_void) {
    let msg0: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let init_frame: unsafe extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_isize: unsafe extern "C" fn(*mut c_void, *mut c_void, isize) =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_f64: unsafe extern "C" fn(*mut c_void, *mut c_void, f64) =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_ptr: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) =
        std::mem::transmute(objc_msgSend as *const ());

    let bx = init_frame(msg0(cls("NSBox"), sel("alloc")), sel("initWithFrame:"), rect);
    msg1_isize(bx, sel("setBoxType:"), 4); // NSBoxCustom
    msg1_isize(bx, sel("setTitlePosition:"), 0); // NSNoTitle
    msg1_f64(bx, sel("setCornerRadius:"), 4.0);
    msg1_f64(bx, sel("setBorderWidth:"), 1.0);
    let fill_color = msg0(cls("NSColor"), sel("controlBackgroundColor"));
    if !fill_color.is_null() {
        msg1_ptr(bx, sel("setFillColor:"), fill_color);
    }
    let border_color = msg0(cls("NSColor"), sel("separatorColor"));
    if !border_color.is_null() {
        msg1_ptr(bx, sel("setBorderColor:"), border_color);
    }
    msg1_ptr(parent_view, sel("addSubview:"), bx);
    let box_cv = msg0(bx, sel("contentView"));
    (bx, box_cv)
}

#[cfg(target_os = "macos")]
unsafe fn add_bottom_icon_button(
    content_view: *mut c_void,
    target: *mut c_void,
    title: &str,
    image_name: &str,
    action_sel: *mut c_void,
    rect: NSRect,
) -> *mut c_void {
    let msg0: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let init_frame: unsafe extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_ptr: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_ptr_ret: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_isize: unsafe extern "C" fn(*mut c_void, *mut c_void, isize) =
        std::mem::transmute(objc_msgSend as *const ());

    let btn = init_frame(msg0(cls("NSButton"), sel("alloc")), sel("initWithFrame:"), rect);
    msg1_ptr(btn, sel("setTitle:"), nsstring(title));
    msg1_isize(btn, sel("setBezelStyle:"), 6); // NSBezelStyleSmallSquare
    let img = msg1_ptr_ret(cls("NSImage"), sel("imageNamed:"), nsstring(image_name));
    if !img.is_null() {
        msg1_ptr(btn, sel("setImage:"), img);
        msg1_isize(btn, sel("setImagePosition:"), 2); // NSImageLeft
    }
    msg1_ptr(btn, sel("setTarget:"), target);
    msg1_ptr(btn, sel("setAction:"), action_sel);
    msg1_ptr(content_view, sel("addSubview:"), btn);
    btn
}

/// Builds the native Cocoa Control Panel (`Bảng điều khiển`) `NSWindow` matching `Main.storyboard` and `openkey-main-control-2.png`.
#[cfg(target_os = "macos")]
pub unsafe fn build_control_panel_window(ui: &AppUiController) -> *mut c_void {
    let (win, cv, delegate) = create_base_window("OpenKey 2.0.5 - Bộ gõ Tiếng Việt", 560.0, 490.0);
    ACTIVE_CP_WINDOW.store(win, Ordering::SeqCst);
    let cfg = &ui.handler.engine.config;

    // 1. Top "Điều khiển" section
    add_label(cv, "Điều khiển", NSRect::new(36.0, 459.0, 94.0, 17.0));
    let (_top_box, _top_cv) = create_custom_box(cv, NSRect::new(20.0, 348.0, 520.0, 112.0));

    add_label(cv, "Kiểu gõ:", NSRect::new(38.0, 430.0, 64.0, 17.0));
    add_popup_button(
        cv,
        delegate,
        &["Telex", "VNI", "Simple Telex 1", "Simple Telex 2"],
        cfg.input_type as isize,
        sel("onPopupInputTypeChanged:"),
        NSRect::new(130.0, 424.0, 129.0, 25.0),
    );

    add_label(cv, "Bảng mã:", NSRect::new(295.0, 429.0, 68.0, 17.0));
    add_popup_button(
        cv,
        delegate,
        &openkey_macos::config::TABLE_CODE_NAMES,
        cfg.code_table as isize,
        sel("onPopupCodeChanged:"),
        NSRect::new(375.0, 424.0, 128.0, 25.0),
    );

    add_label(cv, "Phím chuyển:", NSRect::new(38.0, 395.0, 86.0, 17.0));
    add_checkbox(cv, delegate, "⌃", (cfg.switch_key_status & 0x100) != 0, 101, NSRect::new(130.0, 394.0, 42.0, 18.0));
    add_checkbox(cv, delegate, "⌥", (cfg.switch_key_status & 0x200) != 0, 102, NSRect::new(185.0, 394.0, 42.0, 18.0));
    add_checkbox(cv, delegate, "⌘", (cfg.switch_key_status & 0x400) != 0, 103, NSRect::new(240.0, 394.0, 42.0, 18.0));
    add_checkbox(cv, delegate, "⇧", (cfg.switch_key_status & 0x800) != 0, 104, NSRect::new(295.0, 394.0, 42.0, 18.0));

    let key_char = ((cfg.switch_key_status as u32) >> 24) & 0xFF;
    let key_code = (cfg.switch_key_status & 0xFF) as u16;
    let key_text = if key_code == 49 || key_char == 32 || key_char == 49 {
        "Space".to_string()
    } else if key_char != 0 && key_char != 0xFE {
        ((key_char as u8 as char).to_ascii_uppercase()).to_string()
    } else {
        "Space".to_string()
    };
    add_bezel_textfield(cv, &key_text, NSRect::new(350.0, 390.0, 60.0, 25.0));
    add_checkbox(cv, delegate, "Kêu beep", (cfg.switch_key_status & 0x8000) != 0, 105, NSRect::new(423.0, 394.0, 86.0, 18.0));

    add_label(cv, "Chế độ gõ:", NSRect::new(38.0, 363.0, 75.0, 17.0));
    let r_viet = add_radio_button(cv, delegate, "Tiếng Việt", cfg.language == 1, 1, NSRect::new(130.0, 362.0, 95.0, 18.0));
    let r_eng = add_radio_button(cv, delegate, "English", cfg.language == 0, 0, NSRect::new(295.0, 362.0, 80.0, 18.0));
    ACTIVE_CP_RADIO_VIET.store(r_viet, Ordering::SeqCst);
    ACTIVE_CP_RADIO_ENG.store(r_eng, Ordering::SeqCst);

    // 2. Four Tab Content Boxes at (20, 80, 520, 241)
    let tab_rect = NSRect::new(20.0, 80.0, 520.0, 241.0);

    // Tab 0: Bộ gõ
    let (box0, cv0) = create_custom_box(cv, tab_rect);
    add_checkbox(cv0, delegate, "Đặt dấu oà, uý (thay vì òa, úy)", cfg.use_modern_orthography != 0, 1, NSRect::new(18.0, 195.0, 240.0, 18.0));
    add_checkbox(cv0, delegate, "Sửa lỗi gợi ý (trình duyệt, Excel,...)", cfg.fix_recommend_browser != 0, 5, NSRect::new(18.0, 160.0, 250.0, 18.0));
    add_checkbox(cv0, delegate, "Viết Hoa chữ cái đầu câu", cfg.upper_case_first_char != 0, 10, NSRect::new(18.0, 125.0, 220.0, 18.0));
    add_checkbox(cv0, delegate, "Chuyển chế độ thông minh", cfg.use_smart_switch_key != 0, 9, NSRect::new(18.0, 90.0, 220.0, 18.0));
    add_checkbox(cv0, delegate, "Tự ghi nhớ bảng mã theo ứng dụng", cfg.remember_code != 0, 14, NSRect::new(18.0, 55.0, 250.0, 18.0));
    add_checkbox(cv0, delegate, "Tắt tiếng Việt khi bộ gõ hệ thống khác tiếng Anh", cfg.other_language != 0, 15, NSRect::new(18.0, 23.0, 340.0, 18.0));

    add_checkbox(cv0, delegate, "Kiểm tra chính tả", cfg.check_spelling != 0, 2, NSRect::new(278.0, 195.0, 160.0, 18.0));
    add_checkbox(cv0, delegate, "Tự khôi phục phím với từ sai", cfg.restore_if_wrong_spelling != 0, 3, NSRect::new(278.0, 160.0, 210.0, 18.0));
    add_checkbox(cv0, delegate, "Cho phép \"z w j f\" làm phụ âm", cfg.allow_consonant_zfwj != 0, 11, NSRect::new(278.0, 125.0, 225.0, 18.0));
    add_checkbox(cv0, delegate, "Tạm tắt chính tả bằng phím ⌃", cfg.temp_off_spelling != 0, 16, NSRect::new(278.0, 90.0, 225.0, 18.0));
    add_checkbox(cv0, delegate, "Tạm tắt OpenKey bằng phím ⌘", cfg.temp_off_open_key != 0, 17, NSRect::new(278.0, 55.0, 225.0, 18.0));

    // Tab 1: Gõ tắt (matches openkey-main-control-2.png)
    let (box1, cv1) = create_custom_box(cv, tab_rect);
    add_checkbox(cv1, delegate, "Cho phép gõ tắt", cfg.use_macro != 0, 6, NSRect::new(18.0, 175.0, 160.0, 18.0));
    add_checkbox(cv1, delegate, "Gõ tắt cả khi tắt gõ tiếng Việt", cfg.use_macro_in_english_mode != 0, 7, NSRect::new(18.0, 140.0, 215.0, 18.0));
    add_checkbox(cv1, delegate, "Tự động viết hoa theo phím tắt", cfg.auto_caps_macro != 0, 8, NSRect::new(18.0, 105.0, 225.0, 18.0));
    {
        let msg3_btn: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void) -> *mut c_void =
            std::mem::transmute(objc_msgSend as *const ());
        let msg1_rect: unsafe extern "C" fn(*mut c_void, *mut c_void, NSRect) =
            std::mem::transmute(objc_msgSend as *const ());
        let msg1_ptr: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) =
            std::mem::transmute(objc_msgSend as *const ());
        let macro_btn = msg3_btn(
            cls("NSButton"),
            sel("buttonWithTitle:target:action:"),
            nsstring("Bảng gõ tắt..."),
            delegate,
            sel("onMacroSelected:"),
        );
        msg1_rect(macro_btn, sel("setFrame:"), NSRect::new(14.0, 55.0, 122.0, 32.0));
        msg1_ptr(cv1, sel("addSubview:"), macro_btn);
    }
    add_checkbox(cv1, delegate, "Gõ nhanh (cc=ch, gg=gi, kk=kh, nn=ng,...", cfg.quick_telex != 0, 4, NSRect::new(243.0, 175.0, 267.0, 18.0));
    add_checkbox(cv1, delegate, "Gõ tắt phụ âm đầu: f→ph, j→gi, w→qu", cfg.quick_start_consonant != 0, 12, NSRect::new(243.0, 140.0, 262.0, 18.0));
    add_checkbox(cv1, delegate, "Gõ tắt phụ âm cuối: g→ng, h→nh, k→ch", cfg.quick_end_consonant != 0, 13, NSRect::new(243.0, 105.0, 270.0, 18.0));

    // Tab 2: Hệ thống
    let (box2, cv2) = create_custom_box(cv, tab_rect);
    add_checkbox(cv2, delegate, "Hiện biểu tượng trên thanh Dock", ui.config.show_on_dock, 25, NSRect::new(18.0, 175.0, 235.0, 18.0));
    add_checkbox(cv2, delegate, "Bật bảng này khi khởi động", ui.config.show_ui_on_startup, 26, NSRect::new(18.0, 140.0, 210.0, 18.0));
    add_checkbox(cv2, delegate, "Biểu tượng hiện đại trên thanh menu", ui.config.gray_icon, 18, NSRect::new(18.0, 105.0, 255.0, 18.0));
    add_checkbox(cv2, delegate, "Khởi động cùng macOS", ui.config.run_on_startup, 19, NSRect::new(18.0, 70.0, 185.0, 18.0));
    add_checkbox(cv2, delegate, "Tương thích Telex trên các Layout khác (Dvorak, Coleman, ...)", ui.config.perform_layout_compat, 27, NSRect::new(18.0, 35.0, 410.0, 18.0));
    add_checkbox(cv2, delegate, "Sửa lỗi trên Chromium (beta)", ui.config.fix_chromium_browser, 28, NSRect::new(283.0, 175.0, 224.0, 18.0));
    add_checkbox(cv2, delegate, "Gửi từng phím (bật nếu bị lỗi)", ui.config.send_key_step_by_step, 29, NSRect::new(283.0, 140.0, 215.0, 18.0));
    add_checkbox(cv2, delegate, "Kiểm tra bản mới lúc khởi động", !ui.config.dont_check_update, 30, NSRect::new(283.0, 105.0, 230.0, 18.0));

    // Tab 3: Thông tin
    let (box3, cv3) = create_custom_box(cv, tab_rect);
    add_label(cv3, "OpenKey - Bộ gõ Tiếng Việt nguồn mở đa nền tảng", NSRect::new(78.0, 185.0, 394.0, 20.0));
    add_label(cv3, "Trang chủ: https://open-key.org", NSRect::new(78.0, 155.0, 340.0, 17.0));
    add_label(cv3, "Facebook: https://www.facebook.com/OpenKeyVN", NSRect::new(78.0, 132.0, 360.0, 17.0));
    add_label(cv3, "Phiên bản 2.0.5 (build 62) - Rust macOS Port", NSRect::new(78.0, 102.0, 418.0, 18.0));
    add_label(
        cv3,
        "OpenKey cho macOS là phần mềm mã nguồn mở và được phát hành miễn phí.",
        NSRect::new(78.0, 58.0, 422.0, 36.0),
    );
    add_label(cv3, "Mai Vũ Tuyên © 2019 - maivutuyen.91@gmail.com", NSRect::new(78.0, 22.0, 340.0, 17.0));

    ACTIVE_CP_TAB_BOXES[0].store(box0, Ordering::SeqCst);
    ACTIVE_CP_TAB_BOXES[1].store(box1, Ordering::SeqCst);
    ACTIVE_CP_TAB_BOXES[2].store(box2, Ordering::SeqCst);
    ACTIVE_CP_TAB_BOXES[3].store(box3, Ordering::SeqCst);

    // 3. Four Tab Header Buttons at y = 303.0
    let msg0: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let init_frame: unsafe extern "C" fn(*mut c_void, *mut c_void, NSRect) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_ptr: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_isize: unsafe extern "C" fn(*mut c_void, *mut c_void, isize) =
        std::mem::transmute(objc_msgSend as *const ());

    for (idx, (title, x)) in [
        ("Bộ gõ", 56.0),
        ("Gõ tắt", 165.0),
        ("Hệ thống", 274.0),
        ("Thông tin", 383.0),
    ]
    .iter()
    .enumerate()
    {
        let btn = init_frame(
            msg0(cls("NSButton"), sel("alloc")),
            sel("initWithFrame:"),
            NSRect::new(*x, 303.0, 122.0, 32.0),
        );
        msg1_ptr(btn, sel("setTitle:"), nsstring(title));
        msg1_isize(btn, sel("setBezelStyle:"), 1); // NSBezelStyleRounded
        msg1_isize(btn, sel("setButtonType:"), 2); // NSButtonTypePushOnPushOff
        msg1_isize(btn, sel("setTag:"), idx as isize);
        msg1_ptr(btn, sel("setTarget:"), delegate);
        msg1_ptr(btn, sel("setAction:"), sel("onTabButtonClicked:"));
        msg1_ptr(cv, sel("addSubview:"), btn);
        ACTIVE_CP_TAB_BUTTONS[idx].store(btn, Ordering::SeqCst);
    }

    show_control_panel_tab(0);

    // 4. Three Bottom Action Buttons (Kết thúc, Mặc định, OK)
    let app = msg0(cls("NSApplication"), sel("sharedApplication"));
    add_bottom_icon_button(
        cv,
        app,
        "Kết thúc",
        "ExitButton",
        sel("terminate:"),
        NSRect::new(57.0, 20.0, 120.0, 41.0),
    );
    add_bottom_icon_button(
        cv,
        delegate,
        "Mặc định",
        "ThumbUp",
        sel("onDefaultConfigClicked:"),
        NSRect::new(220.0, 20.0, 120.0, 41.0),
    );
    add_bottom_icon_button(
        cv,
        delegate,
        "OK",
        "OK",
        sel("onOKClicked:"),
        NSRect::new(382.0, 19.0, 120.0, 42.0),
    );

    win
}

#[cfg(target_os = "macos")]
pub unsafe fn cocoa_window_content_size(win: *mut c_void) -> (usize, usize) {
    if win.is_null() {
        return (0, 0);
    }
    let msg0: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let msg_frame: unsafe extern "C" fn(*mut c_void, *mut c_void) -> NSRect =
        std::mem::transmute(objc_msgSend as *const ());
    let cv = msg0(win, sel("contentView"));
    if cv.is_null() {
        return (0, 0);
    }
    let frame = msg_frame(cv, sel("frame"));
    (frame.size.width.round() as usize, frame.size.height.round() as usize)
}

/// Builds the native Cocoa Macro Editor (`Bảng gõ tắt`) `NSWindow` matching `MacroViewController.mm`.
#[cfg(target_os = "macos")]
pub unsafe fn build_macro_window(ui: &AppUiController) -> *mut c_void {
    let (win, cv, delegate) = create_base_window("Bảng gõ tắt - OpenKey", 460.0, 320.0);
    let macros = ui.handler.engine.macros.get_all_macros();
    add_label(
        cv,
        &format!("Danh sách gõ tắt ({} mục):", macros.len()),
        NSRect::new(20.0, 275.0, 420.0, 24.0),
    );
    let preview = macros
        .iter()
        .take(8)
        .map(|(_, k, v)| format!("• {}  →  {}", k, v))
        .collect::<Vec<_>>()
        .join("\n");
    add_label(
        cv,
        if preview.is_empty() {
            "(Chưa có dữ liệu gõ tắt)"
        } else {
            &preview
        },
        NSRect::new(20.0, 100.0, 420.0, 165.0),
    );
    add_checkbox(
        cv,
        delegate,
        "Tự động đổi chữ hoa theo phím tắt",
        ui.handler.engine.config.auto_caps_macro != 0,
        8,
        NSRect::new(20.0, 55.0, 320.0, 24.0),
    );
    add_checkbox(
        cv,
        delegate,
        "Bật gõ tắt",
        ui.handler.engine.config.use_macro != 0,
        6,
        NSRect::new(20.0, 25.0, 200.0, 24.0),
    );
    win
}

/// Builds the native Cocoa Convert Tool (`Công cụ chuyển mã`) `NSWindow` matching `ConvertToolViewController.mm`.
#[cfg(target_os = "macos")]
pub unsafe fn build_convert_tool_window(ui: &AppUiController) -> *mut c_void {
    let (win, cv, delegate) = create_base_window("Công cụ chuyển mã - OpenKey", 420.0, 280.0);
    let ct = &ui.handler.engine.convert_tool;
    add_label(
        cv,
        &format!(
            "Nguồn: {}  →  Đích: {}",
            openkey_macos::config::TABLE_CODE_NAMES
                .get(ct.from_code as usize)
                .copied()
                .unwrap_or("Unicode"),
            openkey_macos::config::TABLE_CODE_NAMES
                .get(ct.to_code as usize)
                .copied()
                .unwrap_or("Unicode"),
        ),
        NSRect::new(20.0, 235.0, 380.0, 24.0),
    );
    add_checkbox(cv, delegate, "Loại bỏ dấu câu (Xoá dấu)", ct.remove_mark, 20, NSRect::new(20.0, 195.0, 300.0, 24.0));
    add_checkbox(cv, delegate, "Chuyển sang chữ HOA", ct.to_all_caps, 21, NSRect::new(20.0, 160.0, 300.0, 24.0));
    add_checkbox(cv, delegate, "Chuyển sang chữ thường", ct.to_all_non_caps, 22, NSRect::new(20.0, 125.0, 300.0, 24.0));
    add_checkbox(cv, delegate, "Viết hoa chữ đầu câu", ct.to_caps_first_letter, 23, NSRect::new(20.0, 90.0, 300.0, 24.0));
    add_checkbox(cv, delegate, "Viết hoa chữ đầu mỗi từ", ct.to_caps_each_word, 24, NSRect::new(20.0, 55.0, 300.0, 24.0));
    win
}

/// Builds the native Cocoa About (`Giới thiệu`) `NSWindow` matching `AboutViewController.m`.
#[cfg(target_os = "macos")]
pub unsafe fn build_about_window(_ui: &AppUiController) -> *mut c_void {
    let (win, cv, _delegate) = create_base_window("Giới thiệu OpenKey", 380.0, 200.0);
    add_label(cv, "OpenKey 2.0.5 (Rust macOS Port)", NSRect::new(20.0, 150.0, 340.0, 24.0));
    add_label(cv, "Bộ gõ Tiếng Việt nguồn mở cho macOS", NSRect::new(20.0, 115.0, 340.0, 24.0));
    add_label(cv, "Tác giả bản gốc: Mai Vũ Tuyên | Giấy phép: GPL-3.0", NSRect::new(20.0, 80.0, 340.0, 24.0));
    win
}

#[cfg(target_os = "macos")]
pub unsafe fn cocoa_window_title(win: *mut c_void) -> String {
    if win.is_null() {
        return String::new();
    }
    let msg0: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let utf8: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *const c_char =
        std::mem::transmute(objc_msgSend as *const ());
    let title_ns = msg0(win, sel("title"));
    if title_ns.is_null() {
        return String::new();
    }
    let c_ptr = utf8(title_ns, sel("UTF8String"));
    if c_ptr.is_null() {
        String::new()
    } else {
        std::ffi::CStr::from_ptr(c_ptr).to_string_lossy().into_owned()
    }
}

#[cfg(target_os = "macos")]
pub unsafe fn cocoa_window_subview_count(win: *mut c_void) -> usize {
    if win.is_null() {
        return 0;
    }
    let msg0: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let msg0_usize: unsafe extern "C" fn(*mut c_void, *mut c_void) -> usize =
        std::mem::transmute(objc_msgSend as *const ());
    let cv = msg0(win, sel("contentView"));
    if cv.is_null() {
        return 0;
    }
    let subviews = msg0(cv, sel("subviews"));
    if subviews.is_null() {
        0
    } else {
        msg0_usize(subviews, sel("count"))
    }
}

/// Initializes the native Cocoa `NSStatusBar` status item and menu matching `createStatusBarMenu` in `AppDelegate.m`.
#[cfg(target_os = "macos")]
pub unsafe fn populate_status_item(status_item: *mut c_void, ui: &AppUiController) {
    let msg0: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
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

    let delegate_cls = register_objc_delegate_class();
    let delegate_target = msg0(msg0(delegate_cls, sel("alloc")), sel("init"));
    let snap = ui.menu_snapshot();

    let button = msg0(status_item, sel("button"));
    if !button.is_null() {
        let img = msg1_ptr(cls("NSImage"), sel("imageNamed:"), nsstring(snap.status_icon_name));
        if !img.is_null() {
            msg1_bool(img, sel("setTemplate:"), if snap.is_template_icon { 1 } else { 0 });
            msg1_ptr(button, sel("setImage:"), img);
            msg1_ptr(button, sel("setTitle:"), nsstring(""));
        } else {
            msg1_ptr(button, sel("setTitle:"), nsstring(snap.status_title));
        }
    }
    let menu_alloc = msg0(cls("NSMenu"), sel("alloc"));
    let menu = msg1_ptr(menu_alloc, sel("initWithTitle:"), nsstring("OpenKey"));
    msg1_bool(menu, sel("setAutoenablesItems:"), 0);

    let item_vn = msg3_item(
        menu,
        sel("addItemWithTitle:action:keyEquivalent:"),
        nsstring("Bật Tiếng Việt"),
        sel("onInputMethodSelected:"),
        nsstring(""),
    );
    msg1_ptr(item_vn, sel("setTarget:"), delegate_target);
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
            sel("onCodeSelected:"),
            nsstring(""),
        );
        msg1_ptr(item, sel("setTarget:"), delegate_target);
        msg1_isize(item, sel("setTag:"), idx as isize);
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
            sel("onInputTypeSelected:"),
            nsstring(""),
        );
        msg1_ptr(item, sel("setTarget:"), delegate_target);
        msg1_isize(item, sel("setTag:"), idx as isize);
        msg1_isize(
            item,
            sel("setState:"),
            if snap.selected_input_type == idx { 1 } else { 0 },
        );
    }

    let sep3 = msg0(cls("NSMenuItem"), sel("separatorItem"));
    msg1_ptr(menu, sel("addItem:"), sep3);

    let item_convert_tool = msg3_item(
        menu,
        sel("addItemWithTitle:action:keyEquivalent:"),
        nsstring("Công cụ chuyển mã..."),
        sel("onConvertToolSelected:"),
        nsstring(""),
    );
    msg1_ptr(item_convert_tool, sel("setTarget:"), delegate_target);

    let item_quick_convert = msg3_item(
        menu,
        sel("addItemWithTitle:action:keyEquivalent:"),
        nsstring(&snap.quick_convert_title),
        sel("onQuickConvert:"),
        nsstring(""),
    );
    msg1_ptr(item_quick_convert, sel("setTarget:"), delegate_target);

    let sep4 = msg0(cls("NSMenuItem"), sel("separatorItem"));
    msg1_ptr(menu, sel("addItem:"), sep4);

    for (title, action) in [
        ("Bảng điều khiển...", "onControlPanelSelected:"),
        ("Gõ tắt...", "onMacroSelected:"),
        ("Giới thiệu", "onAboutSelected:"),
    ] {
        let item = msg3_item(
            menu,
            sel("addItemWithTitle:action:keyEquivalent:"),
            nsstring(title),
            sel(action),
            nsstring(""),
        );
        msg1_ptr(item, sel("setTarget:"), delegate_target);
    }

    let sep5 = msg0(cls("NSMenuItem"), sel("separatorItem"));
    msg1_ptr(menu, sel("addItem:"), sep5);

    msg3_item(
        menu,
        sel("addItemWithTitle:action:keyEquivalent:"),
        nsstring("Thoát"),
        sel("terminate:"),
        nsstring("q"),
    );

    msg1_ptr(status_item, sel("setMenu:"), menu);
}

/// Initializes the native Cocoa `NSStatusBar` status item and menu matching `createStatusBarMenu` in `AppDelegate.m`.
#[cfg(target_os = "macos")]
pub unsafe fn setup_cocoa_status_bar(ui: &AppUiController) -> *mut c_void {
    let msg0: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let msg1_f64: unsafe extern "C" fn(*mut c_void, *mut c_void, f64) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let set_policy: unsafe extern "C" fn(*mut c_void, *mut c_void, isize) -> i8 =
        std::mem::transmute(objc_msgSend as *const ());

    let app = msg0(cls("NSApplication"), sel("sharedApplication"));
    let policy: isize = if ui.config.show_on_dock { 0 } else { 1 };
    set_policy(app, sel("setActivationPolicy:"), policy);

    let msg1_ptr: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void) =
        std::mem::transmute(objc_msgSend as *const ());
    let delegate_cls = register_objc_delegate_class();
    let app_delegate = msg0(msg0(delegate_cls, sel("alloc")), sel("init"));
    msg1_ptr(app, sel("setDelegate:"), app_delegate);
    let status_bar = msg0(cls("NSStatusBar"), sel("systemStatusBar"));
    let status_item = msg1_f64(status_bar, sel("statusItemWithLength:"), -1.0);
    populate_status_item(status_item, ui);
    status_item
}

#[cfg(target_os = "macos")]
pub unsafe fn run_cocoa_app_loop() {
    let msg0: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    let app = msg0(cls("NSApplication"), sel("sharedApplication"));
    msg0(app, sel("run"));
}
