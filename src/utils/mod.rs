
use crate::error::{Error, Result};

use time::format_description::well_known::Rfc3339;
use time::{OffsetDateTime, Duration};

pub fn now() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}
pub fn format(time:OffsetDateTime) -> String {
    time.format(&Rfc3339).unwrap()
}

pub fn now_plus(sec:f64) -> String {
    let new_time = now() + Duration::seconds_f64(sec);
    format(new_time)
}

pub fn parse(time_str:&str) -> Result<OffsetDateTime> {
    OffsetDateTime::parse(time_str, &Rfc3339).map_err(|_| Error::FailParseTime(time_str.to_string()))
}

pub fn encode(content:&str) -> String {
    base64_url::encode(content)
}

pub fn decode(content:&str) -> Result<String> {
    let decoded = base64_url::decode(content).ok().and_then(|v| String::from_utf8(v).ok())
        .ok_or(Error::CryptError(content.to_string()))?;
    Ok(decoded)

}