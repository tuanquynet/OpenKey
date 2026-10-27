use openkey_app::ui::{
    build_about_window, build_control_panel_window, build_convert_tool_window, build_macro_window,
    cocoa_control_panel_active_tab, cocoa_control_panel_tab_is_highlighted,
    cocoa_control_panel_tab_text_padding_diff, cocoa_macro_add_button_title,
    cocoa_macro_cell_text, cocoa_macro_field_values, cocoa_macro_row_count,
    cocoa_macro_select_row, cocoa_macro_set_fields, cocoa_window_content_size,
    cocoa_window_subview_count, cocoa_window_title, macro_add_action, macro_delete_action,
    macro_export_to_path, macro_import_from_path, setup_cocoa_status_bar,
    show_control_panel_tab, AppUiController,
};

fn main() {
    let mut ui = AppUiController::with_domain("com.tuyenmai.openkey.rust_window_test");
    ui.load_default_config();
    ui.handler.engine.macros.add_macro("vn", "Việt Nam", 0);
    ui.save_preferences();

    unsafe {
        let status_item = setup_cocoa_status_bar(&ui);
        assert!(!status_item.is_null());

        let cp_win = build_control_panel_window(&ui);
        assert!(!cp_win.is_null());
        assert_eq!(
            cocoa_window_title(cp_win),
            "OpenKey 2.1.0 - Bộ gõ Tiếng Việt"
        );
        assert_eq!(cocoa_window_content_size(cp_win), (560, 490));
        assert!(cocoa_window_subview_count(cp_win) >= 20);

        // Verify 4 tabs (0: Bộ gõ, 1: Gõ tắt, 2: Hệ thống, 3: Thông tin) switch cleanly
        assert_eq!(cocoa_control_panel_active_tab(), 0);
        assert!(cocoa_control_panel_tab_is_highlighted(0));
        assert!(!cocoa_control_panel_tab_is_highlighted(1));
        let (v_diff, h_diff) = cocoa_control_panel_tab_text_padding_diff(0);
        assert!(
            v_diff <= 3 && h_diff <= 3,
            "expected centered tab text (v_diff <= 3, h_diff <= 3), got v_diff={}, h_diff={}",
            v_diff,
            h_diff
        );
        for tab_idx in [1, 2, 3, 0] {
            show_control_panel_tab(tab_idx);
            assert_eq!(cocoa_control_panel_active_tab(), tab_idx);
            for i in 0..4 {
                assert_eq!(
                    cocoa_control_panel_tab_is_highlighted(i),
                    i == tab_idx,
                    "tab {} highlight mismatch when active tab is {}",
                    i,
                    tab_idx
                );
            }
        }

        let macro_win = build_macro_window(&ui);
        assert!(!macro_win.is_null());
        assert_eq!(cocoa_window_title(macro_win), "Bảng gõ tắt - OpenKey");
        assert_eq!(cocoa_window_content_size(macro_win), (600, 472));
        // Subviews: 2 labels + 2 textfields + 2 icon buttons + 1 scrollview + 2 push buttons + 1 checkbox = 10
        assert!(cocoa_window_subview_count(macro_win) >= 10);

        // 1. Initial state: "vn" was preloaded
        assert_eq!(cocoa_macro_row_count(), 1);
        assert_eq!(cocoa_macro_cell_text(0, 0), "vn");
        assert_eq!(cocoa_macro_cell_text(0, 1), "Việt Nam");
        assert_eq!(cocoa_macro_add_button_title(), "Thêm");

        // 2. Typing existing key switches button to "Sửa", new key switches to "Thêm"
        cocoa_macro_set_fields("vn", "Việt Nam!");
        assert_eq!(cocoa_macro_add_button_title(), "Sửa");
        cocoa_macro_set_fields("ok", "Đồng ý");
        assert_eq!(cocoa_macro_add_button_title(), "Thêm");

        // 3. Trigger Add via macro_add_action()
        macro_add_action().expect("add 'ok' should succeed");
        assert_eq!(cocoa_macro_row_count(), 2);
        // Inputs are cleared after addition, button title reset to "Thêm"
        assert_eq!(cocoa_macro_field_values(), (String::new(), String::new()));
        assert_eq!(cocoa_macro_add_button_title(), "Thêm");
        // Rows sorted by MacroStore keycode key ("vn" [9, 45] < "ok" [31, 40])
        assert_eq!(cocoa_macro_cell_text(0, 0), "vn");
        assert_eq!(cocoa_macro_cell_text(0, 1), "Việt Nam");
        assert_eq!(cocoa_macro_cell_text(1, 0), "ok");
        assert_eq!(cocoa_macro_cell_text(1, 1), "Đồng ý");

        // 4. Selecting row 0 ("vn") populates inputs and sets button to "Sửa"
        cocoa_macro_select_row(0);
        assert_eq!(
            cocoa_macro_field_values(),
            ("vn".to_string(), "Việt Nam".to_string())
        );
        assert_eq!(cocoa_macro_add_button_title(), "Sửa");

        // 5. Updating an existing entry in-place
        cocoa_macro_set_fields("vn", "Cộng hòa Xã hội Chủ nghĩa Việt Nam");
        macro_add_action().expect("update 'vn' should succeed");
        assert_eq!(cocoa_macro_row_count(), 2);
        assert_eq!(
            cocoa_macro_cell_text(0, 1),
            "Cộng hòa Xã hội Chủ nghĩa Việt Nam"
        );

        // 6. Delete macro: selecting "ok" (row 1) and triggering delete
        cocoa_macro_select_row(1);
        assert_eq!(cocoa_macro_field_values().0, "ok");
        macro_delete_action().expect("delete 'ok' should succeed");
        assert_eq!(cocoa_macro_row_count(), 1);
        assert_eq!(cocoa_macro_cell_text(0, 0), "vn");
        assert_eq!(cocoa_macro_field_values(), (String::new(), String::new()));

        // 7. Validation: empty name/content rejected
        cocoa_macro_set_fields("", "Không có phím");
        assert_eq!(
            macro_add_action().unwrap_err(),
            "Bạn hãy nhập từ cần gõ tắt!"
        );
        cocoa_macro_set_fields("test", "");
        assert_eq!(
            macro_add_action().unwrap_err(),
            "Bạn hãy nhập từ cần gõ tắt!"
        );
        cocoa_macro_set_fields("", "");
        assert_eq!(
            macro_delete_action().unwrap_err(),
            "Bạn hãy chọn từ cần xoá!"
        );

        // 8. File Export & Import roundtrip via engine-compatible file format
        let tmp_file = std::env::temp_dir().join(format!(
            "openkey_macro_test_{}.txt",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        // Export single "vn" macro
        macro_export_to_path(&tmp_file).expect("export should succeed");
        let exported_content = std::fs::read_to_string(&tmp_file).unwrap();
        assert!(exported_content.contains("vn:Cộng hòa Xã hội Chủ nghĩa Việt Nam"));

        // Add extra item, then import with keep_existing = false to restore
        cocoa_macro_set_fields("temp", "Tạm thời");
        macro_add_action().unwrap();
        assert_eq!(cocoa_macro_row_count(), 2);

        macro_import_from_path(&tmp_file, false).expect("import should succeed");
        assert_eq!(cocoa_macro_row_count(), 1);
        assert_eq!(cocoa_macro_cell_text(0, 0), "vn");

        // Clean up temp file
        let _ = std::fs::remove_file(tmp_file);

        // 9. Verify persistence back into AppUiController reload
        let reloaded = AppUiController::load_from_domain("com.tuyenmai.openkey.rust_window_test");
        assert_eq!(reloaded.macro_entries().len(), 1);
        assert_eq!(reloaded.macro_entries()[0].0, "vn");

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
