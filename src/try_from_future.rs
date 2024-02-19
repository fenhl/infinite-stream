use crate::internal_prelude::*;

pub fn try_from_future<T, E, S: InfiniteStream<Item = Result<T, E>>, Fut: Future<Output = Result<S, E>>>(fut: Fut) -> TryFromFuture<S, Fut> {
    assert_infinite_stream::<Result<T, E>, _>(TryFromFuture { state: TryFromFutureState::Init(fut) })
}

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct TryFromFuture<S, Fut> {
    #[pin]
    state: TryFromFutureState<S, Fut>,
}

#[pin_project(project = TryFromFutureStateProj)]
enum TryFromFutureState<S, Fut> {
    Init(#[pin] Fut),
    Stream(#[pin] S),
    Empty,
}

impl<T, E, S: InfiniteStream<Item = Result<T, E>>, Fut: Future<Output = Result<S, E>>> InfiniteStream for TryFromFuture<S, Fut> {
    type Item = Result<T, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let mut this = self.project();
        let stream = match this.state.as_mut().project() {
            TryFromFutureStateProj::Init(fut) => match ready!(fut.poll(cx)) {
                Ok(stream) => {
                    this.state.set(TryFromFutureState::Stream(stream));
                    match this.state.as_mut().project() {
                        TryFromFutureStateProj::Stream(stream) => stream,
                        _ => unreachable!(),
                    }
                }
                Err(e) => {
                    this.state.set(TryFromFutureState::Empty);
                    return Poll::Ready(Err(e))
                }
            },
            TryFromFutureStateProj::Stream(stream) => stream,
            TryFromFutureStateProj::Empty => return Poll::Pending,
        };
        stream.poll_next(cx)
    }
}
