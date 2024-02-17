use {
    futures::{
        future::Future,
        stream::Stream,
    },
    crate::{
        InfiniteStream,
        assert_infinite_stream,
        impls::*,
    },
};

pub fn chain<A: Stream, B: InfiniteStream<Item = A::Item>>(first: A, second: B) -> Chain<A, B> {
    assert_infinite_stream::<A::Item, _>(Chain { first: Some(first), second })
}

pub fn unfold<T, F, Fut, Item>(init: T, f: F) -> Unfold<T, F, Fut>
where F: FnMut(T) -> Fut,
Fut: Future<Output = (Item, T)> {
    assert_infinite_stream::<Item, _>(Unfold { f, state: UnfoldState::Value(init) })
}
