use crate::command::parser::nodes::object::Object;

#[derive(Debug)]
pub enum ObjectError {
    ObjectVariantMismatch(Object, Object),
    ObjectSetKeyInvalid(String),
    ObjectSetValueInvalid(String, String),

    ObjectNotPresent,
}

impl std::fmt::Display for ObjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectError::ObjectVariantMismatch(from, to) => {
                write!(f, "Object variant mismatch: {:?} != {:?}", from, to)
            }
            ObjectError::ObjectSetKeyInvalid(key) => {
                write!(f, "Invalid set key for object: \"{}\"", key)
            }
            ObjectError::ObjectSetValueInvalid(key, value) => {
                write!(
                    f,
                    "Invalid set value for object key \"{}\": \"{}\"",
                    key, value
                )
            }
            ObjectError::ObjectNotPresent => write!(f, "Object not present"),
        }
    }
}

impl std::error::Error for ObjectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
