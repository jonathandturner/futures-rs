use {IntoFuture, Future, Poll};
use stream::Stream;

/// A stream combinator which chains a computation onto each item produced by a
/// stream.
///
/// This structure is produced by the `Stream::then` method.
pub struct Then<S, F, U>
    where U: IntoFuture,
{
    stream: S,
    future: Option<U::Future>,
    f: F,
}

pub fn new<S, F, U>(s: S, f: F) -> Then<S, F, U>
    where S: Stream,
          F: FnMut(Result<S::Item, S::Error>) -> U,
          U: IntoFuture,
{
    Then {
        stream: s,
        future: None,
        f: f,
    }
}

impl<S, F, U> Stream for Then<S, F, U>
    where S: Stream,
          F: FnMut(Result<S::Item, S::Error>) -> U,
          U: IntoFuture,
{
    type Item = U::Item;
    type Error = U::Error;

    fn poll(&mut self) -> Poll<Option<U::Item>, U::Error> {
        if self.future.is_none() {
            let item = match try_poll!(self.stream.poll()) {
                Ok(None) => return Poll::Ok(None),
                Ok(Some(e)) => Ok(e),
                Err(e) => Err(e),
            };
            self.future = Some((self.f)(item).into_future());
        }
        assert!(self.future.is_some());
        let res = self.future.as_mut().unwrap().poll();
        if res.is_ready() {
            self.future = None;
        }
        res.map(Some)
    }
}
