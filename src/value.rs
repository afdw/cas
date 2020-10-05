use std::{any::Any, rc::Rc};

#[derive(Clone)]
pub struct Value {
    inner: Rc<dyn Any>,
}

impl Value {
    pub fn new<T: 'static>(value: T) -> Self {
        Value { inner: Rc::new(value) }
    }

    pub fn is<T: 'static>(&self) -> bool {
        self.inner.is::<T>()
    }

    pub fn downcast<T: 'static>(&self) -> Rc<T> {
        self.inner.clone().downcast().unwrap()
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.inner.type_id() == other.inner.type_id() && &*self.inner as *const dyn Any as *const u8 == &*other.inner as *const dyn Any as *const u8
    }
}
