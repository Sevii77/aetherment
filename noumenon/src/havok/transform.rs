// SPDX-FileCopyrightText: 2020 Inseok Lee
// SPDX-License-Identifier: MIT

use crate::havok::object::HavokReal;

#[derive(Debug)]
pub struct HavokTransform {
    pub translation: [f32; 4],
    pub rotation: [f32; 4],
    pub scale: [f32; 4],
}

impl HavokTransform {
    pub fn new(vec: &[HavokReal]) -> Self {
        Self {
            translation: [vec[0], vec[1], vec[2], vec[3]],
            rotation: [vec[4], vec[5], vec[6], vec[7]],
            scale: [vec[8], vec[9], vec[10], vec[11]],
        }
    }

    pub fn from_trs(translation: [f32; 4], rotation: [f32; 4], scale: [f32; 4]) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }
}
