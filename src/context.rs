use std::marker::Tuple;
use std::ops::Fn;

use crate::registry::{Identifier, Registry, RegistryHandle, RegistryInsertError};

pub trait NodeHandler<Args: Tuple, ReturnType>: Fn<Args, Output = ReturnType> {}

impl<CallType: Tuple, ReturnType> std::fmt::Debug for Registry<fn(CallType) -> ReturnType> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Registry").finish()
    }
}

impl<CallType: Tuple, ReturnType> std::fmt::Debug
    for Registry<fn(ReturnType, CallType) -> ReturnType>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Registry").finish()
    }
}

#[derive(Debug)]
pub struct BehaviourContext<CallType: Tuple, ReturnType = crate::state::TreeResult> {
    executors: Registry<fn(CallType) -> ReturnType>,
    decorators: Registry<fn(ReturnType, CallType) -> ReturnType>,
}

impl<CallType: Tuple, ReturnType> BehaviourContext<CallType, ReturnType> {
    pub fn new() -> Self {
        Self {
            executors: Registry::new(),
            decorators: Registry::new(),
        }
    }

    pub fn with_capacity(handler_capacity: usize, decorator_capacity: usize) -> Self {
        Self {
            executors: Registry::with_capacity(handler_capacity),
            decorators: Registry::with_capacity(decorator_capacity),
        }
    }

    pub fn register_executor(
        &mut self,
        id: &Identifier,
        handle: fn(CallType) -> ReturnType,
    ) -> Result<(), RegistryInsertError> {
        self.executors.insert(&id, handle)
    }

    pub fn register_decorator(
        &mut self,
        id: &Identifier,
        decorator: fn(ReturnType, CallType) -> ReturnType,
    ) -> Result<(), RegistryInsertError> {
        self.decorators.insert(&id, decorator)
    }

    pub fn get_executor_handle(&self, id: &Identifier) -> Option<RegistryHandle> {
        self.executors.get_handle(&id)
    }

    pub fn get_decorator_handle(&self, id: &Identifier) -> Option<RegistryHandle> {
        self.decorators.get_handle(&id)
    }

    pub fn call_executor(&self, handle: &RegistryHandle, args: CallType) -> ReturnType {
        self.executors.get(handle).unwrap()(args)
    }

    pub fn call_decorator(
        &self,
        handle: &RegistryHandle,
        args: CallType,
        result: ReturnType,
    ) -> ReturnType {
        self.decorators.get(handle).unwrap()(result, args)
    }

    pub fn clear(&mut self) {
        self.executors.clear();
        self.decorators.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::{identifier::Identifier, state::TreeResult};

    use super::BehaviourContext;
    type Subject = BehaviourContext<(i32, i32)>;

    #[test]
    fn calls_correctly() {
        fn test_func((_, _1): (i32, i32)) -> TreeResult {
            TreeResult::Success
        }

        let mut subject = Subject::new();
        subject
            .executors
            .insert(&Identifier::from("Test"), test_func)
            .unwrap();
        let handle = subject
            .executors
            .get_handle(&Identifier::from("Test"))
            .unwrap();
        assert_eq!(subject.call_executor(&handle, (1, 2)), TreeResult::Success);
    }
}
