mod deserialize;
mod serialize;
mod tests;

pub use deserialize::{call_deserialize_fn, DeserializeJsonFn, DeserializeResult, JsonDeserConfig};
pub use serialize::{JsonSerConfig, SerializeFn};

use serde::Deserialize;

// The index of a column within a row
// TODO: Newtyping for column indices within the layout interfaces
type ColumnIdx = usize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum JsonColumn {
    Normal { key: Box<str> },
    DateTime { key: Box<str>, format: Box<str> },
}

impl JsonColumn {
    pub fn normal<K>(key: K) -> Self
    where
        K: Into<Box<str>>,
    {
        Self::Normal { key: key.into() }
    }

    pub fn datetime<K, F>(key: K, format: F) -> Self
    where
        K: Into<Box<str>>,
        F: Into<Box<str>>,
    {
        Self::DateTime {
            key: key.into(),
            format: format.into(),
        }
    }

    pub fn key(&self) -> &str {
        match self {
            Self::Normal { key } | Self::DateTime { key, .. } => key,
        }
    }

    pub fn format(&self) -> Option<&str> {
        if let Self::DateTime { format, .. } = self {
            Some(format)
        } else {
            None
        }
    }
}
