pub const NICE_SPACE_APPS: &[&str] = &["com.sublimetext.3", "com.sublimetext.2"];

pub const UNICODE_COMPOUND_APPS: &[&str] = &[
    "com.apple.",
    "com.google.Chrome",
    "com.brave.Browser",
    "com.microsoft.edgemac.Dev",
    "com.microsoft.edgemac.Beta",
    "com.microsoft.Edge.Dev",
    "com.microsoft.Edge",
];

pub const RECOMMEND_WORKAROUND_DISABLED_APPS: &[&str] = &["com.apple.Spotlight"];

#[inline]
pub fn contain_unicode_compound_app(top_app: &str) -> bool {
    if top_app.is_empty() {
        return false;
    }
    UNICODE_COMPOUND_APPS
        .iter()
        .any(|&prefix| top_app.starts_with(prefix) || top_app == prefix)
}

#[inline]
pub fn is_exact_unicode_compound_app(top_app: &str) -> bool {
    UNICODE_COMPOUND_APPS.contains(&top_app)
}

#[inline]
pub fn should_use_recommend_workaround(
    fix_recommend_browser: bool,
    spotlight_visible: bool,
    top_app: &str,
) -> bool {
    if !fix_recommend_browser {
        return false;
    }
    if spotlight_visible {
        return false;
    }
    if top_app.is_empty() {
        return true;
    }
    !RECOMMEND_WORKAROUND_DISABLED_APPS.contains(&top_app)
}

#[inline]
pub fn should_use_selection_replacement(spotlight_visible: bool, top_app: &str) -> bool {
    spotlight_visible || RECOMMEND_WORKAROUND_DISABLED_APPS.contains(&top_app)
}

#[inline]
pub fn empty_character_for_app(top_app: &str) -> u16 {
    if NICE_SPACE_APPS.contains(&top_app) {
        0x200C
    } else {
        0x202F
    }
}
