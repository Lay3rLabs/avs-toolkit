use crate::prelude::*;
use dominator::ColorScheme;

static SELECTED_SCHEME: LazyLock<Mutable<Option<ColorScheme>>> =
    LazyLock::new(|| Mutable::new(load_from_storage()));

pub fn color_scheme_signal() -> impl Signal<Item = ColorScheme> {
    map_ref! {
        let selected_scheme = SELECTED_SCHEME.signal_cloned(),
        let system_scheme = dominator::color_scheme(),
        => {
            selected_scheme.unwrap_or(*system_scheme)
        }
    }
}

const STORAGE_KEY: &'static str = "color-scheme";

pub fn set_color_scheme(scheme: ColorScheme) {
    SELECTED_SCHEME.set_neq(Some(scheme));

    web_sys::window()
        .unwrap_ext()
        .local_storage()
        .unwrap_ext()
        .unwrap_ext()
        .set_item(
            STORAGE_KEY,
            match scheme {
                ColorScheme::Light => "light",
                ColorScheme::Dark => "dark",
            },
        )
        .unwrap_ext();
}

fn load_from_storage() -> Option<ColorScheme> {
    web_sys::window()
        .unwrap_ext()
        .local_storage()
        .unwrap_ext()
        .unwrap_ext()
        .get_item(STORAGE_KEY)
        .ok()
        .flatten()
        .and_then(|value| match value.as_str() {
            "light" => Some(ColorScheme::Light),
            "dark" => Some(ColorScheme::Dark),
            _ => None,
        })
}
