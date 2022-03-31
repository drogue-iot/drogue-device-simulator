use crate::simulator::{
    publish::{Publisher, SimulatorStateUpdate},
    simulations::SimulationState,
};
use std::rc::Rc;

struct Inner {
    publisher: Box<dyn Publisher>,
    updater: Box<dyn SimulatorStateUpdate>,
}

#[derive(Clone)]
pub struct Context {
    inner: Rc<Inner>,
}

impl Context {
    pub fn new<P, U>(publisher: P, updater: U) -> Self
    where
        P: Publisher + 'static,
        U: SimulatorStateUpdate + 'static,
    {
        Self {
            inner: Rc::new(Inner {
                publisher: Box::new(publisher),
                updater: Box::new(updater),
            }),
        }
    }

    pub fn publisher(&self) -> &dyn Publisher {
        self.inner.publisher.as_ref()
    }

    pub fn update(&self, state: SimulationState) {
        self.inner.updater.state(state)
    }
}
