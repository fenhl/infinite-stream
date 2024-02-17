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
