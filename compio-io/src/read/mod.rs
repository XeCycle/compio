use std::io::Cursor;

use compio_buf::{BufResult, IoBufMut, IoVectoredBufMut, SetBufInit};

mod buf;
mod ext;

pub use buf::*;
pub use ext::*;

use crate::util::slice_to_buf;

/// AsyncRead
///
/// Async read with a ownership of a buffer
pub trait AsyncRead {
    /// Read some bytes from this source into the buffer, which implements
    /// [`IoBufMut`], and return a [`BufResult`], consisting of the buffer and a
    /// [`usize`] indicating how many bytes were read.
    async fn read<B: IoBufMut>(&mut self, buf: B) -> BufResult<usize, B>;

    /// Like `read`, except that it reads into a type implements
    /// [`IoVectoredBufMut`].
    async fn read_vectored<V: IoVectoredBufMut>(&mut self, buf: V) -> BufResult<usize, V>
    where
        V::Item: IoBufMut + SetBufInit;
}

impl<A: AsyncRead + ?Sized> AsyncRead for &mut A {
    async fn read<T: IoBufMut>(&mut self, buf: T) -> BufResult<usize, T> {
        (**self).read(buf).await
    }

    async fn read_vectored<T: IoVectoredBufMut>(&mut self, buf: T) -> BufResult<usize, T>
    where
        T::Item: IoBufMut + SetBufInit,
    {
        (**self).read_vectored(buf).await
    }
}

impl<A: AsyncRead + ?Sized> AsyncRead for Box<A> {
    async fn read<T: IoBufMut>(&mut self, buf: T) -> BufResult<usize, T> {
        (**self).read(buf).await
    }

    async fn read_vectored<T: IoVectoredBufMut>(&mut self, buf: T) -> BufResult<usize, T>
    where
        T::Item: IoBufMut + SetBufInit,
    {
        (**self).read_vectored(buf).await
    }
}

impl<A: AsRef<[u8]>> AsyncRead for Cursor<A> {
    async fn read<T: IoBufMut>(&mut self, buf: T) -> BufResult<usize, T> {
        self.get_ref().as_ref().read(buf).await
    }

    async fn read_vectored<T: IoVectoredBufMut>(&mut self, buf: T) -> BufResult<usize, T>
    where
        T::Item: IoBufMut + SetBufInit,
    {
        self.get_ref().as_ref().read_vectored(buf).await
    }
}

impl AsyncRead for &[u8] {
    async fn read<T: IoBufMut>(&mut self, mut buf: T) -> BufResult<usize, T> {
        let len = slice_to_buf(self, &mut buf);

        BufResult(Ok(len), buf)
    }

    async fn read_vectored<T: IoVectoredBufMut>(&mut self, mut buf: T) -> BufResult<usize, T>
    where
        T::Item: IoBufMut + SetBufInit,
    {
        let mut this = *self; // An immutable slice to track the read position

        for buf in buf.buf_iter_mut() {
            let n = slice_to_buf(this, buf);
            this = &this[n..];
            if this.is_empty() {
                break;
            }
        }

        BufResult(Ok(self.len() - this.len()), buf)
    }
}

/// # AsyncReadAt
///
/// Async read with a ownership of a buffer and a position
pub trait AsyncReadAt {
    /// Like `read`, except that it reads at a specified position.
    async fn read_at<T: IoBufMut>(&mut self, buf: T, pos: usize) -> BufResult<usize, T>;
}

impl<A: AsyncReadAt + ?Sized> AsyncReadAt for &mut A {
    async fn read_at<T: IoBufMut>(&mut self, buf: T, pos: usize) -> BufResult<usize, T> {
        (**self).read_at(buf, pos).await
    }
}
