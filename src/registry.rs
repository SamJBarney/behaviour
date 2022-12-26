pub use crate::identifier::Identifier;
pub struct Registry<T> {
    keys: Vec<Identifier>,
    values: Vec<T>,
}

impl<T> Registry<T> {
    pub fn new() -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            keys: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
        }
    }

    pub fn contains(&self, id: &Identifier) -> bool {
        self.keys.contains(&id)
    }

    pub fn get_handle(&self, id: &Identifier) -> Option<RegistryHandle> {
        for (idx, key) in self.keys.iter().enumerate() {
            if id == key {
                return Some(RegistryHandle::new(idx));
            }
        }
        None
    }

    pub fn get(&self, handle: &RegistryHandle) -> Option<&T> {
        self.values.get(handle.idx)
    }

    pub fn get_direct(&self, id: &Identifier) -> Option<&T> {
        let handle = self.get_handle(&id)?;
        self.get(&handle)
    }

    pub fn insert(&mut self, id: &Identifier, value: T) -> Result<(), RegistryInsertError> {
        if !self.contains(&id) {
            self.keys.push(id.clone());
            self.values.push(value);
            Ok(())
        } else {
            Err(RegistryInsertError::EntryAlreadyExists)
        }
    }

    pub fn clear(&mut self) {
        self.keys.clear();
        self.values.clear();
    }
}

impl<T> Default for Registry<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RegistryHandle {
    idx: usize,
}

impl RegistryHandle {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }

    pub fn value(&self) -> usize {
        self.idx
    }
}

#[derive(Debug, PartialEq)]
pub enum RegistryInsertError {
    EntryAlreadyExists,
}

#[cfg(test)]
mod tests {
    use super::{Registry, RegistryHandle, RegistryInsertError};
    pub use crate::identifier::Identifier;

    type Subject = Registry<usize>;

    mod with_capacity {
        use super::*;

        #[test]
        pub fn works() {
            let subject = Subject::default();
            assert_eq!(subject.keys.capacity(), 0);
            assert_eq!(subject.values.capacity(), 0);
        }

        #[test]
        pub fn with_capacity() {
            let capacity: usize = 3;
            let subject = Subject::with_capacity(capacity);
            assert_eq!(subject.keys.capacity(), capacity);
            assert_eq!(subject.values.capacity(), capacity);
        }
    }

    mod contains {
        use super::*;

        #[test]
        pub fn works() {
            let mut subject = Subject::default();
            let id = Identifier::from("test");
            subject.keys.push(id.clone());
            subject.values.push(13);

            assert!(subject.contains(&id));
        }

        #[test]
        pub fn key_missing() {
            let subject = Subject::default();
            let id = Identifier::from("test");

            assert!(!subject.contains(&id));
        }
    }

    mod get_handle {
        use super::*;

        #[test]
        pub fn works() {
            let mut subject = Subject::default();
            let id = Identifier::from("test");
            subject.keys.push(id.clone());
            subject.values.push(13);

            assert_eq!(subject.get_handle(&id), Some(RegistryHandle::new(0)));
        }

        #[test]
        pub fn out_of_bounds() {
            let subject = Subject::default();
            let id = Identifier::from("test");

            assert_eq!(subject.get_handle(&id), None);
        }
    }

    mod get {
        use super::*;

        #[test]
        pub fn works() {
            let mut subject = Subject::default();
            let value: usize = 12;
            subject.keys.push(Identifier::from("test"));
            subject.values.push(value);
            let handle = RegistryHandle::new(0);

            assert_eq!(subject.get(&handle), Some(&value));
        }

        #[test]
        pub fn out_of_bounds() {
            let subject = Subject::default();
            let handle = RegistryHandle::new(0);

            assert_eq!(subject.get(&handle), None);
        }
    }

    mod insert {
        use super::*;

        #[test]
        pub fn works() {
            let mut subject = Subject::default();
            let id = Identifier::from("test");
            let value: usize = 12;

            assert_eq!(subject.insert(&id, value), Ok(()));

            assert_eq!(subject.keys.get(0), Some(&id));
            assert_eq!(subject.values.get(0), Some(&value));
        }

        #[test]
        pub fn already_exists() {
            let mut subject = Subject::default();
            let id = Identifier::from("test");
            let existing_value: usize = 1;
            let value: usize = 12;
            subject.keys.push(id.clone());
            subject.values.push(existing_value);

            assert_eq!(
                subject.insert(&id, value),
                Err(RegistryInsertError::EntryAlreadyExists)
            );

            assert_eq!(subject.keys.get(0), Some(&id));
            assert_eq!(subject.values.get(0), Some(&existing_value));
        }
    }
}
