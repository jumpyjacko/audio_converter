use crate::models::task::TaskQueue;

pub struct TasksManager<'a> {
    queue: TaskQueue<'a>,
}

impl TasksManager<'_> {
    pub fn new() -> Self {
        return TasksManager { queue: TaskQueue::new() }
    }
}
