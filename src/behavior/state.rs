#[derive(Debug, PartialEq, Eq)]
pub enum TreeResult {
    Failure,
    Success,
    Running,
}

pub struct TreeState {
    executions: Vec<ExecutionState>,
}

pub struct ExecutionState {
    previous: Vec<usize>,
    position: usize,
}
