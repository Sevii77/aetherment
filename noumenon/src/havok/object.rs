// SPDX-FileCopyrightText: 2020 Inseok Lee
// SPDX-License-Identifier: MIT

#![allow(clippy::bad_bit_mask)]
#![allow(dead_code)]

use core::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use bitflags::bitflags;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct HavokValueType(u32);

bitflags! {
    impl HavokValueType: u32 {
        const EMPTY = 0;
        const BYTE = 1;
        const INT = 2;
        const REAL = 3;
        const VEC4 = 4;
        const VEC8 = 5;
        const VEC12 = 6;
        const VEC16 = 7;
        const OBJECT = 8;
        const STRUCT = 9;
        const STRING = 10;

        const ARRAY = 0x10;
        const ARRAYBYTE = Self::ARRAY.bits() | Self::BYTE.bits();
        const ARRAYINT = Self::ARRAY.bits() | Self::INT.bits();
        const ARRAYREAL = Self::ARRAY.bits() | Self::REAL.bits();
        const ARRAYVEC4 = Self::ARRAY.bits() | Self::VEC4.bits();
        const ARRAYVEC8 = Self::ARRAY.bits() | Self::VEC8.bits();
        const ARRAYVEC12 = Self::ARRAY.bits() | Self::VEC12.bits();
        const ARRAYVEC16 = Self::ARRAY.bits() | Self::VEC16.bits();
        const ARRAYOBJECT = Self::ARRAY.bits() | Self::OBJECT.bits();
        const ARRAYSTRUCT = Self::ARRAY.bits() | Self::STRUCT.bits();
        const ARRAYSTRING = Self::ARRAY.bits() | Self::STRING.bits();

        const TUPLE = 0x20;
        const TUPLEBYTE = Self::TUPLE.bits() | Self::BYTE.bits();
        const TUPLEINT = Self::TUPLE.bits() | Self::INT.bits();
        const TUPLEREAL = Self::TUPLE.bits() | Self::REAL.bits();
        const TUPLEVEC4 = Self::TUPLE.bits() | Self::VEC4.bits();
        const TUPLEVEC8 = Self::TUPLE.bits() | Self::VEC8.bits();
        const TUPLEVEC12 = Self::TUPLE.bits() | Self::VEC12.bits();
        const TUPLEVEC16 = Self::TUPLE.bits() | Self::VEC16.bits();
        const TUPLEOBJECT = Self::TUPLE.bits() | Self::OBJECT.bits();
        const TUPLESTRUCT = Self::TUPLE.bits() | Self::STRUCT.bits();
        const TUPLESTRING = Self::TUPLE.bits() | Self::STRING.bits();
    }
}

impl HavokValueType {
    pub fn is_tuple(self) -> bool {
        (self.bits() & HavokValueType::TUPLE.bits()) != 0
    }

    pub fn is_array(self) -> bool {
        (self.bits() & HavokValueType::ARRAY.bits()) != 0
    }

    pub fn base_type(self) -> HavokValueType {
        HavokValueType::from_bits(self.bits() & 0x0f).unwrap()
    }

    pub fn is_vec(self) -> bool {
        let base_type = self.base_type();
        base_type == HavokValueType::VEC4
            || base_type == HavokValueType::VEC8
            || base_type == HavokValueType::VEC12
            || base_type == HavokValueType::VEC16
    }

    pub fn vec_size(self) -> u8 {
        match self.base_type() {
            HavokValueType::VEC4 => 4,
            HavokValueType::VEC8 => 8,
            HavokValueType::VEC12 => 12,
            HavokValueType::VEC16 => 16,
            _ => panic!(),
        }
    }
}

pub type HavokInteger = i32;
pub type HavokReal = f32;

pub enum HavokValue {
    Integer(HavokInteger),
    Real(HavokReal),
    String(Arc<str>),
    Vec(Vec<HavokReal>),
    Array(Vec<HavokValue>),
    Object(Arc<RefCell<HavokObject>>),

