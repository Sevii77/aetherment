// SPDX-FileCopyrightText: 2020 Inseok Lee
// SPDX-License-Identifier: MIT

#![allow(clippy::arc_with_non_send_sync)]

use crate::havok::byte_reader::ByteReader;
use crate::havok::object::{
    HavokInteger, HavokObject, HavokObjectType, HavokObjectTypeMember, HavokRootObject, HavokValue,
    HavokValueType,
};
use crate::havok::slice_ext::SliceByteOrderExt;
use core::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

#[repr(i8)]
enum HavokTagType {
    Eof = -1,
    Invalid = 0,
    FileInfo = 1,
    Type = 2,
    Object = 3,
    ObjectRemember = 4,
    Backref = 5,
    ObjectNull = 6,
    FileEnd = 7,
}

impl HavokTagType {
    pub fn from_raw(raw: u8) -> Self {
        match raw {
            255 => HavokTagType::Eof,
            0 => HavokTagType::Invalid,
            1 => HavokTagType::FileInfo,
            2 => HavokTagType::Type,
            3 => HavokTagType::Object,
            4 => HavokTagType::ObjectRemember,
            5 => HavokTagType::Backref,
            6 => HavokTagType::ObjectNull,
            7 => HavokTagType::FileEnd,
            _ => panic!(),
        }
    }
}

pub struct HavokBinaryTagFileReader<'a> {
    file_version: u8,
    remembered_strings: Vec<Arc<str>>,
    remembered_types: Vec<Arc<HavokObjectType>>,
    remembered_objects: Vec<Arc<RefCell<HavokObject>>>,
    objects: Vec<Arc<RefCell<HavokObject>>>,
    reader: ByteReader<'a>,
}

impl<'a> HavokBinaryTagFileReader<'a> {
    pub fn read(data: &'a [u8]) -> HavokRootObject {
        let mut reader = Self::new(ByteReader::new(data));

        reader.do_read()
    }

    fn new(reader: ByteReader<'a>) -> Self {
        let file_version = 0;
        let remembered_strings = vec![Arc::from("string"), Arc::from("")];
        let remembered_types = vec![Arc::new(HavokObjectType::new(
            Arc::from("object"),
            None,
            Vec::new(),
        ))];
        let remembered_objects = Vec::new();
        let objects = Vec::new();

        Self {
            file_version,
            remembered_strings,
            remembered_types,
            remembered_objects,
            objects,
            reader,
        }
    }

    fn do_read(&mut self) -> HavokRootObject {
        let signature1 = self.reader.read_bytes(4).to_int_le::<u32>();
        let signature2 = self.reader.read_bytes(4).to_int_le::<u32>();
        if signature1 != 0xCAB0_0D1E || signature2 != 0xD011_FACE {
            panic!()
        }

        loop {
            let tag_type = HavokTagType::from_raw(self.read_packed_int() as u8);
            match tag_type {
                HavokTagType::FileInfo => {
                    self.file_version = self.read_packed_int() as u8;
                    assert_eq!(self.file_version, 3, "Unimplemented version");
                    self.remembered_objects
                        .push(Arc::new(RefCell::new(HavokObject::new(
                            self.remembered_types[0].clone(),
                            HashMap::new(),
                        ))))
                }
                HavokTagType::Type => {
                    let object_type = self.read_type();
                    self.remembered_types.push(Arc::new(object_type));
                }
                HavokTagType::Backref => panic!(),
                HavokTagType::ObjectRemember => {
                    let object = Arc::new(RefCell::new(self.read_object()));

                    self.remembered_objects.push(object.clone());
                    self.objects.push(object);
                }
                HavokTagType::FileEnd => {
                    break;
                }
                _ => panic!(),
            }
        }

        // fill object references
        for object in &self.objects {
            self.fill_object_reference(&mut object.borrow_mut());
        }

        HavokRootObject::new(self.remembered_objects[1].clone())
    }

    fn read_object(&mut self) -> HavokObject {
        let object_type_index = self.read_packed_int();
        let object_type = self.remembered_types[object_type_index as usize].clone();

        let members = object_type.members();
        let data_existence = self.read_bit_field(members.len());

        let data = members
            .into_iter()
            .enumerate()
            .map(|(index, member)| {
                let value = if data_existence[index] {
                    self.read_object_member_value(member)
                } else {
                    Self::default_value(member.type_)
                };
                (index, value)
            })
            .collect::<HashMap<_, _>>();

        HavokObject::new(object_type.clone(), data)
    }

