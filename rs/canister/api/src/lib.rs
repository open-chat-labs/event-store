use candid::{CandidType, Deserialize};
use serde::__private::from_utf8_lossy;
use serde::de::{EnumAccess, Error, Unexpected, VariantAccess};
use serde::{Deserializer, Serialize};
use std::fmt;
use std::fmt::Formatter;
use std::marker::PhantomData;

mod lifecycle;
mod queries;
mod updates;

pub use lifecycle::*;
pub use queries::*;
pub use updates::*;

pub type Milliseconds = u64;
pub type TimestampMillis = u64;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct IdempotentEvent {
    pub idempotency_key: u128,
    pub name: String,
    pub timestamp: TimestampMillis,
    pub user: Option<Anonymizable>,
    pub source: Option<Anonymizable>,
    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct IdempotentEventPrevious {
    pub idempotency_key: u128,
    pub name: String,
    pub timestamp: TimestampMillis,
    pub user: Option<String>,
    pub source: Option<String>,
    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct IndexedEvent {
    pub index: u64,
    pub name: String,
    pub timestamp: TimestampMillis,
    pub user: Option<String>,
    pub source: Option<String>,
    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,
}

#[derive(CandidType, Serialize, Clone, Debug)]
pub enum Anonymizable {
    Public(String),
    Anonymize(String),
}

impl Anonymizable {
    pub fn new(value: String, anonymize: bool) -> Anonymizable {
        if anonymize {
            Anonymizable::Anonymize(value)
        } else {
            Anonymizable::Public(value)
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Anonymizable::Public(s) => s,
            Anonymizable::Anonymize(s) => s,
        }
    }

    pub fn is_public(&self) -> bool {
        matches!(self, Anonymizable::Public(_))
    }
}

impl<'de> Deserialize<'de> for Anonymizable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Field0,
            Field1,
            Value(String),
        }
        #[doc(hidden)]
        struct FieldVisitor;
        impl<'de> serde::de::Visitor<'de> for FieldVisitor {
            type Value = Field;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "variant identifier")
            }
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match value {
                    0u64 => Ok(Field::Field0),
                    1u64 => Ok(Field::Field1),
                    _ => Err(Error::invalid_value(
                        Unexpected::Unsigned(value),
                        &"variant index 0 <= i < 2",
                    )),
                }
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match value {
                    "Public" => Ok(Field::Field0),
                    "Anonymize" => Ok(Field::Field1),
                    value => Ok(Field::Value(value.to_string())),
                }
            }
            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match value {
                    b"Public" => Ok(Field::Field0),
                    b"Anonymize" => Ok(Field::Field1),
                    _ => {
                        let value = &from_utf8_lossy(value);
                        Err(Error::unknown_variant(value, VARIANTS))
                    }
                }
            }
        }
        impl<'de> Deserialize<'de> for Field {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserializer::deserialize_identifier(deserializer, FieldVisitor)
            }
        }
        #[doc(hidden)]
        struct Visitor<'de> {
            marker: PhantomData<Anonymizable>,
            lifetime: PhantomData<&'de ()>,
        }
        impl<'de> serde::de::Visitor<'de> for Visitor<'de> {
            type Value = Anonymizable;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                Formatter::write_str(formatter, "enum Anonymizable")
            }
            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                match EnumAccess::variant(data)? {
                    (Field::Field0, variant) => Result::map(
                        VariantAccess::newtype_variant::<String>(variant),
                        Anonymizable::Public,
                    ),
                    (Field::Field1, variant) => Result::map(
                        VariantAccess::newtype_variant::<String>(variant),
                        Anonymizable::Anonymize,
                    ),
                    (Field::Value(value), _) => Ok(Self::Value::Public(value)),
                }
            }
        }
        #[doc(hidden)]
        const VARIANTS: &[&str] = &["Public", "Anonymize"];
        Deserializer::deserialize_enum(
            deserializer,
            "Anonymizable",
            VARIANTS,
            Visitor {
                marker: PhantomData::<Anonymizable>,
                lifetime: PhantomData,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(true)]
    #[test_case(false)]
    fn deserialization_succeeds(current_version: bool) {
        let bytes = if current_version {
            let value = IdempotentEvent {
                idempotency_key: 1,
                name: "name".to_string(),
                timestamp: 2,
                user: Some(Anonymizable::Public("user".to_string())),
                source: Some(Anonymizable::Public("source".to_string())),
                payload: vec![1, 2, 3],
            };

            rmp_serde::to_vec_named(&value).unwrap()
        } else {
            let value = IdempotentEventPrevious {
                idempotency_key: 1,
                name: "name".to_string(),
                timestamp: 2,
                user: Some("user".to_string()),
                source: Some("source".to_string()),
                payload: vec![1, 2, 3],
            };

            rmp_serde::to_vec_named(&value).unwrap()
        };

        let deserialized: IdempotentEvent = rmp_serde::from_slice(&bytes).unwrap();

        assert_eq!(deserialized.idempotency_key, 1);
        assert_eq!(deserialized.name, "name");
        assert_eq!(deserialized.timestamp, 2);
        assert_eq!(deserialized.user.clone().unwrap().as_str(), "user");
        assert!(deserialized.user.clone().unwrap().is_public());
        assert_eq!(deserialized.source.clone().unwrap().as_str(), "source");
        assert!(deserialized.source.clone().unwrap().is_public());
        assert_eq!(deserialized.payload, vec![1, 2, 3]);
    }
}
