use crate::internal_prelude::*;

macro_rules! unwrap {
    ($t:ty) => {
        impl InfiniteStream for $t {
            type Item = <Self as Stream>::Item;

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
                <Self as Stream>::poll_next(self, cx).map(Option::unwrap)
            }
        }
    };
}

unwrap!(tokio_stream::wrappers::IntervalStream);
