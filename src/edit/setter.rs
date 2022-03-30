use std::rc::Rc;
use yew::{html::Scope, prelude::*};

pub trait Setter<STATE>: Clone {
    fn setter<T, F>(&self, f: F) -> Callback<T>
    where
        F: FnOnce(&mut STATE, T) + 'static,
        T: 'static;
}

pub struct ContextSetter<STATE, C>
where
    C: Component,
{
    scope: Scope<C>,
    m: Rc<dyn Fn(Box<dyn FnOnce(&mut STATE)>) -> C::Message>,
}

impl<STATE, C> Clone for ContextSetter<STATE, C>
where
    C: Component,
{
    fn clone(&self) -> Self {
        Self {
            scope: self.scope.clone(),
            m: self.m.clone(),
        }
    }
}

impl<STATE, C> ContextSetter<STATE, C>
where
    C: Component,
{
    pub fn new<M>(scope: Scope<C>, m: M) -> Self
    where
        M: Fn(Box<dyn FnOnce(&mut STATE)>) -> C::Message + 'static,
    {
        Self {
            scope,
            m: Rc::new(m),
        }
    }
}

impl<STATE, C, M> From<(Scope<C>, M)> for ContextSetter<STATE, C>
where
    C: Component,
    M: Fn(Box<dyn FnOnce(&mut STATE)>) -> C::Message + 'static,
{
    fn from((scope, m): (Scope<C>, M)) -> Self {
        ContextSetter::new(scope, m)
    }
}

impl<STATE, C, M> From<(&Context<C>, M)> for ContextSetter<STATE, C>
where
    C: Component,
    M: Fn(Box<dyn FnOnce(&mut STATE)>) -> C::Message + 'static,
{
    fn from((ctx, m): (&Context<C>, M)) -> Self {
        ContextSetter::new(ctx.link().clone(), m)
    }
}

impl<STATE, C> Setter<STATE> for ContextSetter<STATE, C>
where
    C: Component,
    STATE: 'static,
{
    fn setter<T, F>(&self, f: F) -> Callback<T>
    where
        F: FnOnce(&mut STATE, T) + 'static,
        T: 'static,
    {
        let m = self.m.clone();
        self.scope
            .callback_once(move |v| m(Box::new(move |state| f(state, v))))
    }
}
