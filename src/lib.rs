#![doc = include_str!("../README.md")]
pub use iref;
pub use jiff;
use serde::{Deserialize, Serialize};
pub use url;
pub use validator::{Validate, ValidationErrors};

pub mod doi;
pub use doi::Doi;
pub mod orcid;
pub use orcid::OrcId;
pub mod mifa;
pub mod rembi;


// TODO: may not be necessary if validator does it internally.
// TODO: asref, deref, borrow etc.
/// Wrapper type which guarantees its contents are valid.
#[derive(Debug, Serialize, Clone)]
#[serde(transparent)]
pub struct Valid<T: Validate>(T);

impl<T: Validate> Valid<T> {
    pub fn try_new(value: T) -> Result<Self, ValidationErrors> {
        value.validate()?;
        Ok(Self(value))
    }

    /// Return a reference to the contained value.
    pub fn inner(&self) -> &T {
        &self.0
    }

    /// Consume the wrapper and return the contained value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<'de, T: Deserialize<'de> + Validate> Deserialize<'de> for Valid<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let t = T::deserialize(deserializer)?;
        t.validate().map_err(serde::de::Error::custom)?;
        Ok(Self(t))
    }
}

mod u16_as_str {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(year: &Option<u16>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(y) = year {
            y.to_string().serialize(serializer)
        } else {
            None::<String>.serialize(serializer)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u16>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Some(s) = Option::<String>::deserialize(deserializer)? else {
            return Ok(None);
        };

        let n = s.parse::<u16>().map_err(serde::de::Error::custom)?;
        Ok(Some(n))
    }
}