    fn read_object_member_value(&mut self, member: &HavokObjectTypeMember) -> HavokValue {
        if member.type_.is_array() {
            let array_len = self.read_packed_int();
            if member.type_.base_type() == HavokValueType::OBJECT && member.class_name.is_none() {
                panic!()
            }

            HavokValue::Array(self.read_array(member, array_len as usize))
        } else {
            match member.type_ {
                HavokValueType::BYTE => HavokValue::Integer(self.reader.read() as i32),
                HavokValueType::INT => HavokValue::Integer(self.read_packed_int()),
                HavokValueType::REAL => HavokValue::Real(self.reader.read_f32_le()),
                HavokValueType::STRING => HavokValue::String(self.read_string()),
                HavokValueType::OBJECT => {
                    HavokValue::ObjectReference(self.read_packed_int() as usize)
                }
                HavokValueType::STRUCT => {
                    let target_type = self.find_type(member.class_name.as_ref().unwrap());
                    let data_existence = self.read_bit_field(target_type.member_count());

                    let object = Arc::new(RefCell::new(HavokObject::new(
                        target_type.clone(),
                        HashMap::new(),
                    )));
                    self.objects.push(object.clone());

                    for (member_index, member) in target_type.members().into_iter().enumerate() {
                        if data_existence[member_index] {
                            if member.type_.is_tuple() {
                                panic!()
                            } else {
                                let item = self.read_object_member_value(member);
                                object.borrow_mut().set(member_index, item);
                            }
                        }
                    }
                    
                    HavokValue::Object(object)
                }
                HavokValueType::VEC4
                | HavokValueType::VEC8
                | HavokValueType::VEC12
                | HavokValueType::VEC16 => {
                    let vec_size = member.type_.base_type().vec_size() as usize;
                    HavokValue::Vec(
                        (0..vec_size)
                            .map(|_| self.reader.read_f32_le())
                            .collect::<Vec<_>>(),
                    )
                }
                _ => panic!("unimplemented {}", member.type_.bits()),
            }
        }
    }

