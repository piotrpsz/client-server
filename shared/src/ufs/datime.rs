use chrono::{DateTime, Local};
use serde::{Deserialize, Deserializer, Serialize};

// Żeby zastosować trait Serialize/Deserialize
// dla typu DateTime<Local> musimy go opakować.
// To opakowanie to typ 'Datime'.
#[derive(Clone)]
pub struct Datime(pub DateTime<Local>);


/********************************************************************
 *                                                                  *
 *                            S e r d e                             *
 *         Serialization/Deserialization for DateTime<Local>        *
 *                                                                  *
 ********************************************************************/
impl Serialize for Datime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i64(self.0.timestamp())
    }
}

/*
{"name":".","path":"./.","owner_id":1000,"owner_name":"piotr","group_id":1000,"group_name":"piotr",
"file_type":"Directory","size":4096,"mode":16893,"permissions":"drwxrwxr-x","last_access":174643752
0,"last_modification":1746437520,"last_status_changed":1746437520}

 */

struct I64Visitor;
impl<'de> serde::de::Visitor<'de> for I64Visitor {
    type Value = Datime;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a i64")
    }
    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        let utc = DateTime::from_timestamp(v as i64, 0).unwrap();
        let local = utc.with_timezone(&Local);
        Ok(Datime(local))
    }
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where E: serde::de::Error,
    {
        let utc = DateTime::from_timestamp(v, 0).unwrap();
        let local = utc.with_timezone(&Local);
        Ok(Datime(local))
    }
}

impl<'de> Deserialize<'de> for Datime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_i64(I64Visitor)
    }
}

