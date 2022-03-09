use crate::print;

use super::Task;
use alloc::collections::VecDeque;

use core::task::{RawWaker, RawWakerVTable, Waker};

use core::task::{Context, Poll};

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}

pub struct SimpleExecutor {
    task_queue: VecDeque<Task>,
}

impl SimpleExecutor {
    pub fn new() -> SimpleExecutor {
        SimpleExecutor {
            task_queue: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task)
    }
}

impl SimpleExecutor {
    pub fn run(&mut self) {
        let mut count = 1;
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();

            // for
            if count == 1 {
                print!(" task waker: {:?}", waker);
                count += 1;
            }
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {} // task done
                Poll::Pending => self.task_queue.push_back(task),
            }
        }
    }
}