    ObjectReference(usize),
}

impl HavokValue {
    pub fn as_int(&self) -> HavokInteger {
        match self {
            Self::Integer(x) => *x,
            _ => panic!(),
        }
    }

    pub fn as_object(&self) -> Arc<RefCell<HavokObject>> {
        match self {
            Self::Object(x) => x.clone(),
            _ => panic!(),
        }
    }

    pub fn as_array(&self) -> &Vec<HavokValue> {
        match self {
            Self::Array(x) => x,
            _ => panic!(),
        }
    }

    pub fn as_string(&self) -> &str {
        match self {
            Self::String(x) => x,
            _ => panic!(),
        }
    }

    pub fn as_vec(&self) -> &Vec<HavokReal> {
        match self {
            Self::Vec(x) => x,
            _ => panic!(),
        }
    }

    pub fn as_real(&self) -> HavokReal {
        match self {
            Self::Real(x) => *x,
            _ => panic!(),
        }
    }
}

pub struct HavokRootObject {
    object: Arc<RefCell<HavokObject>>,
}

impl HavokRootObject {
    pub fn new(object: Arc<RefCell<HavokObject>>) -> Self {
        Self { object }
    }

    pub fn find_object_by_type(&self, type_name: &'static str) -> Arc<RefCell<HavokObject>> {
        let root_obj = self.object.borrow();
        let named_variants = root_obj.get("namedVariants");

        for variant in named_variants.as_array() {
            let variant_obj = variant.as_object();
            if variant_obj.borrow().get("className").as_string() == type_name {
                return variant_obj.borrow().get("variant").as_object();
            }
        }
        unreachable!()
    }
}

pub struct HavokObjectTypeMember {
    pub name: Arc<str>,
    pub type_: HavokValueType,
    pub tuple_size: u32,
    pub class_name: Option<Arc<str>>,
}

impl HavokObjectTypeMember {
    pub fn new(
        name: Arc<str>,
        type_: HavokValueType,
        tuple_size: u32,
        type_name: Option<Arc<str>>,
    ) -> Self {
        Self {
            name,
            type_,
            tuple_size,
            class_name: type_name,
        }
    }
}

pub struct HavokObjectType {
    pub name: Arc<str>,
    parent: Option<Arc<HavokObjectType>>,
    members: Vec<HavokObjectTypeMember>,
}

impl HavokObjectType {
    pub fn new(
        name: Arc<str>,
        parent: Option<Arc<HavokObjectType>>,
        members: Vec<HavokObjectTypeMember>,
    ) -> Self {
        Self {
            name,
            parent,
            members,
        }
    }

    pub fn members(&self) -> Vec<&HavokObjectTypeMember> {
        if let Some(x) = &self.parent {
            x.members()
                .into_iter()
                .chain(self.members.iter())
                .collect::<Vec<_>>()
        } else {
            self.members.iter().collect::<Vec<_>>()
        }
    }

    pub fn member_count(&self) -> usize {
        (if let Some(x) = &self.parent {
            x.members.len()
        } else {
            0
        }) + self.members.len()
    }
}

pub struct HavokObject {
    pub object_type: Arc<HavokObjectType>,
    data: HashMap<usize, HavokValue>,
}

impl HavokObject {
    pub fn new(object_type: Arc<HavokObjectType>, data: HashMap<usize, HavokValue>) -> Self {
        Self { object_type, data }
    }

    pub fn set(&mut self, index: usize, value: HavokValue) {
        self.data.insert(index, value);
    }

    pub fn get(&self, member_name: &str) -> &HavokValue {
        let member_index = self
            .object_type
            .members()
            .iter()
            .position(|&x| &*x.name == member_name)
            .unwrap();

        self.data.get(&member_index).unwrap()
    }

    pub(crate) fn members_mut(&mut self) -> impl Iterator<Item = (&usize, &mut HavokValue)> {
        self.data.iter_mut()
    }
}
