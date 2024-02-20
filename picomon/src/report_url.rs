use core::fmt;
use core::str;

pub struct ReportUrl {
    buf: [u8; 1024], // Maximum length for now is 1024 TODO make that a generic parameter
    cursor: usize,
}

impl<'a> ReportUrl {
    pub fn new() -> Self {
        ReportUrl {
            buf: [0; 1024],
            cursor: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        str::from_utf8(&self.buf[0..self.cursor]).unwrap()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    /* Not currently used
    pub fn clear(&mut self) {
        self.cursor = 0;
    }

    pub fn len(&self) -> usize {
        self.cursor
    }

    pub fn empty(&self) -> bool {
        self.cursor == 0
    }

    pub fn full(&self) -> bool {
        self.capacity() == self.cursor
    }
     */
}

impl fmt::Write for ReportUrl {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let cap = self.capacity();
        for (i, &b) in self.buf[self.cursor..cap]
            .iter_mut()
            .zip(s.as_bytes().iter())
        {
            *i = b;
        }
        self.cursor = usize::min(cap, self.cursor + s.as_bytes().len());
        Ok(())
    }
}
