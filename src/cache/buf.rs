use bytes::Buf;
use std::{
    io::{prelude::*, Cursor, SeekFrom},
    iter,
};

/// Contains a buffer, exposing various reads on it.
///
/// # Panics
///
/// In general, these methods will panic if they attempt to read beyond end of file.
pub struct Buffer {
    buf: Cursor<Vec<u8>>,
}

impl Buffer {
    /// Constructor for [`Buffer`].
    pub fn new(file: Vec<u8>) -> Buffer {
        Buffer { buf: Cursor::new(file) }
    }

    /// Reads a byte as signed 8-bit integer. Wraps [Buf::get_i8()](https://docs.rs/bytes/1.0.1/bytes/buf/trait.Buf.html#method.get_i8)
    #[inline(always)]
    pub fn read_byte(&mut self) -> i8 {
        self.buf.get_i8()
    }

    /// Reads a byte as unsigned 8-bit integer. Wraps [Buf::get_u8()](https://docs.rs/bytes/1.0.1/bytes/buf/trait.Buf.html#method.get_u8)
    #[inline(always)]
    pub fn read_unsigned_byte(&mut self) -> u8 {
        self.buf.get_u8()
    }

    /// Reads four bytes as an 32-bit signed integer. Wraps [Buf::get_i32()](https://docs.rs/bytes/1.0.1/bytes/buf/trait.Buf.html#method.get_i32)
    #[inline(always)]
    pub fn read_int(&mut self) -> i32 {
        self.buf.get_i32()
    }
    /// Reads four bytes as an 32-bit unsigned integer. Wraps [Buf::get_u32()](https://docs.rs/bytes/1.0.1/bytes/buf/trait.Buf.html#method.get_u32)
    #[inline(always)]
    pub fn read_unsigned_int(&mut self) -> u32 {
        self.buf.get_u32()
    }

    /// Reads two bytes as an 16-bit unsigned integer. Wraps [Buf::get_u16()](https://docs.rs/bytes/1.0.1/bytes/buf/trait.Buf.html#method.get_u16)
    #[inline(always)]
    pub fn read_unsigned_short(&mut self) -> u16 {
        self.buf.get_u16()
    }

    /// Reads two bytes as an 16-bit signed integer. Wraps [Buf::get_i16()](https://docs.rs/bytes/1.0.1/bytes/buf/trait.Buf.html#method.get_i16)
    #[inline(always)]
    pub fn read_short(&mut self) -> i16 {
        (self.read_byte() as i16) << 8 | (self.read_unsigned_byte() as i16)
    }

    /// Reads two or four unsigned bytes as an 32-bit unsigned integer.
    #[inline(always)]
    pub fn read_smart32(&mut self) -> Option<u32> {
        let [.., condition] = self.read_bitflags();
        self.buf.seek(SeekFrom::Current(-1)).expect("Can never be invalid");

        if condition {
            Some(self.buf.get_u32() & 0x7FFFFFFF)
        } else {
            let value = self.buf.get_u16() as u32;
            if value == 0x7FFF {
                None
            } else {
                Some(value)
            }
        }
    }

    /// Reads one or two unsigned bytes as an 16-bit unsigned integer.
    #[inline(always)]
    pub fn read_unsigned_smart(&mut self) -> u16 {
        let mut i = self.buf.get_u8() as u16;
        if i >= 0x80 {
            i -= 0x80;
            i << 8 | (self.buf.get_u8() as u16)
        } else {
            i
        }
    }

    /// Reads either one or two bytes.
    #[inline(always)]
    pub fn read_decr_smart(&mut self) -> Option<u16> {
        match self.read_unsigned_byte() as u16 {
            first if first < 128 => first.checked_sub(1),
            first => (first << 8 | self.read_unsigned_byte() as u16)
                .checked_sub(0x8000)
                .unwrap()
                .checked_sub(1),
        }
    }

