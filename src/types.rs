use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::collections::BTreeMap;
use std::error::Error;
use surrealdb_core::sql::Value;

pub struct Vars<'this> {
    err: Vec<Box<dyn Error>>,
    vars: BTreeMap<&'this str, Value>,
}

impl<'this> Vars<'this> {
    pub fn new() -> Self {
        Self {
            err: vec![],
            vars: BTreeMap::new(),
        }
    }

    pub fn put<T: Serialize + 'static>(mut self, key: &'this str, val: T) -> Self {
        let value = surrealdb_core::sql::to_value(val);
        match value {
            Ok(val) => {
                self.vars.insert(key, val);
            }
            Err(err) => {
                self.err.push(Box::new(err));
            }
        }
        self
    }
}

impl<'this> Serialize for Vars<'this> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if !self.err.is_empty() {
            let err = self
                .err
                .iter()
                .map(|err| err.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            return Err(serde::ser::Error::custom(err));
        }

        let mut map = serializer.serialize_map(Some(self.vars.len()))?;
        for (k, v) in &self.vars {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}
