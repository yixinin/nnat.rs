use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;

use super::io::BiStream;

pub struct Tunnel<I, O>
where
    I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    O: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    i: BiStream<I>,
    o: BiStream<O>,
}

impl<I, O> Tunnel<I, O>
where
    I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    O: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    pub fn new(i: BiStream<I>, o: BiStream<O>) -> Self {
        let conn = Self { i: i, o: o };
        return conn;
    }
    pub async fn copy(&mut self) -> Result<(u64, u64), std::io::Error> {
        // let mut i = self.i.lock().unwrap();
        // let mut o = self.o.lock().unwrap();
        let x = tokio::io::copy_bidirectional(self.i.inner_mut(), self.o.inner_mut()).await;
        match x {
            Ok((a, b)) => {
                return Ok((a, b));
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
}

// impl Future for Connection
// // where
// //     I: AsyncRead + AsyncWrite + Unpin + Send,
// //     O: AsyncRead + AsyncWrite + Unpin + Send,
// {
//     type Output = Result<(u64, u64), std::io::Error>;

//     fn poll(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Self::Output> {
//         // Look at the shared state to see if the timer has already completed.
//         let mut state = self.state.lock().unwrap();
//         if state.done {
//             if let Some(err) = &state.err {
//                 return Poll::Ready(Err(std::io::Error::new(err.kind(), err.to_string())));
//             }
//             Poll::Ready(Ok((state.a2b, state.b2a)))
//         } else {
//             // Set waker so that the thread can wake up the current task
//             // when the timer has completed, ensuring that the future is polled
//             // again and sees that `completed = true`.
//             //
//             // It's tempting to do this once rather than repeatedly cloning
//             // the waker each time. However, the `TimerFuture` can move between
//             // tasks on the executor, which could cause a stale waker pointing
//             // to the wrong task, preventing `TimerFuture` from waking up
//             // correctly.
//             //
//             // N.B. it's possible to check for this using the `Waker::will_wake`
//             // function, but we omit that here to keep things simple.
//             state.waker = Some(cx.waker().clone());
//             Poll::Pending
//         }
//     }
// }
