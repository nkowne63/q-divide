use std::cell::RefCell;
use std::rc::Rc;

pub fn cellize<T>(t: T) -> Rc<RefCell<T>> {
    Rc::new(RefCell::new(t))
}
