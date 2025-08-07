// SPDX-FileCopyrightText: 2020 Inseok Lee
// SPDX-License-Identifier: MIT

use crate::havok::HavokAnimation;
use crate::havok::byte_reader::ByteReader;
use crate::havok::object::HavokObject;
use crate::havok::transform::HavokTransform;
use core::{cell::RefCell, cmp};
use std::f32;
use std::sync::Arc;

#[repr(u8)]
#[allow(clippy::upper_case_acronyms)]
enum RotationQuantization {
    POLAR32 = 0,
    THREECOMP40 = 1,
    THREECOMP48 = 2,
    THREECOMP24 = 3,
    STRAIGHT16 = 4,
    UNCOMPRESSED = 5,
}

impl RotationQuantization {
    pub fn from_raw(raw: u8) -> Self {
        match raw {
            0 => Self::POLAR32,
            1 => Self::THREECOMP40,
            2 => Self::THREECOMP48,
            3 => Self::THREECOMP24,
            4 => Self::STRAIGHT16,
            5 => Self::UNCOMPRESSED,
            _ => panic!(),
        }
    }

    pub fn align(&self) -> usize {
        match self {
            Self::POLAR32 => 4,
            Self::THREECOMP40 => 1,
            Self::THREECOMP48 => 2,
            Self::THREECOMP24 => 1,
            Self::STRAIGHT16 => 2,
            Self::UNCOMPRESSED => 4,
        }
    }

    pub fn bytes_per_quaternion(&self) -> usize {
        match self {
            Self::POLAR32 => 4,
            Self::THREECOMP40 => 5,
            Self::THREECOMP48 => 6,
            Self::THREECOMP24 => 3,
            Self::STRAIGHT16 => 2,
            Self::UNCOMPRESSED => 16,
        }
    }
}

#[repr(u8)]
#[allow(clippy::upper_case_acronyms)]
enum ScalarQuantization {
    BITS8 = 0,
    BITS16 = 1,
}

impl ScalarQuantization {
    pub fn from_raw(raw: u8) -> Self {
        match raw {
            0 => Self::BITS8,
            1 => Self::BITS16,
            _ => panic!(),
        }
    }

    pub fn bytes_per_component(&self) -> usize {
        match self {
            Self::BITS8 => 1,
            Self::BITS16 => 2,
        }
    }
}

pub struct HavokSplineCompressedAnimation {
    duration: f32,
    number_of_transform_tracks: usize,
    num_frames: usize,
    num_blocks: usize,
    max_frames_per_block: usize,
    mask_and_quantization_size: u32,
    block_inverse_duration: f32,
    frame_duration: f32,
    block_offsets: Vec<u32>,
    data: Vec<u8>,
}

impl HavokSplineCompressedAnimation {
    pub fn new(object: Arc<RefCell<HavokObject>>) -> Self {
        let root = object.borrow();

        let duration = root.get("duration").as_real();
        let number_of_transform_tracks = root.get("numberOfTransformTracks").as_int() as usize;
        let num_frames = root.get("numFrames").as_int() as usize;
        let num_blocks = root.get("numBlocks").as_int() as usize;
        let max_frames_per_block = root.get("maxFramesPerBlock").as_int() as usize;
        let mask_and_quantization_size = root.get("maskAndQuantizationSize").as_int() as u32;
        let block_inverse_duration = root.get("blockInverseDuration").as_real();
        let frame_duration = root.get("frameDuration").as_real();

        let raw_block_offsets = root.get("blockOffsets").as_array();
        let block_offsets = raw_block_offsets
            .iter()
            .map(|x| x.as_int() as u32)
            .collect::<Vec<_>>();

        let raw_data = root.get("data").as_array();
        let data = raw_data
            .iter()
            .map(|x| x.as_int() as u8)
            .collect::<Vec<_>>();

        Self {
            duration,
            number_of_transform_tracks,
            num_frames,
            num_blocks,
            max_frames_per_block,
            mask_and_quantization_size,
            block_inverse_duration,
            frame_duration,
            block_offsets,
            data,
        }
    }

