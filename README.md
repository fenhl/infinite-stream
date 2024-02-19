`infinite-stream` is a [Rust](https://rust-lang.org/) library for streams (asynchronous iterators) that always keep yielding items (or panic, or become pending forever). This means that the return type of the `next` future is `Self::Item` instead of `Option<Self::Item>`, for example, which lets you skip handling the `None` case.

Besides manually implementing the `InfiniteStream` trait, this crate currently offers the following ways to construct an infinite stream:

* An `expect` extension method on regular (possibly finite) streams, which returns a wrapper stream that panics if `None` is yielded from the inner stream.
* A `pending` function which returns an infinite stream that never yields any items, and a `chain_pending` method on regular streams which becomes pending after the inner stream is exhausted.
* The functions `unfold` and `try_unfold` which are similar to the ones for regular streams.

This crate's implementation is heavily based on the implementation of the `Stream` trait and related helpers from the [`futures`](https://docs.rs/futures) crate. However, this crate is in early stages of development and doesn't offer equivalents for most of the helpers from `futures` yet. Pull requests are welcome.
