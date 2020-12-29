// Copyright 2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use bytes::buf::UninitSlice;
use bytes::*;
use std::borrow::Borrow;
use std::io::*;

#[derive(Debug, PartialEq, Eq)]
struct ShrinkingBytesMut {
    target_capacity: usize,
    inner: BytesMut,
}

impl ShrinkingBytesMut {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            target_capacity: capacity,
            inner: BytesMut::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        if self.inner.capacity() > self.target_capacity {
            self.inner.truncate(self.target_capacity);
        }
    }

    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        self.inner.extend_from_slice(slice)
    }
}

impl Borrow<[u8]> for ShrinkingBytesMut {
    fn borrow(&self) -> &[u8] {
        self.inner.borrow()
    }
}

impl Read for ShrinkingBytesMut {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut buffer: Cursor<&[u8]> = Cursor::new(self.inner.borrow());
        match buffer.read(buf) {
            Ok(bytes) => {
                self.advance(bytes);
                Ok(bytes)
            }
            Err(e) => Err(e),
        }
    }
}

unsafe impl BufMut for ShrinkingBytesMut {
    #[inline]
    fn remaining_mut(&self) -> usize {
        self.inner.remaining_mut()
    }

    #[inline]
    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.inner.advance_mut(cnt)
    }

    #[inline]
    fn chunk_mut(&mut self) -> &mut UninitSlice {
        self.inner.chunk_mut()
    }
}

impl Buf for ShrinkingBytesMut {
    fn remaining(&self) -> usize {
        self.inner.remaining()
    }

    fn chunk(&self) -> &[u8] {
        self.inner.chunk()
    }

