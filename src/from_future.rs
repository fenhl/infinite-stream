use crate::internal_prelude::*;

pub fn from_future<S: InfiniteStream, Fut: Future<Output = S>>(fut: Fut) -> FromFuture<S, Fut> {
    assert_infinite_stream::<S::Item, _>(FromFuture { state: FromFutureState::Init(fut) })
}

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct FromFuture<S, Fut> {
    #[pin]
    state: FromFutureState<S, Fut>,
}

#[pin_project(project = FromFutureStateProj)]
enum FromFutureState<S, Fut> {
    Init(#[pin] Fut),
    Stream(#[pin] S),
}

impl<S: InfiniteStream, Fut: Future<Output = S>> InfiniteStream for FromFuture<S, Fut> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let mut this = self.project();
        let stream = match this.state.as_mut().project() {
            FromFutureStateProj::Init(fut) => {
                let stream = ready!(fut.poll(cx));
                this.state.set(FromFutureState::Stream(stream));
                match this.state.as_mut().project() {
                    FromFutureStateProj::Stream(stream) => stream,
                    _ => unreachable!(),
                }
            }
            FromFutureStateProj::Stream(stream) => stream,
        };
        stream.poll_next(cx)
    }
}
