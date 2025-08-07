// SPDX-FileCopyrightText: 2020 Inseok Lee
// SPDX-License-Identifier: MIT

#![allow(dead_code)]

use crate::havok::animation_binding::HavokAnimationBinding;
use crate::havok::object::HavokObject;
use crate::havok::skeleton::HavokSkeleton;
use std::cell::RefCell;
use std::sync::Arc;

pub struct HavokAnimationContainer {
    pub skeletons: Vec<HavokSkeleton>,
    pub bindings: Vec<HavokAnimationBinding>,
}

impl HavokAnimationContainer {
    pub fn new(object: Arc<RefCell<HavokObject>>) -> Self {
        let root = object.borrow();

        let raw_skeletons = root.get("skeletons").as_array();
        let skeletons = raw_skeletons
            .iter()
            .map(|x| HavokSkeleton::new(x.as_object()))
            .collect::<Vec<_>>();

        let raw_bindings = root.get("bindings").as_array();
        let bindings = raw_bindings
            .iter()
            .map(|x| HavokAnimationBinding::new(x.as_object()))
            .collect::<Vec<_>>();

        Self {
            skeletons,
            bindings,
        }
    }
}
