use cosmwasm_std::Event;

pub trait TypedEvent: TryFrom<Event> + Into<Event> {
    const NAME: &'static str;
    fn is_type(ty: &str) -> bool {
        Self::NAME == ty || Self::NAME == format!("wasm-{ty}")
    }
}
