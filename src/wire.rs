//! Private response envelopes and serde helpers shared by the resource modules.

use serde::{Deserialize, Deserializer};

/// `{ "status": "...", "message": "..." }`: responses whose only payload is a
/// message.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct MessageResponse {
    pub(crate) message: String,
}

/// `{ "status": "...", "data": T }`: the envelope used by the newer routes
/// (env, projects, databases, teams).
#[derive(Debug, Deserialize)]
pub(crate) struct DataResponse<T> {
    pub(crate) data: T,
}

/// Accepts a JSON string or number and returns it as a string. Some IDs
/// (database backup IDs) are documented as strings but sent as numbers.
pub(crate) fn string_or_number<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
        String(String),
        Number(serde_json::Number),
    }
    Ok(match Raw::deserialize(d)? {
        Raw::String(s) => s,
        Raw::Number(n) => n.to_string(),
    })
}

/// Treats an explicit JSON `null` like a missing field and falls back to
/// `T::default()`.
pub(crate) fn null_as_default<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(d)?.unwrap_or_default())
}
