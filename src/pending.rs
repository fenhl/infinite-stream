use {
    std::marker::PhantomData,
    crate::internal_prelude::*,
};

pub fn pending<T>() -> Pending<T> {
    assert_infinite_stream::<T, _>(Pending { _data: PhantomData })
}

#[must_use = "streams do nothing unless polled"]
pub struct Pending<T> {
    _data: PhantomData<T>,
}

impl<T> Unpin for Pending<T> {}

impl<T> InfiniteStream for Pending<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Item> {
        Poll::Pending
    }
}

impl<T> Clone for Pending<T> {
    fn clone(&self) -> Self {
        pending()
    }
}