    fn get_block_and_time(&self, frame: usize, delta: f32) -> (usize, f32, u8) {
        let mut block_out = frame / (self.max_frames_per_block - 1);

        block_out = cmp::max(block_out, 0);
        block_out = cmp::min(block_out, self.num_blocks - 1);

        let first_frame_of_block = block_out * (self.max_frames_per_block - 1);
        let real_frame = (frame - first_frame_of_block) as f32 + delta;
        let block_time_out = real_frame * self.frame_duration;

        let quantized_time_out = ((block_time_out * self.block_inverse_duration)
            * (self.max_frames_per_block as f32 - 1.)) as u8;

        (block_out, block_time_out, quantized_time_out)
    }

    #[allow(non_snake_case)]
    fn find_span(n: usize, p: usize, u: u8, U: &[u8]) -> usize {
        if u >= U[n + 1] {
            return n;
        }
        if u <= U[0] {
            return p;
        }

        let mut low = p;
        let mut high = n + 1;
        let mut mid = (low + high) / 2;
        while u < U[mid] || u >= U[mid + 1] {
            if u < U[mid] {
                high = mid;
            } else {
                low = mid;
            }
            mid = (low + high) / 2;
        }
        mid
    }

    fn read_knots(
        data: &mut ByteReader,
        u: u8,
        frame_duration: f32,
    ) -> (usize, usize, Vec<f32>, usize) {
        let n = data.read_u16_le() as usize;
        let p = data.read() as usize;
        let raw = data.raw();
        let span = Self::find_span(n, p, u, raw);

        #[allow(non_snake_case)]
        let mut U = vec![0.; 2 * p];

        for i in 0..2 * p {
            let item = raw[i + 1] as usize + span - p;
            U[i] = (item as f32) * frame_duration;
        }

        data.seek(n + p + 2);

        (n, p, U, span)
    }

    fn unpack_signed_quaternion_32(data: &[u8]) -> [f32; 4] {
        let input = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

        let low = (input & 0x3FFFF) as f32;
        let high = ((input & 0xFFC0000) >> 18) as f32 / 1023.0;

        let value = 1. - high * high;

        let a = f32::sqrt(low);
        let b = low - a * a;
        let c = if a == 0. { f32::MAX } else { 1. / (a + a) };

        let theta = a / 511.0 * (f32::consts::PI / 2.);
        let phi = b * c * (f32::consts::PI / 2.);

        // spherical coordinate to cartesian coordinate
        let mut result = [
            f32::sin(theta) * f32::cos(phi),
            f32::sin(theta) * f32::sin(phi),
            f32::cos(theta),
            1.,
        ];
        for item in result.iter_mut() {
            *item *= f32::sqrt(1. - value * value);
        }
        result[3] = value;

        let mask = input >> 28;
        for (i, item) in result.iter_mut().enumerate() {
            if mask & (1 << i) != 0 {
                *item = -*item;
            }
        }

        result
    }

    fn unpack_signed_quaternion_40(data: &[u8]) -> [f32; 4] {
        let permute = [256, 513, 1027, 0];
        let data_mask_and = [4095, 4095, 4095, 0];
        let data_mask_or = [0, 0, 0, 2047];
        let data = [
            u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
        ];

        let mut buf = [0u32; 4];
        unsafe {
            let m = core::slice::from_raw_parts(
                permute.as_ptr() as *const u8,
                permute.len() * core::mem::size_of::<u32>(),
            );
            let a = core::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                data.len() * core::mem::size_of::<u32>(),
            );
            let r = core::slice::from_raw_parts_mut(
                buf.as_mut_ptr() as *mut u8,
                buf.len() * core::mem::size_of::<u32>(),
            );
            for i in 0..16 {
                r[i] = a[m[i] as usize];
            }
        }

        let mask = 2;
        for (i, item) in buf.iter_mut().enumerate() {
            if mask & (1 << i) != 0 {
                *item >>= 4;
            }
        }

        for (i, item) in buf.iter_mut().enumerate() {
            *item = (*item & data_mask_and[i]) | data_mask_or[i];
        }

        let mut result = [0., 0., 0., 1.];
        for (i, &item) in buf.iter().enumerate() {
            result[i] = ((item as f32) - 2047.0) / 2895.0;
        }

