use crate::prelude::*;
use dominator::{apply_methods, clone, events, DomBuilder};
use futures_signals::signal::Mutable;
use web_sys::{EventTarget, HtmlElement};

pub fn set_on_hover<A>(hovered: &Mutable<bool>) -> impl FnOnce(DomBuilder<A>) -> DomBuilder<A>
where
    A: AsRef<EventTarget>,
{
    let hovered = hovered.clone();

    move |dom| {
        apply_methods!(dom, {
            .event(clone!(hovered => move |_: events::MouseEnter| {
                hovered.set_neq(true);
            }))

            .event(move |_: events::MouseLeave| {
                hovered.set_neq(false);
            })
        })
    }
}

pub fn handle_on_click<A, F>(mut f: F) -> impl FnOnce(DomBuilder<A>) -> DomBuilder<A>
where
    A: AsRef<EventTarget>,
    F: FnMut() + 'static,
{
    move |dom| {
        apply_methods!(dom, {
            .event(move |_: events::Click| {
                f();
            })
        })
    }
}

pub trait MixinFnOnce<T>: FnOnce(DomBuilder<T>) -> DomBuilder<T> {}
impl<T, F> MixinFnOnce<T> for F where F: FnOnce(DomBuilder<T>) -> DomBuilder<T> {}

pub trait MixinFn<T>: Fn(DomBuilder<T>) -> DomBuilder<T> {}
impl<T, F> MixinFn<T> for F where F: Fn(DomBuilder<T>) -> DomBuilder<T> {}

pub trait MixinFnMut<T>: FnMut(DomBuilder<T>) -> DomBuilder<T> {}
impl<T, F> MixinFnMut<T> for F where F: FnMut(DomBuilder<T>) -> DomBuilder<T> {}
