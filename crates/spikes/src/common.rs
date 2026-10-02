use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

struct NoopWake;
impl Wake for NoopWake {
    fn wake(self: Arc<Self>) {}
}

pub fn block_on<F: Future>(mut future: F) -> F::Output {
    let mut future = unsafe { std::pin::Pin::new_unchecked(&mut future) };
    let waker = Waker::from(Arc::new(NoopWake));
    let mut cx = Context::from_waker(&waker);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(result) => return result,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

pub fn init_test_image(width: usize, height: usize) -> Vec<u8> {
    let mut data = vec![0u8; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            data[idx] = ((x ^ y) & 0xFF) as u8;
            data[idx + 1] = ((x * 2) & 0xFF) as u8;
            data[idx + 2] = ((y * 2) & 0xFF) as u8;
            data[idx + 3] = 255;
        }
    }
    data
}