        let length_square = result.iter().map(|x| x * x).sum::<f32>();
        let mut remaining = f32::sqrt(1. - length_square);
        if (data[1] & 64) == 64 {
            remaining = -remaining;
        }

        match data[1] & 48 {
            0 => [remaining, result[0], result[1], result[2]],
            16 => [result[0], remaining, result[1], result[2]],
            32 => [result[0], result[1], remaining, result[2]],
            48 => [result[0], result[1], result[2], remaining],
            _ => panic!(),
        }
    }

    fn unpack_signed_quaternion_48(data: &[u8]) -> [f32; 4] {
        let data = [
            u16::from_le_bytes([data[0], data[1]]),
            u16::from_le_bytes([data[2], data[3]]),
            u16::from_le_bytes([data[4], data[5]]),
        ];

        let item1 = data[0] & 0x7FFF;
        let item2 = data[1] & 0x7FFF;
        let item3 = data[2] & 0x7FFF;
        let missing_index = (((data[1] & 0x8000) >> 14) | ((data[0] & 0x8000) >> 15)) as usize;
        let mut vals = [0x3fff, 0x3fff, 0x3fff, 0x3fff];

        let mut index = usize::from(missing_index == 0);
        vals[index] = item1;
        index += 1 + (usize::from(missing_index == index + 1));
        vals[index] = item2;
        index += 1 + (usize::from(missing_index == index + 1));
        vals[index] = item3;

        let mut result = [0., 0., 0., 1.];
        for (i, &item) in vals.iter().enumerate() {
            result[i] = ((item as f32) - 16383.0) / 23169.0;
        }

        let length_square = result.iter().map(|x| x * x).sum::<f32>();
        let mut remaining = f32::sqrt(1. - length_square);
        if data[2] & 0x8000 != 0 {
            remaining = -remaining;
        }

        result[missing_index] = remaining;

        result
    }

    fn unpack_quaternion(quantization: &RotationQuantization, data: &[u8]) -> [f32; 4] {
        match quantization {
            RotationQuantization::POLAR32 => Self::unpack_signed_quaternion_32(data),
            RotationQuantization::THREECOMP40 => Self::unpack_signed_quaternion_40(data),
            RotationQuantization::THREECOMP48 => Self::unpack_signed_quaternion_48(data),
            _ => panic!(),
        }
    }

    fn read_packed_quaternions(
        quantization: RotationQuantization,
        data: &mut ByteReader,
        n: usize,
        p: usize,
        span: usize,
    ) -> Vec<[f32; 4]> {
        data.align(quantization.align());
        let bytes_per_quaternion = quantization.bytes_per_quaternion();

        let mut result = Vec::new();
        for i in 0..(p + 1) {
            result.push(Self::unpack_quaternion(
                &quantization,
                &data.raw()[bytes_per_quaternion * (i + span - p)..],
            ));
        }

        data.seek(bytes_per_quaternion * (n + 1));

        result
    }

    fn unpack_vec_8(min_p: [f32; 4], max_p: [f32; 4], vals: &[u8]) -> [f32; 4] {
        let mut result = [0., 0., 0., 1.];
        for i in 0..4 {
            result[i] = ((vals[i] as f32) / 255.) * (max_p[i] - min_p[i]) + min_p[i];
        }

        result
    }

    fn unpack_vec_16(min_p: [f32; 4], max_p: [f32; 4], vals: &[u16]) -> [f32; 4] {
        let mut result = [0., 0., 0., 1.];
        for i in 0..4 {
            result[i] = ((vals[i] as f32) / 65535.) * (max_p[i] - min_p[i]) + min_p[i];
        }

        result
    }

    #[allow(non_snake_case)]
    fn recompose(stat_mask: u8, dyn_mask: u8, S: [f32; 4], I: [f32; 4], in_out: &mut [f32; 4]) {
        for i in 0..4 {
            if stat_mask & (1 << i) != 0 {
                in_out[i] = S[i];
            }
        }

        for i in 0..4 {
            if dyn_mask & (1 << i) != 0 {
                in_out[i] = I[i];
            }
        }
    }

    #[allow(non_snake_case)]
    fn evaluate(time: f32, p: usize, U: &[f32], P: &[[f32; 4]]) -> [f32; 4] {
        if p > 3 {
            panic!()
        }

        let mut result = [0., 0., 0., 0.];
        if p == 1 {
            let t = (time - U[0]) / (U[1] - U[0]);

            for (i, item) in result.iter_mut().enumerate() {
                *item = P[0][i] + t * (P[1][i] - P[0][i]);
            }
        } else {
            // evaluate interpolation.
            let p_minus_1 = p - 1;
            let mut values = [1.; 16];
            let mut low = [0.; 16];
            let mut high = [0.; 16];

            for i in 1..(p + 1) {
                high[4 * i] = time - U[p_minus_1 + 1 - i];
                low[4 * i] = U[i + p_minus_1] - time;
                let mut val = 0.;
                for j in 0..i {
                    let a = low[4 * (j + 1)] + high[4 * (i - j)];
                    let b = values[4 * j] / a;
                    let c = low[4 * (j + 1)] * b;
                    values[4 * j] = val + c;
                    val = high[4 * (i - j)] * b;
                }
                values[4 * i] = val;
            }
            for i in 0..(p + 1) {
                for (j, item) in result.iter_mut().enumerate() {
                    *item += values[4 * i] * P[i][j];
                }
            }
        }

        result
    }

    fn compute_packed_nurbs_offsets<'a>(base: &'a [u8], p: &[u32], o2: usize, o3: u32) -> &'a [u8] {
        let offset = (p[o2] + (o3 & 0x7fff_ffff)) as usize;

        &base[offset..]
    }

    fn unpack_quantization_types(
        packed_quantization_types: u8,
    ) -> (ScalarQuantization, RotationQuantization, ScalarQuantization) {
        let translation = ScalarQuantization::from_raw(packed_quantization_types & 0x03);
        let rotation = RotationQuantization::from_raw((packed_quantization_types >> 2) & 0x0F);
        let scale = ScalarQuantization::from_raw((packed_quantization_types >> 6) & 0x03);

        (translation, rotation, scale)
    }

    fn sample_translation(
        &self,
        quantization: ScalarQuantization,
        time: f32,
        quantized_time: u8,
        mask: u8,
        data: &mut ByteReader,
    ) -> [f32; 4] {
        let result = if mask != 0 {
            Self::read_nurbs_curve(
                quantization,
                data,
                quantized_time,
                self.frame_duration,
                time,
                mask,
                [0., 0., 0., 0.],
            )
        } else {
            [0., 0., 0., 0.]
        };

        data.align(4);

        result
    }

    fn sample_rotation(
        &self,
        quantization: RotationQuantization,
        time: f32,
        quantized_time: u8,
        mask: u8,
        data: &mut ByteReader,
    ) -> [f32; 4] {
        let result = Self::read_nurbs_quaternion(
            quantization,
            data,
            quantized_time,
            self.frame_duration,
            time,
            mask,
        );

        data.align(4);

        result
    }

    fn sample_scale(
        &self,
        quantization: ScalarQuantization,
        time: f32,
        quantized_time: u8,
        mask: u8,
        data: &mut ByteReader,
    ) -> [f32; 4] {
        let result = if mask != 0 {
            Self::read_nurbs_curve(
                quantization,
                data,
                quantized_time,
                self.frame_duration,
                time,
                mask,
                [1., 1., 1., 1.],
            )
        } else {
            [1., 1., 1., 1.]
        };

        data.align(4);

        result
    }

    #[allow(non_snake_case)]
    fn read_nurbs_curve(
        quantization: ScalarQuantization,
        data: &mut ByteReader,
        quantized_time: u8,
        frame_duration: f32,
        u: f32,
        mask: u8,
        I: [f32; 4],
    ) -> [f32; 4] {
        let mut max_p = [0., 0., 0., 1.];
        let mut min_p = [0., 0., 0., 1.];
        let mut S = [0., 0., 0., 1.];

        let (n, p, U, span) = if mask & 0xf0 != 0 {
            Self::read_knots(data, quantized_time, frame_duration)
        } else {
            (0, 0, vec![0.; 10], 0)
        };
        data.align(4);

        for i in 0..3 {
            if (1 << i) & mask != 0 {
                S[i] = data.read_f32_le();
            } else if (1 << (i + 4)) & mask != 0 {
                min_p[i] = data.read_f32_le();
                max_p[i] = data.read_f32_le();
            }
        }

        let stat_mask = mask & 0x0f;
        let dyn_mask = (!mask >> 4) & (!mask & 0x0f);

        if mask & 0xf0 != 0 {
            let bytes_per_component = quantization.bytes_per_component();
            data.align(2);

            let sizes = [0, 1, 1, 2, 1, 2, 2, 3];
            let size = sizes[((mask >> 4) & 7) as usize];
            let mut new_data = data.clone();
            new_data.seek(bytes_per_component * size * (span - p));

            let mut P = [[0., 0., 0., 1.]; 4];
            for pv in P.iter_mut().take(p + 1) {
                match quantization {
                    ScalarQuantization::BITS8 => {
                        let mut vals = [0; 4];
                        for (j, item) in vals.iter_mut().enumerate().take(3) {
                            if (1 << (j + 4)) & mask != 0 {
                                *item = new_data.read();
                            }
                        }

                        *pv = Self::unpack_vec_8(min_p, max_p, &vals);
                    }
                    ScalarQuantization::BITS16 => {
                        let mut vals = [0; 4];
                        for (j, item) in vals.iter_mut().enumerate().take(3) {
                            if (1 << (j + 4)) & mask != 0 {
                                *item = new_data.read_u16_le();
                            }
                        }

                        *pv = Self::unpack_vec_16(min_p, max_p, &vals);
                    }
                }

                Self::recompose(stat_mask, dyn_mask, S, I, pv);
            }

            let result = Self::evaluate(u, p, &U, &P);

            data.seek(bytes_per_component * size * (n + 1));

            result
        } else {
            let mut result = I;
            Self::recompose(stat_mask, dyn_mask, S, I, &mut result);

            result
        }
    }

    #[allow(non_snake_case)]
    fn read_nurbs_quaternion(
        quantization: RotationQuantization,
        data: &mut ByteReader,
        quantized_time: u8,
        frame_duration: f32,
        u: f32,
        mask: u8,
    ) -> [f32; 4] {
        if mask & 0xf0 != 0 {
            let (n, p, U, span) = Self::read_knots(data, quantized_time, frame_duration);
            let P = Self::read_packed_quaternions(quantization, data, n, p, span);
            Self::evaluate(u, p, &U, &P)
        } else if mask & 0x0f != 0 {
            data.align(quantization.align());
            let result = Self::unpack_quaternion(&quantization, data.raw());
            data.seek(quantization.bytes_per_quaternion());

            result
        } else {
            [0., 0., 0., 1.]
        }
    }
}

