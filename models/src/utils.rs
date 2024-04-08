use serde::{Deserialize, Deserializer, Serializer};
use std::time::Duration;

pub fn deserialize_duration_from_unix_timestamp<'de, D>(
    deserializer: D,
) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s = u64::deserialize(deserializer)?;
    Ok(Duration::from_millis(s))
}

pub fn serialize_duration_in_millis<S>(
    duration: &Duration,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let millis = duration.as_millis();
    if millis > u64::MAX as u128 {
        use serde::ser::Error;
        Err(S::Error::custom("Duration is too large to fit in a u64"))
    } else {
        serializer.serialize_u64(millis as u64)
    }
}
