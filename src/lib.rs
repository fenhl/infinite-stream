use {
    std::{
        ops::DerefMut,
        pin::Pin,
        task::{
            Context,
            Poll,
        },
    },
    futures::{
        future::{
            Either,
            Future,
        },
        stream::Stream,
    },
    crate::impls::*,
};
pub use crate::fns::*;

mod fns;
mod impls;

pub trait InfiniteStream {
    type Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item>;
}

impl<S: InfiniteStream + Unpin + ?Sized> InfiniteStream for &mut S {
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        S::poll_next(Pin::new(&mut **self), cx)
    }
}

impl<P: DerefMut + Unpin> InfiniteStream for Pin<P>
where P::Target: InfiniteStream {
    type Item = <P::Target as InfiniteStream>::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        self.get_mut().as_mut().poll_next(cx)
    }
}

impl<S: InfiniteStream + Unpin + ?Sized> InfiniteStream for Box<S> {
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        Pin::new(&mut **self).poll_next(cx)
    }
}

impl<A: InfiniteStream, B: InfiniteStream<Item = A::Item>> InfiniteStream for Either<A, B> {
    type Item = A::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        match self.as_pin_mut() {
            Either::Left(x) => x.poll_next(cx),
            Either::Right(x) => x.poll_next(cx),
        }
    }
}

pub trait InfiniteStreamExt: InfiniteStream {
    fn next(&mut self) -> Next<'_, Self>
    where Self: Unpin {
        assert_future::<Self::Item, _>(Next(self))
    }

    fn left_stream<B: InfiniteStream<Item = Self::Item>>(self) -> Either<Self, B>
    where Self: Sized {
        assert_infinite_stream::<Self::Item, _>(Either::Left(self))
    }

    fn poll_next_unpin(&mut self, cx: &mut Context<'_>) -> Poll<Self::Item>
    where Self: Unpin {
        Pin::new(self).poll_next(cx)
    }

    fn right_stream<A: InfiniteStream<Item = Self::Item>>(self) -> Either<A, Self>
    where Self: Sized {
        assert_infinite_stream::<Self::Item, _>(Either::Right(self))
    }
}

impl<T: InfiniteStream + ?Sized> InfiniteStreamExt for T {}

pub trait StreamExt: Stream {
    fn expect(self, msg: &str) -> Expect<'_, Self>
    where Self: Sized {
        assert_infinite_stream::<Self::Item, _>(Expect { stream: self, msg })
    }
}

impl<T: Stream + ?Sized> StreamExt for T {}

/// Just a helper function to ensure the futures we're returning all have the right implementations.
fn assert_future<T, Fut: Future<Output = T>>(future: Fut) -> Fut { future }

/// Just a helper function to ensure the infinite streams we're returning all have the right implementations.
fn assert_infinite_stream<T, S: InfiniteStream<Item = T>>(stream: S) -> S { stream }
