// SPDX-FileCopyrightText: 2020 Inseok Lee
// SPDX-License-Identifier: MIT

use core::mem::size_of;

#[derive(Clone)]
pub struct ByteReader<'a> {
    data: &'a [u8],
    cursor: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    pub fn read(&mut self) -> u8 {
        let result = self.data[self.cursor];
        self.cursor += 1;

        result
    }

    pub fn read_u16_le(&mut self) -> u16 {
        let result = u16::from_le_bytes([self.data[self.cursor], self.data[self.cursor + 1]]);
        self.cursor += size_of::<u16>();

        result
    }

    pub fn read_f32_le(&mut self) -> f32 {
        let result = f32::from_le_bytes([
            self.data[self.cursor],
            self.data[self.cursor + 1],
            self.data[self.cursor + 2],
            self.data[self.cursor + 3],
        ]);
        self.cursor += size_of::<f32>();

        result
    }

    pub fn read_bytes(&mut self, size: usize) -> &[u8] {
        let result = &self.data[self.cursor..self.cursor + size];
        self.cursor += size;

        result
    }

    pub fn align(&mut self, align: usize) {
        self.cursor = Self::round_up(self.cursor, align)
    }

    fn round_up(num_to_round: usize, multiple: usize) -> usize {
        if multiple == 0 {
            return num_to_round;
        }

        let remainder = num_to_round % multiple;
        if remainder == 0 {
            num_to_round
        } else {
            num_to_round + multiple - remainder
        }
    }

    pub fn raw(&self) -> &[u8] {
        &self.data[self.cursor..]
    }

    pub fn seek(&mut self, offset: usize) {
        self.cursor += offset;
    }
}
