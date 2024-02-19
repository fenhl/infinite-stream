pub(crate) use {
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
        assert_infinite_stream,
    },
};