impl HavokAnimation for HavokSplineCompressedAnimation {
    fn duration(&self) -> f32 {
        self.duration
    }

    fn sample(&self, time: f32) -> Vec<HavokTransform> {
        let frame_float = ((time / 1000.) / self.duration) * (self.num_frames as f32 - 1.);
        let frame = frame_float as usize;
        let delta = frame_float - frame as f32;

        let (block, block_time, quantized_time) = self.get_block_and_time(frame, delta);

        let mut data = ByteReader::new(Self::compute_packed_nurbs_offsets(
            &self.data,
            &self.block_offsets,
            block,
            self.mask_and_quantization_size,
        ));
        let mut mask = ByteReader::new(Self::compute_packed_nurbs_offsets(
            &self.data,
            &self.block_offsets,
            block,
            0x8000_0000,
        ));

        let mut result = Vec::with_capacity(self.number_of_transform_tracks);
        for _ in 0..self.number_of_transform_tracks {
            let packed_quantization_types = mask.read();

            let (translation_type, rotation_type, scale_type) =
                Self::unpack_quantization_types(packed_quantization_types);

            let translation = self.sample_translation(
                translation_type,
                block_time,
                quantized_time,
                mask.read(),
                &mut data,
            );
            let rotation = self.sample_rotation(
                rotation_type,
                block_time,
                quantized_time,
                mask.read(),
                &mut data,
            );
            let scale = self.sample_scale(
                scale_type,
                block_time,
                quantized_time,
                mask.read(),
                &mut data,
            );

            result.push(HavokTransform::from_trs(translation, rotation, scale));
        }

        result
    }
}
