use std::fmt;

use crate::{
    AttributeId,
    AttributeQName,
    AttributeQNameRef,
    AttributeValue,
    QName,
};


/// Representation of the SVG attribute object.
#[derive(Clone, PartialEq, Debug)]
pub struct Attribute {
    /// Attribute name.
    pub name: AttributeQName,
    /// Attribute value.
    pub value: AttributeValue,
}

// TODO: fix docs
macro_rules! impl_is_type {
    ($name:ident) => (
        #[allow(missing_docs)]
        pub fn $name(&self) -> bool {
            self.value.$name()
        }
    )
}

impl Attribute {
    /// Constructs a new attribute.
    pub fn new<'a, N, T>(name: N, value: T) -> Attribute
        where AttributeQNameRef<'a>: From<N>, AttributeValue: From<T>
    {
        Attribute {
            name: AttributeQNameRef::from(name).into(),
            value: AttributeValue::from(value),
        }
    }

    /// Constructs a new attribute with a default value, if it known.
    pub fn new_default(id: AttributeId) -> Option<Attribute> {
        match AttributeValue::default_value(id) {
            Some(v) => Some(Attribute::new(id, v)),
            None => None,
        }
    }

    /// Returns an SVG attribute ID.
    pub fn id(&self) -> Option<AttributeId> {
        match self.name {
            QName::Id(id) => Some(id),
            QName::Name(_) => None,
        }
    }

    /// Returns `true` if the attribute has the selected ID.
    pub fn has_id(&self, id: AttributeId) -> bool {
        self.name.has_id(id)
    }

    /// Returns `true` if the attribute is an SVG attribute.
    pub fn is_svg(&self) -> bool {
        match self.name {
            QName::Id(_) => true,
            QName::Name(_) => false,
        }
    }

    impl_is_type!(is_none);
    impl_is_type!(is_inherit);
    impl_is_type!(is_current_color);
    impl_is_type!(is_aspect_ratio);
    impl_is_type!(is_color);
    impl_is_type!(is_length);
    impl_is_type!(is_length_list);
    impl_is_type!(is_link);
    impl_is_type!(is_func_link);
    impl_is_type!(is_paint);
    impl_is_type!(is_number);
    impl_is_type!(is_number_list);
    impl_is_type!(is_path);
    impl_is_type!(is_points);
    impl_is_type!(is_string);
    impl_is_type!(is_transform);
    impl_is_type!(is_viewbox);
    impl_is_type!(is_link_container);
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}='{}'", self.name, self.value)
    }
}
