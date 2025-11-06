use std::{cell::RefCell, rc::Rc};

pub type Shared<T> = Rc<RefCell<T>>;