    /// Reads masked data.
    #[inline(always)]
    pub fn read_masked_data(&mut self) -> Vec<(Option<u32>, Option<u32>)> {
        let mut result = Vec::new();
        let mut mask = self.read_unsigned_byte();
        while mask > 0 {
            if mask & 0x1 == 1 {
                result.push((self.read_smart32(), self.read_decr_smart().map(|c| c as u32)));
            } else {
                result.push((None, None));
            }
            mask /= 2;
        }
        result
    }

    /// Reads a multiple of two bytes as an 32-bit unsigned integer.
    #[inline(always)]
    pub fn read_smarts(&mut self) -> u32 {
        let mut value: u32 = 0;
        loop {
            match self.read_unsigned_smart() as u32 {
                0x7FFF => value = value.checked_add(0x7FFF).expect("Detected u32 overflow in buffer.read_smarts()"),
                offset => break value.checked_add(offset).expect("Detected u32 overflow in buffer.read_smarts()"),
            }
        }
    }

    /// Reads one byte, returning 8 boolean bitflags.
    #[inline(always)]
    pub fn read_bitflags(&mut self) -> [bool; 8] {
        let flags = self.buf.get_u8();
        [
            flags & 0x1 != 0,
            flags & 0x2 != 0,
            flags & 0x4 != 0,
            flags & 0x8 != 0,
            flags & 0x10 != 0,
            flags & 0x20 != 0,
            flags & 0x40 != 0,
            flags & 0x80 != 0,
        ]
    }

    /// Reads a 0-terminated String from the buffer, similar to an [`OsString`](https://doc.rust-lang.org/std/ffi/struct.OsString.html).
    /// The bytes must be valid 1-byte UTF-8.
    #[inline(always)]
    pub fn read_string(&mut self) -> String {
        iter::repeat_with(|| self.buf.get_u8())
            .take_while(|i| *i != 0)
            .map(|i| i as char)
            .collect::<String>()
    }

    /// Reads a 0-start and 0-terminated String from the buffer, similar to [`OsString`](https://doc.rust-lang.org/std/ffi/struct.OsString.html).
    /// The bytes must be valid 1-byte UTF-8. The leading 0 is discarded.
    #[inline(always)]
    pub fn read_padded_string(&mut self) -> String {
        self.buf.get_u8();
        self.read_string()
    }

    /// Reads `count` bytes from the buffer.
    #[inline(always)]
    pub fn read_n_bytes(&mut self, count: usize) -> Vec<u8> {
        iter::repeat_with(|| self.buf.get_u8()).take(count).collect()
    }

    /// Reads three unsigned bytes as an 32-bit unsigned integer.
    #[inline(always)]
    pub fn read_3_unsigned_bytes(&mut self) -> u32 {
        (self.buf.get_u8() as u32) << 16 | (self.buf.get_u8() as u32) << 8 | (self.buf.get_u8() as u32)
    }

    /// Reads three unsigned bytes , returning a `(red, blue, green)` colour tuple.
    #[inline(always)]
    pub fn read_rgb(&mut self) -> (u8, u8, u8) {
        let red = self.buf.get_u8();
        let green = self.buf.get_u8();
        let blue = self.buf.get_u8();

        (red, green, blue)
    }

    /// Reads two obfuscated bytes.
    #[inline(always)]
    pub fn read_masked_index(&mut self) -> u16 {
        // big TODO
        self.buf.get_u16()
    }

    /// Wraps [Buf::remaining()](https://docs.rs/bytes/1.0.1/bytes/buf/trait.Buf.html#tymethod.remaining).
    #[inline(always)]
    pub fn remaining(&mut self) -> usize {
        self.buf.remaining()
    }

    /// Go to some offset in the buffer. Wraps [Cursor::seek()](https://doc.rust-lang.org/std/io/trait.Seek.html#tymethod.seek).
    #[inline(always)]
    pub fn seek(&mut self, pos: SeekFrom) -> Result<u64, std::io::Error> {
        self.buf.seek(pos)
    }

    /// Returns the remainder of the buffer.
    pub fn remainder(&mut self) -> Vec<u8> {
        let count = self.buf.remaining();
        self.read_n_bytes(count)
    }
}
