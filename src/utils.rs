use core::fmt;

use serde::{
    de::{self, Visitor},
    Deserializer,
};

pub fn deserialize_hex_option<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_option(HexOrIntVisitor)
}

struct HexOrIntVisitor;

impl<'de> Visitor<'de> for HexOrIntVisitor {
    type Value = Option<usize>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer or a hex string")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let v = if v.starts_with("0x") || v.starts_with("0X") {
            &v[2..]
        } else {
            &v
        };
        match usize::from_str_radix(v, 16) {
            Ok(num) => Ok(Some(num)),
            Err(e) => {
                dbg!(e);
                Err(E::custom(format!("Invalid hex string: {}", v)))
            }
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&v)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v >= i64::MIN && v <= i64::MAX {
            Ok(Some(v as usize))
        } else {
            Err(E::custom(format!("Integer out of range: {}", v)))
        }
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v <= u64::MAX {
            Ok(Some(v as usize))
        } else {
            Err(E::custom(format!("Integer out of range: {}", v)))
        }
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let int_val = v as i64;
        self.visit_i64(int_val)
    }
}
