use crate::internal_prelude::*;

pub fn try_unfold<T, E, F, Fut, Item>(init: T, f: F) -> TryUnfold<T, F, Fut>
where F: FnMut(T) -> Fut,
Fut: Future<Output = Result<(Item, T), E>> {
    assert_infinite_stream::<Result<Item, E>, _>(TryUnfold { f, state: TryUnfoldState::Value(init) })
}

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct TryUnfold<T, F, Fut>
where F: FnMut(T) -> Fut,
Fut: Future {
    f: F,
    #[pin]
    state: TryUnfoldState<T, Fut>,
}

#[pin_project(project = TryUnfoldStateProj, project_replace = TryUnfoldStateProjReplace)]
enum TryUnfoldState<T, Fut> {
    Value(T),
    Future(#[pin] Fut),
    Empty,
}

impl<T, E, F, Fut, Item> InfiniteStream for TryUnfold<T, F, Fut>
where F: FnMut(T) -> Fut,
Fut: Future<Output = Result<(Item, T), E>> {
    type Item = Result<Item, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Item> {
        let mut this = self.project();
        let fut = match this.state.as_mut().project() {
            TryUnfoldStateProj::Value(_) => {
                let TryUnfoldStateProjReplace::Value(state) = this.state.as_mut().project_replace(TryUnfoldState::Empty) else { unreachable!() };
                this.state.set(TryUnfoldState::Future((this.f)(state)));
                match this.state.as_mut().project() {
                    TryUnfoldStateProj::Future(fut) => fut,
                    _ => unreachable!(),
                }
            }
            TryUnfoldStateProj::Future(fut) => fut,
            TryUnfoldStateProj::Empty => return Poll::Pending,
        };
        Poll::Ready(match ready!(fut.poll(cx)) {
            Ok((item, next_state)) => {
                this.state.set(TryUnfoldState::Value(next_state));
                Ok(item)
            }
            Err(e) => {
                this.state.set(TryUnfoldState::Empty);
                Err(e)
            }
        })
    }
}
