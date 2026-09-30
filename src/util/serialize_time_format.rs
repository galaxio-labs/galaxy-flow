use serde::Serialize;
use serde::ser::{self, Serializer};
use time::{OffsetDateTime, format_description};

// 序列化时将时间格式化
pub fn serialize_time_format<S>(value: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let format =
        format_description::parse_borrowed::<3>("[year]-[month]-[day] [hour]:[minute]:[second]")
            .expect("Invalid datetime format string"); // 硬编码格式应该总是有效

    value
        .format(&format)
        .map_err(|_| ser::Error::custom("Failed to format datetime"))?
        .serialize(serializer)
}
