use uuid::Uuid;
use xml::attribute::OwnedAttribute;

pub mod element_source;

/// Returns the value of the first attribute with the given name
pub fn attr_value<'a>(attributes: &'a [OwnedAttribute], name: &str) -> Option<&'a str> {
    attributes.iter()
        .find(|attr| attr.name.local_name == name)
        .map(|attr| attr.value.as_str())
}

/// Generates a new UUID.
pub fn uuid_gen() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
pub mod test;