    fn advance(&mut self, cnt: usize) {
        self.inner.advance(cnt);
        if self.inner.capacity() > self.target_capacity && self.inner.len() < self.inner.capacity()
        {
            self.inner.truncate(self.inner.len())
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Buffer {
    read: ShrinkingBytesMut,
    write: ShrinkingBytesMut,
    tmp: Vec<u8>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Buffer {
    pub fn new() -> Buffer {
        Self::with_capacity(4096, 4096)
    }

    pub fn with_capacity(read: usize, write: usize) -> Self {
        Self {
            read: ShrinkingBytesMut::with_capacity(read),
            write: ShrinkingBytesMut::with_capacity(write),
            tmp: vec![0; read],
        }
    }

    pub fn clear(&mut self) {
        self.read.clear();
        self.write.clear();
    }

    // write from the tx buffer to a given sink
    pub fn write_to<T: Write>(&mut self, sink: &mut T) -> Result<Option<usize>> {
        match sink.write(self.write.borrow()) {
            Ok(bytes) => {
                self.write.advance(bytes);
                Ok(Some(bytes))
            }
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    // read from the source into the rx buffer
    pub fn read_from<T: Read>(&mut self, source: &mut T) -> Result<Option<usize>> {
        match source.read(&mut self.tmp) {
            Ok(bytes) => {
                self.read.put(&self.tmp[0..bytes]);
                Ok(Some(bytes))
            }
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn extend_from_slice(&mut self, extend: &[u8]) {
        self.write.extend_from_slice(extend)
    }

    pub fn write_pending(&self) -> usize {
        self.write.len()
    }

    pub fn read_pending(&self) -> usize {
        self.read.len()
    }

    pub fn put_u32(&mut self, n: u32) {
        self.write.put_u32(n)
    }
}

impl Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.write.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl BufRead for Buffer {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        if self.read_pending() > 0 {
            Ok(self.read.borrow())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                "Buffer contains no bytes to read",
            ))
        }
    }

    fn consume(&mut self, amt: usize) {
        self.read.advance(amt);
    }
}

impl Read for Buffer {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut buffer: Cursor<&[u8]> = Cursor::new(self.read.borrow());
        match buffer.read(buf) {
            Ok(bytes) => {
                self.read.advance(bytes);
                Ok(bytes)
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn write_and_write_to() {
        let mut buffer = Buffer::new();

        let messages: Vec<&[u8]> = vec![
            b"break the ice",
            b"he hath eaten me out of house and home",
            b"brevity is the soul of wit",
        ];

        for message in messages {
            buffer.write(message).expect("write failed");
            let mut sink = Vec::new();
            if let Ok(Some(len)) = buffer.write_to(&mut sink) {
                assert_eq!(sink.len(), len);
                assert_eq!(sink, message);
            }
        }
    }

    #[test]
    fn read_from_and_read() {
        let mut buffer = Buffer::new();

        let mut messages: Vec<&[u8]> = vec![
            b"break the ice",
            b"he hath eaten me out of house and home",
            b"brevity is the soul of wit",
        ];

        for message in &mut messages {
            let check = message.clone();
            buffer.read_from(message).expect("read from failed");
            let mut sink = vec![0; 4096];
            if let Ok(len) = buffer.read(&mut sink) {
                sink.truncate(len); // we need to shrink our buffer to match the bytes we've read
                assert_eq!(sink.len(), len); // sink contains the number of bytes read into it
                assert_eq!(sink.len(), check.len()); // sink has same number of bytes as check string
                assert_eq!(sink.as_slice(), check); // sink has same data as check string
            }
        }
    }

    #[test]
    fn read_from_multiple() {
        let mut buffer = Buffer::new();

        let mut messages: Vec<&[u8]> = vec![b"brave ", b"new ", b"world"];

        for message in &mut messages {
            buffer.read_from(message).expect("read from failed");
        }

        let check = b"brave new world";
        let mut sink = vec![0; 4096];
        if let Ok(len) = buffer.read(&mut sink) {
            sink.truncate(len); // we need to shrink our buffer to match the bytes we've read
            assert_eq!(sink.len(), len); // sink contains the number of bytes read into it
            assert_eq!(sink.len(), check.len()); // sink has same number of bytes as check string
            assert_eq!(sink.as_slice(), check); // sink has same data as check string
        }
        sink.clear();
        if let Ok(len) = buffer.read(&mut sink) {
            assert_eq!(len, 0);
            sink.truncate(len); // we need to shrink our buffer to match the bytes we've read
            assert_eq!(sink.len(), len); // sink contains the number of bytes read into it
            assert_eq!(sink.as_slice(), b""); // sink has same data as check string
        }
    }

    #[test]
    fn write_to_multiple() {
        let mut buffer = Buffer::new();

        buffer.write(b"DEAD").expect("write failed");

        let check = b"DEAD";

        let mut sink = Vec::new();
        assert_eq!(buffer.write_to(&mut sink).unwrap().unwrap(), 4);
        assert_eq!(sink.len(), check.len());
        assert_eq!(sink.as_slice(), check);

        buffer.write(b"BEEF").expect("write failed");

        let check = b"BEEF";

        let mut sink = Vec::new();
        assert_eq!(buffer.write_to(&mut sink).unwrap().unwrap(), 4);
        assert_eq!(sink.len(), check.len());
        assert_eq!(sink.as_slice(), check);

        buffer.write(b"DEAD").expect("write failed");
        buffer.write(b"BEEF").expect("write failed");

        let check = b"DEADBEEF";

        let mut sink = Vec::new();
        assert_eq!(buffer.write_to(&mut sink).unwrap().unwrap(), 8);
        assert_eq!(sink.len(), check.len());
        assert_eq!(sink.as_slice(), check);
    }

    #[test]
    fn partial_read() {
        let mut buffer = Buffer::new();

        let mut messages: Vec<&[u8]> = vec![b"DEAD", b"BEEF"];

        buffer
            .read_from(&mut messages[0])
            .expect("read from failed");

        // reading with fill_buf() will return the bytes we just wrote
        if let Ok(content) = buffer.fill_buf() {
            assert_eq!(content.len(), 4); // sink has same number of bytes as check string
            assert_eq!(content, b"DEAD"); // sink has same data as check string
        } else {
            panic!("buffer had no data");
        }

        // reading again with fill_buf() will still have the same data
        if let Ok(content) = buffer.fill_buf() {
            assert_eq!(content.len(), 4); // sink has same number of bytes as check string
            assert_eq!(content, b"DEAD"); // sink has same data as check string
        } else {
            panic!("buffer had no data");
        }

        // append to the buffer and read to get all written data
        buffer
            .read_from(&mut messages[1])
            .expect("read from failed");
        let mut sink = vec![0; 4096];
        if let Ok(len) = buffer.read(&mut sink) {
            sink.truncate(len); // we need to shrink our buffer to match the bytes we've read
            assert_eq!(sink.len(), 8); // sink has same number of bytes as check string
            assert_eq!(sink.as_slice(), b"DEADBEEF"); // sink has same data as check string
        }

        assert_eq!(buffer.read_pending(), 0);
    }
}
