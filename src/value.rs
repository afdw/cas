use std::{
    any::Any,
    hash::{Hash, Hasher},
    rc::Rc,
};

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

    pub fn try_downcast<T: 'static>(&self) -> Option<Rc<T>> {
        self.inner.clone().downcast().ok()
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.inner.type_id() == other.inner.type_id() && &*self.inner as *const dyn Any as *const u8 == &*other.inner as *const dyn Any as *const u8
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.type_id().hash(state);
        (&*self.inner as *const dyn Any as *const u8).hash(state);
    }
}
