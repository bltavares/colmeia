// This file is generated by rust-protobuf 2.20.0. Do not edit
// @generated

// https://github.com/rust-lang/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_imports)]
#![allow(unused_results)]
//! Generated file from `schema.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
// const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_20_0;

#[derive(PartialEq,Clone,Default)]
pub struct Index {
    // message fields
    field_type: ::protobuf::SingularField<::std::string::String>,
    content: ::protobuf::SingularField<::std::vec::Vec<u8>>,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a Index {
    fn default() -> &'a Index {
        <Index as ::protobuf::Message>::default_instance()
    }
}

impl Index {
    pub fn new() -> Index {
        ::std::default::Default::default()
    }

    // required string type = 1;


    pub fn get_field_type(&self) -> &str {
        match self.field_type.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }
    pub fn clear_field_type(&mut self) {
        self.field_type.clear();
    }

    pub fn has_field_type(&self) -> bool {
        self.field_type.is_some()
    }

    // Param is passed by value, moved
    pub fn set_field_type(&mut self, v: ::std::string::String) {
        self.field_type = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_field_type(&mut self) -> &mut ::std::string::String {
        if self.field_type.is_none() {
            self.field_type.set_default();
        }
        self.field_type.as_mut().unwrap()
    }

    // Take field
    pub fn take_field_type(&mut self) -> ::std::string::String {
        self.field_type.take().unwrap_or_else(|| ::std::string::String::new())
    }

    // optional bytes content = 2;


    pub fn get_content(&self) -> &[u8] {
        match self.content.as_ref() {
            Some(v) => &v,
            None => &[],
        }
    }
    pub fn clear_content(&mut self) {
        self.content.clear();
    }

    pub fn has_content(&self) -> bool {
        self.content.is_some()
    }

    // Param is passed by value, moved
    pub fn set_content(&mut self, v: ::std::vec::Vec<u8>) {
        self.content = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_content(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.content.is_none() {
            self.content.set_default();
        }
        self.content.as_mut().unwrap()
    }

    // Take field
    pub fn take_content(&mut self) -> ::std::vec::Vec<u8> {
        self.content.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }
}

impl ::protobuf::Message for Index {
    fn is_initialized(&self) -> bool {
        if self.field_type.is_none() {
            return false;
        }
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.field_type)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_bytes_into(wire_type, is, &mut self.content)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.field_type.as_ref() {
            my_size += ::protobuf::rt::string_size(1, &v);
        }
        if let Some(ref v) = self.content.as_ref() {
            my_size += ::protobuf::rt::bytes_size(2, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.field_type.as_ref() {
            os.write_string(1, &v)?;
        }
        if let Some(ref v) = self.content.as_ref() {
            os.write_bytes(2, &v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> Index {
        Index::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                "type",
                |m: &Index| { &m.field_type },
                |m: &mut Index| { &mut m.field_type },
            ));
            fields.push(::protobuf::reflect::accessor::make_singular_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                "content",
                |m: &Index| { &m.content },
                |m: &mut Index| { &mut m.content },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<Index>(
                "Index",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static Index {
        static instance: ::protobuf::rt::LazyV2<Index> = ::protobuf::rt::LazyV2::INIT;
        instance.get(Index::new)
    }
}

impl ::protobuf::Clear for Index {
    fn clear(&mut self) {
        self.field_type.clear();
        self.content.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Index {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Index {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Stat {
    // message fields
    mode: ::std::option::Option<u32>,
    uid: ::std::option::Option<u32>,
    gid: ::std::option::Option<u32>,
    size: ::std::option::Option<u64>,
    blocks: ::std::option::Option<u64>,
    offset: ::std::option::Option<u64>,
    byteOffset: ::std::option::Option<u64>,
    mtime: ::std::option::Option<u64>,
    ctime: ::std::option::Option<u64>,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a Stat {
    fn default() -> &'a Stat {
        <Stat as ::protobuf::Message>::default_instance()
    }
}

impl Stat {
    pub fn new() -> Stat {
        ::std::default::Default::default()
    }

    // required uint32 mode = 1;


    pub fn get_mode(&self) -> u32 {
        self.mode.unwrap_or(0)
    }
    pub fn clear_mode(&mut self) {
        self.mode = ::std::option::Option::None;
    }

    pub fn has_mode(&self) -> bool {
        self.mode.is_some()
    }

    // Param is passed by value, moved
    pub fn set_mode(&mut self, v: u32) {
        self.mode = ::std::option::Option::Some(v);
    }

    // optional uint32 uid = 2;


    pub fn get_uid(&self) -> u32 {
        self.uid.unwrap_or(0)
    }
    pub fn clear_uid(&mut self) {
        self.uid = ::std::option::Option::None;
    }

    pub fn has_uid(&self) -> bool {
        self.uid.is_some()
    }

    // Param is passed by value, moved
    pub fn set_uid(&mut self, v: u32) {
        self.uid = ::std::option::Option::Some(v);
    }

    // optional uint32 gid = 3;


    pub fn get_gid(&self) -> u32 {
        self.gid.unwrap_or(0)
    }
    pub fn clear_gid(&mut self) {
        self.gid = ::std::option::Option::None;
    }

    pub fn has_gid(&self) -> bool {
        self.gid.is_some()
    }

    // Param is passed by value, moved
    pub fn set_gid(&mut self, v: u32) {
        self.gid = ::std::option::Option::Some(v);
    }

    // optional uint64 size = 4;


    pub fn get_size(&self) -> u64 {
        self.size.unwrap_or(0)
    }
    pub fn clear_size(&mut self) {
        self.size = ::std::option::Option::None;
    }

    pub fn has_size(&self) -> bool {
        self.size.is_some()
    }

    // Param is passed by value, moved
    pub fn set_size(&mut self, v: u64) {
        self.size = ::std::option::Option::Some(v);
    }

    // optional uint64 blocks = 5;


    pub fn get_blocks(&self) -> u64 {
        self.blocks.unwrap_or(0)
    }
    pub fn clear_blocks(&mut self) {
        self.blocks = ::std::option::Option::None;
    }

    pub fn has_blocks(&self) -> bool {
        self.blocks.is_some()
    }

    // Param is passed by value, moved
    pub fn set_blocks(&mut self, v: u64) {
        self.blocks = ::std::option::Option::Some(v);
    }

    // optional uint64 offset = 6;


    pub fn get_offset(&self) -> u64 {
        self.offset.unwrap_or(0)
    }
    pub fn clear_offset(&mut self) {
        self.offset = ::std::option::Option::None;
    }

    pub fn has_offset(&self) -> bool {
        self.offset.is_some()
    }

    // Param is passed by value, moved
    pub fn set_offset(&mut self, v: u64) {
        self.offset = ::std::option::Option::Some(v);
    }

    // optional uint64 byteOffset = 7;


    pub fn get_byteOffset(&self) -> u64 {
        self.byteOffset.unwrap_or(0)
    }
    pub fn clear_byteOffset(&mut self) {
        self.byteOffset = ::std::option::Option::None;
    }

    pub fn has_byteOffset(&self) -> bool {
        self.byteOffset.is_some()
    }

    // Param is passed by value, moved
    pub fn set_byteOffset(&mut self, v: u64) {
        self.byteOffset = ::std::option::Option::Some(v);
    }

    // optional uint64 mtime = 8;


    pub fn get_mtime(&self) -> u64 {
        self.mtime.unwrap_or(0)
    }
    pub fn clear_mtime(&mut self) {
        self.mtime = ::std::option::Option::None;
    }

    pub fn has_mtime(&self) -> bool {
        self.mtime.is_some()
    }

    // Param is passed by value, moved
    pub fn set_mtime(&mut self, v: u64) {
        self.mtime = ::std::option::Option::Some(v);
    }

    // optional uint64 ctime = 9;


    pub fn get_ctime(&self) -> u64 {
        self.ctime.unwrap_or(0)
    }
    pub fn clear_ctime(&mut self) {
        self.ctime = ::std::option::Option::None;
    }

    pub fn has_ctime(&self) -> bool {
        self.ctime.is_some()
    }

    // Param is passed by value, moved
    pub fn set_ctime(&mut self, v: u64) {
        self.ctime = ::std::option::Option::Some(v);
    }
}

impl ::protobuf::Message for Stat {
    fn is_initialized(&self) -> bool {
        if self.mode.is_none() {
            return false;
        }
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.mode = ::std::option::Option::Some(tmp);
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.uid = ::std::option::Option::Some(tmp);
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.gid = ::std::option::Option::Some(tmp);
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.size = ::std::option::Option::Some(tmp);
                },
                5 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.blocks = ::std::option::Option::Some(tmp);
                },
                6 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.offset = ::std::option::Option::Some(tmp);
                },
                7 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.byteOffset = ::std::option::Option::Some(tmp);
                },
                8 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.mtime = ::std::option::Option::Some(tmp);
                },
                9 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.ctime = ::std::option::Option::Some(tmp);
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(v) = self.mode {
            my_size += ::protobuf::rt::value_size(1, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.uid {
            my_size += ::protobuf::rt::value_size(2, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.gid {
            my_size += ::protobuf::rt::value_size(3, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.size {
            my_size += ::protobuf::rt::value_size(4, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.blocks {
            my_size += ::protobuf::rt::value_size(5, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.offset {
            my_size += ::protobuf::rt::value_size(6, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.byteOffset {
            my_size += ::protobuf::rt::value_size(7, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.mtime {
            my_size += ::protobuf::rt::value_size(8, v, ::protobuf::wire_format::WireTypeVarint);
        }
        if let Some(v) = self.ctime {
            my_size += ::protobuf::rt::value_size(9, v, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.mode {
            os.write_uint32(1, v)?;
        }
        if let Some(v) = self.uid {
            os.write_uint32(2, v)?;
        }
        if let Some(v) = self.gid {
            os.write_uint32(3, v)?;
        }
        if let Some(v) = self.size {
            os.write_uint64(4, v)?;
        }
        if let Some(v) = self.blocks {
            os.write_uint64(5, v)?;
        }
        if let Some(v) = self.offset {
            os.write_uint64(6, v)?;
        }
        if let Some(v) = self.byteOffset {
            os.write_uint64(7, v)?;
        }
        if let Some(v) = self.mtime {
            os.write_uint64(8, v)?;
        }
        if let Some(v) = self.ctime {
            os.write_uint64(9, v)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> Stat {
        Stat::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                "mode",
                |m: &Stat| { &m.mode },
                |m: &mut Stat| { &mut m.mode },
            ));
            fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                "uid",
                |m: &Stat| { &m.uid },
                |m: &mut Stat| { &mut m.uid },
            ));
            fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                "gid",
                |m: &Stat| { &m.gid },
                |m: &mut Stat| { &mut m.gid },
            ));
            fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                "size",
                |m: &Stat| { &m.size },
                |m: &mut Stat| { &mut m.size },
            ));
            fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                "blocks",
                |m: &Stat| { &m.blocks },
                |m: &mut Stat| { &mut m.blocks },
            ));
            fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                "offset",
                |m: &Stat| { &m.offset },
                |m: &mut Stat| { &mut m.offset },
            ));
            fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                "byteOffset",
                |m: &Stat| { &m.byteOffset },
                |m: &mut Stat| { &mut m.byteOffset },
            ));
            fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                "mtime",
                |m: &Stat| { &m.mtime },
                |m: &mut Stat| { &mut m.mtime },
            ));
            fields.push(::protobuf::reflect::accessor::make_option_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                "ctime",
                |m: &Stat| { &m.ctime },
                |m: &mut Stat| { &mut m.ctime },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<Stat>(
                "Stat",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static Stat {
        static instance: ::protobuf::rt::LazyV2<Stat> = ::protobuf::rt::LazyV2::INIT;
        instance.get(Stat::new)
    }
}

impl ::protobuf::Clear for Stat {
    fn clear(&mut self) {
        self.mode = ::std::option::Option::None;
        self.uid = ::std::option::Option::None;
        self.gid = ::std::option::Option::None;
        self.size = ::std::option::Option::None;
        self.blocks = ::std::option::Option::None;
        self.offset = ::std::option::Option::None;
        self.byteOffset = ::std::option::Option::None;
        self.mtime = ::std::option::Option::None;
        self.ctime = ::std::option::Option::None;
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Stat {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Stat {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x0cschema.proto\";\n\x05Index\x12\x14\n\x04type\x18\x01\x20\x02(\tR\
    \x04typeB\0\x12\x1a\n\x07content\x18\x02\x20\x01(\x0cR\x07contentB\0:\0\
    \"\xe2\x01\n\x04Stat\x12\x14\n\x04mode\x18\x01\x20\x02(\rR\x04modeB\0\
    \x12\x12\n\x03uid\x18\x02\x20\x01(\rR\x03uidB\0\x12\x12\n\x03gid\x18\x03\
    \x20\x01(\rR\x03gidB\0\x12\x14\n\x04size\x18\x04\x20\x01(\x04R\x04sizeB\
    \0\x12\x18\n\x06blocks\x18\x05\x20\x01(\x04R\x06blocksB\0\x12\x18\n\x06o\
    ffset\x18\x06\x20\x01(\x04R\x06offsetB\0\x12\x20\n\nbyteOffset\x18\x07\
    \x20\x01(\x04R\nbyteOffsetB\0\x12\x16\n\x05mtime\x18\x08\x20\x01(\x04R\
    \x05mtimeB\0\x12\x16\n\x05ctime\x18\t\x20\x01(\x04R\x05ctimeB\0:\0B\0b\
    \x06proto2\
";

static file_descriptor_proto_lazy: ::protobuf::rt::LazyV2<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::rt::LazyV2::INIT;

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::Message::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    file_descriptor_proto_lazy.get(|| {
        parse_descriptor_proto()
    })
}
