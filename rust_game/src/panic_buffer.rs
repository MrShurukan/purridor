use core::fmt::{self, Write};

pub struct PanicBuffer {
    buffer: [u8; 1024],
    len: usize,
}

impl PanicBuffer {
    pub(crate) const fn new() -> Self {
        Self {
            buffer: [0; 1024],
            len: 0,
        }
    }

    pub(crate) fn as_bytes_with_nul(&mut self) -> &[u8] {
        let end = self.len.min(self.buffer.len() - 1);
        self.buffer[end] = 0;

        &self.buffer[..=end]
    }
}

impl Write for PanicBuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if self.len >= self.buffer.len() - 1 {
                break;
            }

            self.buffer[self.len] =
                if byte == 0 { b'?' } else { byte };

            self.len += 1;
        }

        Ok(())
    }
}