//! Wrapper around [`Cursor`](std::io::Cursor).
//!
//! This module provides various reads used to decode the cache data
use std::{
    io::{prelude::*, Cursor, SeekFrom},
    iter,
};

use bytes::{Buf, Bytes};

pub trait BufExtra: Buf {
    fn get_array<const LENGTH: usize>(&mut self) -> [u8; LENGTH] {
        let mut dst = [0; LENGTH];
        self.copy_to_slice(&mut dst);
        dst
    }

    /// Reads two or four unsigned bytes as an 32-bit unsigned integer.
    fn get_smart32(&mut self) -> Option<u32> {
        let condition = self.chunk()[0] & 0x80 == 0x80;

        if condition {
            Some(self.get_u32() & 0x7FFFFFFF)
        } else {
            let value = self.get_u16() as u32;
            if value == 0x7FFF {
                None
            } else {
                Some(value)
            }
        }
    }

    /// Reads one or two unsigned bytes as an 16-bit unsigned integer.
    #[inline(always)]
    fn get_unsigned_smart(&mut self) -> u16 {
        let mut i = self.get_u8() as u16;
        if i >= 0x80 {
            i -= 0x80;
            i << 8 | (self.get_u8() as u16)
        } else {
            i
        }
    }

    /// Reads either one or two bytes.
    fn get_decr_smart(&mut self) -> Option<u16> {
        match self.get_u8() as u16 {
            first if first < 128 => first.checked_sub(1),
            first => (first << 8 | self.get_u8() as u16).checked_sub(0x8000).unwrap().checked_sub(1),
        }
    }

    /// Reads masked data.
    fn get_masked_data(&mut self) -> Vec<(Option<u32>, Option<u32>)> {
        let mut result = Vec::new();
        let mut mask = self.get_u8();
        while mask > 0 {
            if mask & 0x1 == 1 {
                result.push((self.get_smart32(), self.get_decr_smart().map(|c| c as u32)));
            } else {
                result.push((None, None));
            }
            mask /= 2;
        }
        result
    }

    /// Reads a multiple of two bytes as an 32-bit unsigned integer.
    #[inline(always)]
    fn get_smarts(&mut self) -> u32 {
        let mut value: u32 = 0;
        loop {
            match self.get_unsigned_smart() as u32 {
                0x7FFF => value = value.checked_add(0x7FFF).expect("Detected u32 overflow in buffer.get_smarts()"),
                offset => break value.checked_add(offset).expect("Detected u32 overflow in buffer.get_smarts()"),
            }
        }
    }

    /// Reads one byte, returning 8 boolean bitflags.
    #[inline(always)]
    fn get_bitflags(&mut self) -> [bool; 8] {
        let flags = self.get_u8();
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

    /// Reads a 0-terminated String from the buffer
    #[inline(always)]
    fn get_string(&mut self) -> String {
        let nul_pos = memchr::memchr(0, self.chunk()).unwrap();
        let s = self.chunk()[0..nul_pos].iter().map(|&i| i as char).collect::<String>();
        self.advance(nul_pos + 1);
        s
    }

    /// Reads a 0-start and 0-terminated String from the buffer.
    #[inline(always)]
    fn get_padded_string(&mut self) -> String {
        self.get_u8();
        self.get_string()
    }

    /// Reads three unsigned bytes , returning a `[red, blue, green]` array.
    #[inline(always)]
    fn get_rgb(&mut self) -> [u8; 3] {
        self.get_array()
    }

    /// Reads two obfuscated bytes.
    #[inline(always)]
    fn get_masked_index(&mut self) -> u16 {
        // big TODO
        self.get_u16()
    }
}

impl<T: Buf> BufExtra for T {}
