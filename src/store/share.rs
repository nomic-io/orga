use super::{Read, Write};
use crate::Result;
use std::cell::RefCell;
use std::rc::Rc;

// TODO: we can probably use UnsafeCell instead of RefCell since operations are
// guaranteed not to interfere with each other.

/// A shared reference to a store, allowing the store to be cloned and read from
/// or written to by multiple consumers.
///
/// `Shared` has the `Clone` trait - it is safe to clone references to the store
/// since `get`, `put`, and `delete` all operate atomically so there will never
/// be more than one reference borrowing the underlying store at a time.
pub struct Shared<T>(Rc<RefCell<T>>);

impl<T> Shared<T> {
    /// Constructs a `Shared` by wrapping the given store.
    pub fn new(inner: T) -> Self {
        Shared(Rc::new(RefCell::new(inner)))
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Shared<T> {
        Self(self.0.clone())
    }
}

impl<R: Read> Read for Shared<R> {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let store = self.0.borrow();
        store.get(key)
    }
}

impl<W: Write> Write for Shared<W> {
    fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        let mut store = self.0.borrow_mut();
        store.put(key, value)
    }

    fn delete(&mut self, key: &[u8]) -> Result<()> {
        let mut store = self.0.borrow_mut();
        store.delete(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::*;

    #[test]
    fn share() {
        let mut store = MapStore::new().into_shared();
        let mut share0 = store.clone();
        let share1 = store.clone();

        share0.put(vec![123], vec![5]).unwrap();
        assert_eq!(store.get(&[123]).unwrap(), Some(vec![5]));
        assert_eq!(share0.get(&[123]).unwrap(), Some(vec![5]));
        assert_eq!(share1.get(&[123]).unwrap(), Some(vec![5]));

        store.put(vec![123], vec![6]).unwrap();
        assert_eq!(store.get(&[123]).unwrap(), Some(vec![6]));
        assert_eq!(share0.get(&[123]).unwrap(), Some(vec![6]));
        assert_eq!(share1.get(&[123]).unwrap(), Some(vec![6]));
    }
}
