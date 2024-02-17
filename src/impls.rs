use {
    std::{
        pin::Pin,
        task::{
            Context,
            Poll,
        },
    },
    futures::{
        future::Future,
        ready,
        stream::Stream,
    },
    pin_project::pin_project,
    crate::{
        InfiniteStream,
        InfiniteStreamExt as _,
    },
};

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct Chain<A: Stream, B: InfiniteStream<Item = A::Item>> {
    #[pin]
    pub(crate) first: Option<A>,
    #[pin]
    pub(crate) second: B,
}

impl<A: Stream, B: InfiniteStream<Item = A::Item>> InfiniteStream for Chain<A, B> {
    type Item = A::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let mut this = self.project();
        if let Some(first) = this.first.as_mut().as_pin_mut() {
            if let Some(item) = ready!(first.poll_next(cx)) {
                return Poll::Ready(item)
            }
            this.first.set(None);
        }
        this.second.poll_next(cx)
    }
}

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct Expect<'a, S: Stream> {
    #[pin]
    pub(crate) stream: S,
    pub(crate) msg: &'a str,
}

impl<'a, S: Stream> InfiniteStream for Expect<'a, S> {
    type Item = S::Item;

    #[track_caller]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let mut this = self.project();
        let opt_item = ready!(this.stream.as_mut().poll_next(cx));
        Poll::Ready(opt_item.expect(this.msg))
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Next<'a, S: InfiniteStream + Unpin + ?Sized>(pub(crate) &'a mut S);

impl<S: InfiniteStream + Unpin + ?Sized> Unpin for Next<'_, S> {}

impl<S: InfiniteStream + Unpin + ?Sized> Future for Next<'_, S> {
    type Output = S::Item;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.poll_next_unpin(cx)
    }
}

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct Unfold<T, F, Fut>
where F: FnMut(T) -> Fut,
Fut: Future {
    pub(crate) f: F,
    #[pin]
    pub(crate) state: UnfoldState<T, Fut>,
}

#[pin_project(project = UnfoldStateProj, project_replace = UnfoldStateProjReplace)]
pub(crate) enum UnfoldState<T, Fut> {
    Value(T),
    Future(#[pin] Fut),
    Empty,
}

impl<T, F, Fut, Item> InfiniteStream for Unfold<T, F, Fut>
where F: FnMut(T) -> Fut,
Fut: Future<Output = (Item, T)> {
    type Item = Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let mut this = self.project();
        let fut = match this.state.as_mut().project() {
            UnfoldStateProj::Value(_) => {
                let UnfoldStateProjReplace::Value(state) = this.state.as_mut().project_replace(UnfoldState::Empty) else { unreachable!() };
                this.state.set(UnfoldState::Future((this.f)(state)));
                match this.state.as_mut().project() {
                    UnfoldStateProj::Future(fut) => fut,
                    _ => unreachable!(),
                }
            }
            UnfoldStateProj::Future(fut) => fut,
            UnfoldStateProj::Empty => unreachable!(),
        };
        let (item, next_state) = ready!(fut.poll(cx));
        this.state.set(UnfoldState::Value(next_state));
        Poll::Ready(item)
    }
}
