use std::rc::Rc;
use yew::{html::Scope, prelude::*};

pub trait Setter<STATE>: Clone {
    fn setter<T, F>(&self, f: F) -> Callback<T>
    where
        F: FnOnce(&mut STATE, T) + 'static,
        T: 'static;

    fn map<OUT, M>(&self, mapper: M) -> MapSetter<Self, STATE, OUT>
    where
        M: Fn(&mut STATE) -> &mut OUT + 'static,
    {
        MapSetter {
            setter: self.clone(),
            mapper: Rc::new(mapper),
        }
    }

    fn map_or<OUT, M>(&self, mapper: M) -> MapOrSetter<Self, STATE, OUT>
    where
        M: Fn(&mut STATE) -> Option<&mut OUT> + 'static,
    {
        MapOrSetter {
            setter: self.clone(),
            mapper: Rc::new(mapper),
        }
    }
}

pub struct MapSetter<S, IN, OUT>
where
    S: Setter<IN>,
{
    setter: S,
    mapper: Rc<dyn Fn(&mut IN) -> &mut OUT>,
}

impl<S, IN, OUT> Clone for MapSetter<S, IN, OUT>
where
    S: Setter<IN>,
{
    fn clone(&self) -> Self {
        Self {
            setter: self.setter.clone(),
            mapper: self.mapper.clone(),
        }
    }
}

impl<S, IN, OUT> Setter<OUT> for MapSetter<S, IN, OUT>
where
    S: Setter<IN>,
    IN: 'static,
    OUT: 'static,
{
    fn setter<T, F>(&self, f: F) -> Callback<T>
    where
        F: FnOnce(&mut OUT, T) + 'static,
        T: 'static,
    {
        let mapper = self.mapper.clone();
        self.setter.setter(move |state, v| f(mapper(state), v))
    }
}

pub struct MapOrSetter<S, IN, OUT>
where
    S: Setter<IN>,
{
    setter: S,
    mapper: Rc<dyn Fn(&mut IN) -> Option<&mut OUT>>,
}

impl<S, IN, OUT> Clone for MapOrSetter<S, IN, OUT>
where
    S: Setter<IN>,
{
    fn clone(&self) -> Self {
        Self {
            setter: self.setter.clone(),
            mapper: self.mapper.clone(),
        }
    }
}

impl<S, IN, OUT> Setter<OUT> for MapOrSetter<S, IN, OUT>
where
    S: Setter<IN>,
    IN: 'static,
    OUT: 'static,
{
    fn setter<T, F>(&self, f: F) -> Callback<T>
    where
        F: FnOnce(&mut OUT, T) + 'static,
        T: 'static,
    {
        let mapper = self.mapper.clone();
        self.setter.setter(move |state, v| {
            if let Some(out) = mapper(state) {
                f(out, v)
            }
        })
    }
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
