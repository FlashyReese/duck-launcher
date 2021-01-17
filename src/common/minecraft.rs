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

pub async fn get_version_manifest() -> Result<Option<VersionManifest>, reqwest::Error>{
    let client = reqwest::Client::new();

    let response = client.get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
        .send()
        .await?;

    let value: VersionManifest = response.json().await?;

    Ok(Some(value))
}