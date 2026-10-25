use openkey_app::ui::{
    build_about_window, build_control_panel_window, build_convert_tool_window, build_macro_window,
    cocoa_control_panel_active_tab, cocoa_window_content_size, cocoa_window_subview_count,
    cocoa_window_title, setup_cocoa_status_bar, show_control_panel_tab, AppUiController,
};

fn main() {
    let mut ui = AppUiController::with_domain("com.tuyenmai.openkey.rust_window_test");
    ui.load_default_config();
    ui.handler.engine.macros.add_macro("vn", "Việt Nam", 0);

    unsafe {
        let status_item = setup_cocoa_status_bar(&ui);
        assert!(!status_item.is_null());

        let cp_win = build_control_panel_window(&ui);
        assert!(!cp_win.is_null());
        assert_eq!(
            cocoa_window_title(cp_win),
            "OpenKey 2.0.5 - Bộ gõ Tiếng Việt"
        );
        assert_eq!(cocoa_window_content_size(cp_win), (560, 490));
        assert!(cocoa_window_subview_count(cp_win) >= 20);

        // Verify 4 tabs (0: Bộ gõ, 1: Gõ tắt, 2: Hệ thống, 3: Thông tin) switch cleanly
        assert_eq!(cocoa_control_panel_active_tab(), 0);
        for tab_idx in [1, 2, 3, 0] {
            show_control_panel_tab(tab_idx);
            assert_eq!(cocoa_control_panel_active_tab(), tab_idx);
        }

        let macro_win = build_macro_window(&ui);
        assert!(!macro_win.is_null());
        assert_eq!(cocoa_window_title(macro_win), "Bảng gõ tắt - OpenKey");
        assert!(cocoa_window_subview_count(macro_win) >= 4);

        let convert_win = build_convert_tool_window(&ui);
        assert!(!convert_win.is_null());
        assert_eq!(
            cocoa_window_title(convert_win),
            "Công cụ chuyển mã - OpenKey"
        );
        assert!(cocoa_window_subview_count(convert_win) >= 6);

        let about_win = build_about_window(&ui);
        assert!(!about_win.is_null());
        assert_eq!(cocoa_window_title(about_win), "Giới thiệu OpenKey");
        assert!(cocoa_window_subview_count(about_win) >= 3);
    }

    println!("running 1 test\ntest cocoa_main_thread_windows_and_status_bar ... ok\n\ntest result: ok. 1 passed; 0 failed");
}
