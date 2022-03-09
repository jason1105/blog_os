use crate::print;
use alloc::task::Wake;
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{stream::StreamExt, task::AtomicWaker, Stream};
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
        Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
    );
}

pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            print!("scancode queue full\n");
        } else {
            WAKER.wake();
        }
    } else {
        print!("scancode queue uninitialized\n");
    }
}

struct ScancodeStream {
    _private: (),
    count: usize,
}

impl ScancodeStream {
    fn new() -> Self {
        SCANCODE_QUEUE.init_once(|| ArrayQueue::new(100));
        Self {
            _private: (),
            count: 1,
        }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        SCANCODE_QUEUE
            .get()
            .map(|queue| {
                if let Ok(scancode) = queue.pop() {
                    Poll::Ready(Some(scancode))
                } else {
                    // We assume the queue is empty.
                    // but interrupt occurs right now, a scancode is pushed to the queue.
                    // so we need to poll again.
                    WAKER.register(&cx.waker());
                    if self.count == 1 {
                        print!(" stream waker: {:?}", &cx.waker());
                        self.get_mut().count += 1;
                    }
                    if let Ok(scancode) = queue.pop() {
                        WAKER.take();
                        Poll::Ready(Some(scancode))
                    } else {
                        Poll::Pending
                    }
                }
            })
            .unwrap()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

pub async fn keyboard_task() {
    let mut scancodes = ScancodeStream::new();
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        // who executes scancodes.next().await????↓↓↓↓↓↓
        while let Some(scancode) = scancodes.next().await {
            let mut keyboard = KEYBOARD.lock();
            //print!("get scancode: {:?}", scancode);
            if let Ok(Some(key_eveny)) = keyboard.add_byte(scancode) {
                if let Some(key) = keyboard.process_keyevent(key_eveny) {
                    match key {
                        DecodedKey::RawKey(key) => print!("{:?}", key),
                        DecodedKey::Unicode(character) => print!("{}", character),
                    }
                }
            }
        }
    } else {
        print!("scancode queue uninitialized\n");
    }
}
