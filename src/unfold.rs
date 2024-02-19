use crate::internal_prelude::*;

pub fn unfold<T, F, Fut, Item>(init: T, f: F) -> Unfold<T, F, Fut>
where F: FnMut(T) -> Fut,
Fut: Future<Output = (Item, T)> {
    assert_infinite_stream::<Item, _>(Unfold { f, state: UnfoldState::Value(init) })
}

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct Unfold<T, F, Fut>
where F: FnMut(T) -> Fut,
Fut: Future {
    f: F,
    #[pin]
    state: UnfoldState<T, Fut>,
}

#[pin_project(project = UnfoldStateProj, project_replace = UnfoldStateProjReplace)]
enum UnfoldState<T, Fut> {
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
