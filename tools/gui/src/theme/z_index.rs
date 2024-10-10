#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zindex {
    Modal,
    Dropdown,
    Sidebar,
    Default,
}

impl Zindex {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Modal => "1000",
            Self::Dropdown => "100",
            Self::Sidebar => "10",
            Self::Default => "1",
        }
    }
}
