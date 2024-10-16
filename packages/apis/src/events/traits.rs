use cosmwasm_std::Event;

pub trait TypedEvent: TryFrom<Event> + Into<Event> {
    const NAME: &'static str;
    fn is_type(ty: &str) -> bool {
        ty == Self::NAME || ty == format!("wasm-{}", Self::NAME)
    }
}
