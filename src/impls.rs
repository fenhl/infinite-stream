use crate::internal_prelude::*;

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

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct Filter<S: InfiniteStream, Fut, F> {
    #[pin]
    pub(crate) stream: S,
    pub(crate) f: F,
    #[pin]
    pub(crate) pending_fut: Option<Fut>,
    pub(crate) pending_item: Option<S::Item>,
}

impl<S: InfiniteStream, Fut: Future<Output = bool>, F: for<'a> FnMut(&'a S::Item) -> Fut> InfiniteStream for Filter<S, Fut, F> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let mut this = self.project();
        Poll::Ready(loop {
            if let Some(fut) = this.pending_fut.as_mut().as_pin_mut() {
                let res = ready!(fut.poll(cx));
                this.pending_fut.set(None);
                if res {
                    break this.pending_item.take().unwrap()
                }
                *this.pending_item = None;
            } else {
                let item = ready!(this.stream.as_mut().poll_next(cx));
                this.pending_fut.set(Some((this.f)(&item)));
                *this.pending_item = Some(item);
            }
        })
    }
}

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct FilterMap<S: InfiniteStream, Fut, F> {
    #[pin]
    pub(crate) stream: S,
    pub(crate) f: F,
    #[pin]
    pub(crate) pending: Option<Fut>,
}

impl<T, S: InfiniteStream, Fut: Future<Output = Option<T>>, F: FnMut(S::Item) -> Fut> InfiniteStream for FilterMap<S, Fut, F> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let mut this = self.project();
        Poll::Ready(loop {
            if let Some(fut) = this.pending.as_mut().as_pin_mut() {
                let item = ready!(fut.poll(cx));
                this.pending.set(None);
                if let Some(item) = item {
                    break item
                }
            } else {
                let item = ready!(this.stream.as_mut().poll_next(cx));
                this.pending.set(Some((this.f)(item)));
            }
        })
    }
}

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct Map<S: InfiniteStream, F> {
    #[pin]
    pub(crate) stream: S,
    pub(crate) f: F,
}

impl<T, S: InfiniteStream, F: FnMut(S::Item) -> T> InfiniteStream for Map<S, F> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let mut this = self.project();
        let item = ready!(this.stream.as_mut().poll_next(cx));
        Poll::Ready((this.f)(item))
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
pub struct Then<S: InfiniteStream, Fut, F> {
    #[pin]
    pub(crate) stream: S,
    #[pin]
    pub(crate) fut: Option<Fut>,
    pub(crate) f: F,
}

impl<T, S: InfiniteStream, Fut: Future<Output = T>, F: FnMut(S::Item) -> Fut> InfiniteStream for Then<S, Fut, F> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let mut this = self.project();
        Poll::Ready(loop {
            if let Some(fut) = this.fut.as_mut().as_pin_mut() {
                let item = ready!(fut.poll(cx));
                this.fut.set(None);
                break item
            } else {
                let item = ready!(this.stream.as_mut().poll_next(cx));
                this.fut.set(Some((this.f)(item)));
            }
        })
    }
}
