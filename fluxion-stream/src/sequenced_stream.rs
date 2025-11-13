use futures::Stream;

use crate::sequenced::Sequenced;

pub trait SequencedStreamExt<T>: Stream<Item = Sequenced<T>> {}

impl<S, T> SequencedStreamExt<T> for S where S: Stream<Item = Sequenced<T>> {}
