use crate::prelude::*;

pub struct Label {
    pub text: String,
    pub direction: LabelDirection,
    pub size: LabelSize,
    pub color: Color,
}

pub enum LabelDirection {
    Row,
    Column,
}

pub enum LabelSize {
    Md,
}

impl LabelSize {
    pub fn class(&self) -> &'static str {
        match self {
            LabelSize::Md => &*TEXT_SIZE_MD,
        }
    }
}

impl Label {
    pub fn new() -> Self {
        Self {
            text: "".to_string(),
            direction: LabelDirection::Row,
            size: LabelSize::Md,
            color: Color::Darkish,
        }
    }

    pub fn with_text(mut self, text: impl ToString) -> Self {
        self.text = text.to_string();
        self
    }

    pub fn with_direction(mut self, direction: LabelDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_size(mut self, size: LabelSize) -> Self {
        self.size = size;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn render(self, child: Dom) -> Dom {
        static COLUMN: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "column")
                .style("justify-content", "center")
                .style("gap", ".5rem")
            }
        });

        static ROW: LazyLock<String> = LazyLock::new(|| {
            class! {
                .style("display", "flex")
                .style("flex-direction", "row")
                .style("align-items", "center")
                .style("gap", ".5rem")
            }
        });

        let Self {
            text,
            direction,
            size,
            color,
        } = self;

        html!("div", {
            .class(match direction {
                LabelDirection::Row => &*ROW,
                LabelDirection::Column => &*COLUMN
            })
            .child(html!("label", {
                .class([color.class(), size.class()])
                .text(&text)
            }))
            .child(child)
        })
    }
}
