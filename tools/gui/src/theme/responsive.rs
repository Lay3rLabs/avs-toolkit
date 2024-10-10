use crate::prelude::*;
use gloo_events::EventListener;

impl WindowSize {
    pub fn font_size(&self) -> String {
        let Self { width, height } = *self;

        // Define minimum and maximum font sizes
        let font_size_min = 12.0; // For small screens
        let font_size_max = 26.0; // For large screens

        // Define viewport width breakpoints
        let viewport_width_min = 320.0; // Minimum screen width (e.g., mobile)
        let viewport_width_max = 1920.0; // Maximum screen width (e.g., desktop)

        // Clamp the width to the defined viewport range
        let clamped_width = width.clamp(viewport_width_min, viewport_width_max);

        // Calculate the font size using linear interpolation
        let font_size = font_size_min
            + (font_size_max - font_size_min) * (clamped_width - viewport_width_min)
                / (viewport_width_max - viewport_width_min);

        // Return the calculated font size
        format!("{}px", font_size)

        // this kinda works but it's too small on smaller screens:
        // let scale_ratio = self.width / 1920.0;
        // format!("{}px", scale_ratio * 16.0)
    }
}

thread_local! {
    static WINDOW_SIZE: WindowSizeListener = {
        let window = web_sys::window().unwrap_ext();
        let width = window.inner_width().unwrap_ext().as_f64().unwrap();
        let height = window.inner_height().unwrap_ext().as_f64().unwrap();
        let size = Mutable::new(WindowSize {
            width,
            height
        });

        let listener = {
            EventListener::new(&window, "resize", clone!(window, size => move |event| {
                let width = window.inner_width().unwrap().as_f64().unwrap();
                let height = window.inner_height().unwrap().as_f64().unwrap();
                size.set_neq(WindowSize {
                    width,
                    height
                });
            }))
        };

        WindowSizeListener {
            size,
            listener
        }
    };
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct WindowSize {
    pub width: f64,
    pub height: f64,
}

pub struct WindowSizeListener {
    size: Mutable<WindowSize>,
    listener: EventListener,
}

impl WindowSizeListener {
    pub fn size_signal() -> impl Signal<Item = WindowSize> {
        WINDOW_SIZE.with(|s| s.size.signal())
    }
}
