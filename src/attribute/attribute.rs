// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;

use {
    AttributeId,
    AttributeValue,
    Name,
    NameRef,
    SvgId,
    WriteBuffer,
    WriteOptions,
    ToStringWithOptions,
};

/// Type alias for `Name<AttributeId>`.
pub type AttributeName = Name<AttributeId>;
/// Type alias for `NameRef<AttributeId>`.
pub type AttributeNameRef<'a> = NameRef<'a, AttributeId>;

impl SvgId for AttributeId {
    fn name(&self) -> &str { self.name() }
}

/// Representation of the SVG attribute object.
#[derive(PartialEq,Clone,Debug)]
pub struct Attribute {
    /// Attribute name.
    pub name: AttributeName,
    /// Attribute value.
    pub value: AttributeValue,
    /// Visibility.
    ///
    /// Unlike many other DOM implementations, libsvgdom supports hiding of the attributes,
    /// instead of removing them. Invisible attributes act just like other attributes,
    /// but they will not be printed during SVG writing.
    /// Unless you enable them via [`WriteOptions`].
    ///
    /// All attributes are visible by default.
    ///
    /// [`WriteOptions`]: struct.WriteOptions.html
    pub visible: bool,
}

macro_rules! impl_is_type {
    ($name:ident, $t:ident) => (
        #[allow(missing_docs)]
        pub fn $name(&self) -> bool {
            match self.value {
                AttributeValue::$t(_) => true,
                _ => false,
            }
        }
    )
}

impl Attribute {
    /// Constructs a new attribute.
    pub fn new<'a, N, T>(name: N, value: T) -> Attribute
        where AttributeNameRef<'a>: From<N>, AttributeValue: From<T>
    {
        let n = AttributeNameRef::from(name);
        Attribute {
            name: AttributeName::from(n),
            value: AttributeValue::from(value),
            visible: true,
        }
    }

    /// Returns an SVG attribute ID.
    pub fn id(&self) -> Option<AttributeId> {
        match self.name {
            Name::Id(id) => Some(id),
            Name::Name(_) => None,
        }
    }

    /// Returns `true` if the attribute has the selected ID.
    pub fn has_id(&self, id: AttributeId) -> bool {
        match self.name {
            Name::Id(id2) => id2 == id,
            Name::Name(_) => false,
        }
    }

    /// Returns `true` if the attribute is an SVG attribute.
    pub fn is_svg(&self) -> bool {
        match self.name {
            Name::Id(_) => true,
            Name::Name(_) => false,
        }
    }

    /// Constructs a new attribute with a default value, if it known.
    pub fn default(id: AttributeId) -> Option<Attribute> {
        match AttributeValue::default_value(id) {
            Some(v) => Some(Attribute::new(id, v)),
            None => None,
        }
    }

    /// Returns `true` if the current attribute's value is equal to a default by the SVG spec.
    pub fn check_is_default(&self) -> bool {
        if let Name::Id(id) = self.name {
            match AttributeValue::default_value(id) {
                Some(v) => self.value == v,
                None => false,
            }
        } else {
            false
        }
    }

    impl_is_type!(is_color, Color);
    impl_is_type!(is_length, Length);
    impl_is_type!(is_length_list, LengthList);
    impl_is_type!(is_link, Link);
    impl_is_type!(is_func_link, FuncLink);
    impl_is_type!(is_number, Number);
    impl_is_type!(is_number_list, NumberList);
    impl_is_type!(is_path, Path);
    impl_is_type!(is_predef_value, PredefValue);
    impl_is_type!(is_string, String);
    impl_is_type!(is_transform, Transform);
}

fn write_quote(opt: &WriteOptions, out: &mut Vec<u8>) {
    out.push(if opt.use_single_quote { b'\'' } else { b'"' });
}

impl WriteBuffer for Attribute {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        match self.name {
            Name::Id(id) => buf.extend_from_slice(id.name().as_bytes()),
            Name::Name(ref name) => buf.extend_from_slice(name.as_bytes()),
        }
        buf.push(b'=');
        write_quote(opt, buf);
        self.value.write_buf_opt(opt, buf);
        write_quote(opt, buf);
    }
}

impl_display!(Attribute);
