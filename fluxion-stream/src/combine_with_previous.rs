use crate::fluxion_stream::FluxionStream;
use fluxion_core::Ordered;
use futures::{Stream, StreamExt, future};

pub trait CombineWithPreviousExt<T>: Stream<Item = T> + Sized
where
    T: Ordered + Clone + Send + Sync + 'static,
{
    fn combine_with_previous(self) -> FluxionStream<impl Stream<Item = (Option<T>, T)>>;
}

impl<T, S> CombineWithPreviousExt<T> for S
where
    S: Stream<Item = T> + Send + Sized + 'static,
    T: Ordered + Clone + Send + Sync + 'static,
{
    fn combine_with_previous(self) -> FluxionStream<impl Stream<Item = (Option<T>, T)>> {
        let result = self.scan(None, |state: &mut Option<T>, current: T| {
            let previous = state.take();
            *state = Some(current.clone());
            future::ready(Some((previous, current)))
        });
        FluxionStream::new(result)
    }
}
