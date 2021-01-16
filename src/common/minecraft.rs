use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionManifest{
    pub latest: VersionLatest,
    pub versions: Vec<Version>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionLatest{
    pub release: String,
    pub snapshot: String
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version{
    pub id: String,
    pub r#type: String,
    pub url: String,
    pub time: String,
    pub release_time: String
}

pub fn get_version_manifest() -> Result<Option<VersionManifest>, reqwest::Error>{
    let client = reqwest::blocking::Client::new();

    let response: serde_json::Value = client.get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
        .send()?
        .json()?;

    let value: VersionManifest = serde_json::from_str(&*response.to_string()).expect("");

    Ok(Some(value))
}