    fn read_array(&mut self, member: &HavokObjectTypeMember, array_len: usize) -> Vec<HavokValue> {
        let base_type = member.type_.base_type();
        match base_type {
            HavokValueType::STRING => (0..array_len)
                .map(|_| HavokValue::String(self.read_string()))
                .collect::<Vec<_>>(),
            HavokValueType::STRUCT => {
                let target_type = self.find_type(member.class_name.as_ref().unwrap());
                let data_existence = self.read_bit_field(target_type.member_count());

                let mut result_objects = Vec::new();
                for _ in 0..array_len {
                    let object = Arc::new(RefCell::new(HavokObject::new(
                        target_type.clone(),
                        HashMap::new(),
                    )));

                    result_objects.push(object.clone());
                    self.objects.push(object);
                }

                // struct of array
                for (member_index, member) in target_type.members().into_iter().enumerate() {
                    if data_existence[member_index] {
                        if member.type_.is_tuple() {
                            panic!()
                        } else {
                            let data = self.read_array(member, array_len);
                            for (index, item) in data.into_iter().enumerate() {
                                result_objects[index].borrow_mut().set(member_index, item);
                            }
                        }
                    }
                }

                result_objects
                    .into_iter()
                    .map(HavokValue::Object)
                    .collect::<Vec<_>>()
            }
            HavokValueType::OBJECT => (0..array_len)
                .map(|_| {
                    let object_index = self.read_packed_int();

                    HavokValue::ObjectReference(object_index as usize)
                })
                .collect::<Vec<_>>(),
            HavokValueType::BYTE => (0..array_len)
                .map(|_| HavokValue::Integer(self.reader.read() as HavokInteger))
                .collect::<Vec<_>>(),
            HavokValueType::INT => {
                if self.file_version >= 3 {
                    self.read_packed_int(); // type?
                }
                (0..array_len)
                    .map(|_| HavokValue::Integer(self.read_packed_int()))
                    .collect::<Vec<_>>()
            }
            HavokValueType::REAL => (0..array_len)
                .map(|_| HavokValue::Real(self.reader.read_f32_le()))
                .collect::<Vec<_>>(),
            HavokValueType::VEC4
            | HavokValueType::VEC8
            | HavokValueType::VEC12
            | HavokValueType::VEC16 => {
                let vec_size = member.type_.base_type().vec_size() as usize;
                (0..array_len)
                    .map(|_| {
                        HavokValue::Vec(
                            (0..vec_size)
                                .map(|_| self.reader.read_f32_le())
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>()
            }
            _ => panic!(
                "unimplemented {} {}",
                member.type_.bits(),
                member.type_.base_type().bits()
            ),
        }
    }

    fn read_type(&mut self) -> HavokObjectType {
        let name = self.read_string();
        let _version = self.read_packed_int();
        let parent = self.read_packed_int();
        let member_count = self.read_packed_int();

        let parent = self.remembered_types[parent as usize].clone();
        let members = (0..member_count)
            .map(|_| {
                let member_name = self.read_string();
                let type_ = HavokValueType::from_bits(self.read_packed_int() as u32).unwrap();

                let tuple_size = if type_.is_tuple() {
                    self.read_packed_int()
                } else {
                    0
                };
                let type_name = if type_.base_type() == HavokValueType::OBJECT
                    || type_.base_type() == HavokValueType::STRUCT
                {
                    Some(self.read_string())
                } else {
                    None
                };

                HavokObjectTypeMember::new(member_name, type_, tuple_size as u32, type_name)
            })
            .collect::<Vec<_>>();

        HavokObjectType::new(name, Some(parent), members)
    }

    fn read_string(&mut self) -> Arc<str> {
        let length = self.read_packed_int();
        if length < 0 {
            return self.remembered_strings[-length as usize].clone();
        }

        let result = Arc::from(
            std::str::from_utf8(self.reader.read_bytes(length as usize))
                .unwrap()
                .to_owned(),
        );
        self.remembered_strings.push(Arc::clone(&result));

        result
    }

    fn read_bit_field(&mut self, count: usize) -> Vec<bool> {
        let bytes_to_read = ((count + 7) & 0xffff_fff8) / 8;
        let bytes = self.reader.read_bytes(bytes_to_read);

        let mut result = Vec::with_capacity(count);
        for byte in bytes {
            let mut byte = *byte;
            for _ in 0..8 {
                result.push((byte & 1) == 1);
                byte >>= 1;

                if result.len() == count {
                    break;
                }
            }
        }

        result
    }

    fn read_packed_int(&mut self) -> HavokInteger {
        let mut byte = self.reader.read();

        let mut result = ((byte & 0x7f) >> 1) as u32;
        let neg = byte & 1;

        let mut shift = 6;
        while byte & 0x80 != 0 {
            byte = self.reader.read();

            result |= ((byte as u32) & 0xffff_ff7f) << shift;
            shift += 7;
        }
        if neg == 1 {
            -(result as HavokInteger)
        } else {
            result as HavokInteger
        }
    }

    fn find_type(&self, type_name: &str) -> Arc<HavokObjectType> {
        self.remembered_types
            .iter()
            .find(|&x| &*x.name == type_name)
            .unwrap()
            .clone()
    }

    fn fill_object_reference(&self, object: &mut HavokObject) {
        let mut values_to_update = Vec::new();
        for (index, mut value) in object.members_mut() {
            match &mut value {
                HavokValue::ObjectReference(x) => {
                    let object_ref = &self.remembered_objects[*x];
                    values_to_update.push((*index, HavokValue::Object(object_ref.clone())));
                }
                HavokValue::Array(x) => {
                    x.iter_mut().for_each(|item| {
                        if let HavokValue::ObjectReference(x) = item {
                            let object_ref = &self.remembered_objects[*x];

                            *item = HavokValue::Object(object_ref.clone())
                        }
                    });
                }
                _ => {}
            }
        }

        for (index, value) in values_to_update {
            object.set(index, value);
        }
    }

    fn default_value(type_: HavokValueType) -> HavokValue {
        if type_.is_vec() {
            HavokValue::Array(
                (0..type_.vec_size())
                    .map(|_| Self::default_value(type_.base_type()))
                    .collect::<Vec<_>>(),
            )
        } else if type_.is_array() || type_.is_tuple() {
            HavokValue::Array(Vec::new())
        } else {
            match type_ {
                HavokValueType::EMPTY => HavokValue::Integer(HavokInteger::default()),
                HavokValueType::BYTE => HavokValue::Integer(HavokInteger::default()),
                HavokValueType::INT => HavokValue::Integer(HavokInteger::default()),
                HavokValueType::OBJECT => HavokValue::ObjectReference(0),
                _ => panic!("unimplemented {}", type_.bits()),
            }
        }
    }
}
