use serde::{Deserialize, Serialize};
use reqwest::StatusCode;

pub const MOJANG_API: &str = "https://authserver.mojang.com";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateResponse {
    pub error: Option<String>,
    pub error_message: Option<String>,
    pub access_token: Option<String>,
    pub client_token: Option<String>,
    pub available_profiles: Option<Vec<AuthenticateResponseProfile>>,
    pub selected_profile: Option<AuthenticateResponseProfile>,
    pub user: Option<AuthenticateResponseUser>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateResponseProfile {
    pub agent: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub user_id: Option<String>,
    pub created_at: Option<i64>,
    pub legacy_profile: Option<bool>,
    pub suspended: Option<bool>,
    pub paid: Option<bool>,
    pub migrated: Option<bool>,
    pub legacy: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateResponseUser {
    pub id: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub register_ip: Option<String>,
    pub migrated_from: Option<String>,
    pub migrated_at: Option<i64>,
    pub registered_at: Option<i64>,
    pub password_changed_at: Option<i64>,
    pub date_of_birth: Option<u64>,
    pub suspended: Option<bool>,
    pub blocked: Option<bool>,
    pub secured: Option<bool>,
    pub migrated: Option<bool>,
    pub email_verified: Option<bool>,
    pub legacy_user: Option<bool>,
    pub verified_by_parent: Option<bool>,
    pub properties: Option<Vec<UserProperty>>,
}

pub fn authenticate(email: &str, password: &str, client_token: &str) -> Result<Option<AuthenticateResponse>, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let request_url = format!("{api}/{path}", api = MOJANG_API, path = "authenticate");

    let json: &serde_json::Value = &serde_json::json!({
            "agent": {
                "name": "Minecraft",
                "version": 1
            },
            "username": email,
            "password": password,
            "clientToken": client_token,
            "requestUser": true
        });

    let response: serde_json::Value = client.post(&request_url)
        .header("Content-Type", "application/json")
        .json(&json)
        .send()?
        .json()?;

    let value: AuthenticateResponse = serde_json::from_str(&*response.to_string()).expect("");

    Ok(Some(value))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResponse {
    pub error: Option<String>,
    pub error_message: Option<String>,
    pub access_token: Option<String>,
    pub client_token: Option<String>,
    pub selected_profile: Option<RefreshResponseProfile>,
    pub user: Option<RefreshResponseUser>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshResponseProfile {
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshResponseUser {
    pub id: Option<String>,
    pub properties: Option<Vec<UserProperty>>,
}

pub fn refresh(access_token: &str, client_token: &str, selected_profile: &RefreshResponseProfile) -> Result<Option<RefreshResponse>, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let request_url = format!("{api}/{path}", api = MOJANG_API, path = "refresh");

    let json: &serde_json::Value = &serde_json::json!({
        "accessToken": access_token,
        "clientToken": client_token,
        "selectedProfile": {
            "id": selected_profile.id,
            "name": selected_profile.name
        },
        "requestUser": true
    });

    let response: serde_json::Value = client.post(&request_url)
        .header("Content-Type", "application/json")
        .json(&json)
        .send()?
        .json()?;

    let value: RefreshResponse = serde_json::from_str(&*response.to_string()).expect("");

    Ok(Some(value))
}

pub fn validate(access_token: &str, client_token: &str) -> Result<bool, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let request_url = format!("{api}/{path}", api = MOJANG_API, path = "validate");

    let json: &serde_json::Value = &serde_json::json!({
        "accessToken": access_token,
        "clientToken": client_token
    });

    let response = client.post(&request_url)
        .header("Content-Type", "application/json")
        .json(&json)
        .send()?;

    if response.status() == StatusCode::NO_CONTENT {
        Ok(true)
    }else{
        Ok(false)
    }
}

pub fn sign_out(email: &str, password: &str) -> Result<bool, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let request_url = format!("{api}/{path}", api = MOJANG_API, path = "signout");

    let json: &serde_json::Value = &serde_json::json!({
        "username": email,
        "password": password
    });

    let response = client.post(&request_url)
        .header("Content-Type", "application/json")
        .json(&json)
        .send()?;

    if response.status() == StatusCode::NO_CONTENT {
        Ok(true)
    }else{
        Ok(false)
    }
}

pub fn invalidate(access_token: &str, client_token: &str) -> Result<bool, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let request_url = format!("{api}/{path}", api = MOJANG_API, path = "invalidate");

    let json: &serde_json::Value = &serde_json::json!({
        "accessToken": access_token,
        "clientToken": client_token
    });

    let response = client.post(&request_url)
        .header("Content-Type", "application/json")
        .json(&json)
        .send()?;

    if response.status() == StatusCode::NO_CONTENT {
        Ok(true)
    }else{
        Ok(false)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProperty {
    pub name: String,
    pub value: String,
}