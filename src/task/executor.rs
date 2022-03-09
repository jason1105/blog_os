use core::task::{Context, Poll, Waker};

use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use crossbeam_queue::ArrayQueue;

use super::{Task, TaskId};

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Self {
        Self {
            task_id,
            task_queue,
        }
    }

    fn to_waker(&self) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id: self.task_id,
            task_queue: self.task_queue.clone(),
        }))
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.task_queue.push(self.task_id);
    }
}

pub struct Executor {
    task_queue: Arc<ArrayQueue<TaskId>>,
    wakers: BTreeMap<TaskId, Waker>,
    tasks: BTreeMap<TaskId, Task>,
}

impl Executor {
    pub fn new() -> Self {
        let task_queue = Arc::new(ArrayQueue::new(100));
        let wakers = BTreeMap::new();
        let tasks = BTreeMap::new();
        Self {
            task_queue,
            wakers,
            tasks,
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        self.tasks.insert(task_id, task);
        self.task_queue.push(task_id).unwrap();
    }

    pub fn run(&mut self) {
        loop {
            self.run_task();
        }
    }

    pub fn run_task(&mut self) {
        let Self {
            tasks,
            task_queue,
            wakers,
        } = self;

        while let Ok(task_id) = task_queue.pop() {
            if let Some(task) = self.tasks.get_mut(&task_id) {
                let waker = self
                    .wakers
                    .entry(task_id)
                    .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()).to_waker());
                let mut context = Context::from_waker(&waker);
                match task.poll(&mut context) {
                    Poll::Ready(()) => {
                        self.tasks.remove(&task_id);
                        self.wakers.remove(&task_id);
                    }
                    Poll::Pending => {}
                }
            }
        }
    }
}
