#![no_std]
extern crate alloc;
use alloc::vec::Vec;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BoolVec {
    bytes: Vec<u8>,
    tail: u64,
    tail_len: u8,
    len: usize,
}

#[rustfmt::skip]
impl BoolVec {
    pub const fn new() -> Self { Self { bytes: Vec::new(), tail: 0, tail_len: 0, len: 0 } }
    pub fn with_capacity(n: usize) -> Self {
        Self { bytes: Vec::with_capacity(n.div_ceil(7)), ..Self::new() }
    }
    pub fn from_slice(bits: &[bool]) -> Self { bits.iter().copied().collect() }
    pub fn push(&mut self, bit: bool) {
        self.tail |= (bit as u64) << (63 - self.tail_len);
        self.tail_len += 1; self.len += 1; self.flush();
    }
    pub fn pop(&mut self) -> Option<bool> {
        if self.len == 0 { return None; }
        if self.tail_len == 0 {
            let byte = self.bytes.pop().unwrap();
            self.tail_len = block_len(byte) as u8;
            self.tail = (0..self.tail_len as usize).fold(0, |v, i| v | (block_get(byte, i) as u64) << (63 - i));
        }
        self.tail_len -= 1; self.len -= 1;
        let mask = 1 << (63 - self.tail_len); let bit = self.tail & mask != 0; self.tail &= !mask;
        Some(bit)
    }
    pub fn get(&self, mut i: usize) -> Option<bool> {
        if i >= self.len { return None; }
        for &byte in &self.bytes {
            if i < block_len(byte) { return Some(block_get(byte, i)); }
            i -= block_len(byte);
        }
        Some(tail_get(self.tail, i))
    }
    pub const fn len(&self) -> usize { self.len }
    pub const fn is_empty(&self) -> bool { self.len == 0 }
    pub fn clear(&mut self) { self.bytes.clear(); self.tail = 0; self.tail_len = 0; self.len = 0; }
    pub fn iter(&self) -> Iter<'_> {
        Iter { bits: self, byte: 0, inner: 0, bytes_left: self.len - self.tail_len as usize, tail: 0, left: self.len }
    }
    pub fn to_vec(&self) -> Vec<bool> { self.iter().collect() }
    pub fn into_encoded_parts(mut self) -> (Vec<u8>, usize) {
        while self.tail_len >= 7 {
            let n = repeats(self.tail, self.tail_len);
            if n >= 3 { self.bytes.push(repeated(self.tail, n)); self.consume(n * 3); }
            else { self.bytes.push(literal(self.tail)); self.consume(7); }
        }
        if self.tail_len != 0 { self.bytes.push(literal(self.tail)); }
        (self.bytes, self.len)
    }
    fn flush(&mut self) {
        while self.tail_len >= 7 {
            let mismatch = (3..self.tail_len as usize).find(|&i| tail_get(self.tail, i) != tail_get(self.tail, i % 3));
            match mismatch {
                Some(i) if i / 3 >= 3 => { let n = i / 3; self.bytes.push(repeated(self.tail, n)); self.consume(n * 3); }
                Some(_) => { self.bytes.push(literal(self.tail)); self.consume(7); }
                None if self.tail_len == 48 => { self.bytes.push(repeated(self.tail, 16)); self.consume(48); }
                None => break,
            }
        }
    }
    fn consume(&mut self, n: usize) { self.tail <<= n; self.tail_len -= n as u8; }
}

#[rustfmt::skip]
impl Extend<bool> for BoolVec { fn extend<T: IntoIterator<Item = bool>>(&mut self, bits: T) { for bit in bits { self.push(bit); } } }
#[rustfmt::skip]
impl FromIterator<bool> for BoolVec {
    fn from_iter<T: IntoIterator<Item = bool>>(bits: T) -> Self {
        let iter = bits.into_iter(); let mut out = Self::with_capacity(iter.size_hint().0); out.extend(iter); out
    }
}
#[rustfmt::skip]
impl From<&[bool]> for BoolVec { fn from(bits: &[bool]) -> Self { Self::from_slice(bits) } }
#[rustfmt::skip]
impl<'a> IntoIterator for &'a BoolVec { type Item = bool; type IntoIter = Iter<'a>; fn into_iter(self) -> Iter<'a> { self.iter() } }

#[derive(Clone, Debug)]
pub struct Iter<'a> {
    bits: &'a BoolVec,
    byte: usize,
    inner: usize,
    bytes_left: usize,
    tail: usize,
    left: usize,
}
#[rustfmt::skip]
impl Iterator for Iter<'_> {
    type Item = bool;
    fn next(&mut self) -> Option<bool> {
        if self.left == 0 { return None; } self.left -= 1;
        if self.bytes_left != 0 {
            let byte = self.bits.bytes[self.byte]; let bit = block_get(byte, self.inner);
            self.inner += 1; self.bytes_left -= 1;
            if self.inner == block_len(byte) { self.byte += 1; self.inner = 0; }
            Some(bit)
        } else { let bit = tail_get(self.bits.tail, self.tail); self.tail += 1; Some(bit) }
    }
    fn size_hint(&self) -> (usize, Option<usize>) { (self.left, Some(self.left)) }
}
impl ExactSizeIterator for Iter<'_> {}

#[rustfmt::skip]
const fn block_len(b: u8) -> usize { if b < 128 { 7 } else { ((b & 15) as usize + 1) * 3 } }
#[rustfmt::skip]
const fn block_get(b: u8, i: usize) -> bool { b & (1 << if b < 128 { 6 - i } else { 6 - i % 3 }) != 0 }
#[rustfmt::skip]
const fn tail_get(t: u64, i: usize) -> bool { t & (1 << (63 - i)) != 0 }
#[rustfmt::skip]
const fn literal(t: u64) -> u8 { (t >> 57) as u8 }
#[rustfmt::skip]
const fn repeated(t: u64, n: usize) -> u8 { 128 | ((t >> 57) as u8 & 112) | (n as u8 - 1) }
#[rustfmt::skip]
fn repeats(t: u64, len: u8) -> usize { (1..16).take_while(|&n| (n + 1) * 3 <= len as usize && (0..3).all(|i| tail_get(t, n * 3 + i) == tail_get(t, i))).count() + 1 }
