use std::collections::BTreeMap;

use reqwest::blocking::get;
use serde::{Deserialize, Serialize};

use crate::errors::MyError;

pub fn valid_did(string: &str) -> bool {
    let parts: Vec<&str> = string.split_terminator(':').collect();
    parts[0] == "did" && (parts[1].starts_with("plc") || parts[1].starts_with("web"))
}

#[tauri::command]
pub fn resolve_did(handle: &str) -> Result<String, MyError> {
    // Url::try_from(handle)

    let _atproto = format!(
        "https://dns.google/resolve?name=_atproto.{}&type=TXT",
        handle
    );
    let well_known = format!("https://{}/.well-known/atproto-did", handle);

    let response = get(well_known)
        .map_err(MyError::from)?
        .text()
        .map_err(MyError::from)?;
    if valid_did(&response) {
        return Ok(response);
    }
    #[derive(Deserialize, Debug)]
    struct DNSLookupAnswer {
        pub data: String,
    }
    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    struct DNSLookup {
        status: u8,
        answer: Option<Vec<DNSLookupAnswer>>,
    }

    let response = get(_atproto).map_err(MyError::from)?;
    let lookup = response.json::<DNSLookup>().map_err(MyError::from)?;

    if let Some(mut answer) = lookup.answer {
        let did = answer[0].data.split_off(4);
        if valid_did(&did) {
            return Ok(did);
        }
    }
    Err(MyError::NoAnswerFound(lookup.status))
}

#[derive(Deserialize, Debug)]
pub enum ServiceType {
    AtprotoPersonalDataServer,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DidService {
    pub id: String,
    #[serde(rename = "type")]
    pub service_type: ServiceType,
    pub service_endpoint: String,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DidDocument {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    pub also_known_as: Vec<String>,
    pub verification_method: Vec<serde_json::Value>,
    pub service: Vec<DidService>,
}
#[derive(Deserialize, Debug)]
pub struct PdsResponse {
    pub authorization_servers: Vec<String>,
    // pub resource: String,
    // pub scopes_supported: Vec<String>,
    // pub bearer_methods_supported: Vec<String>,
    // pub resource_documentation: String,
    #[serde(flatten)]
    _rest: BTreeMap<String, serde_json::Value>,
}

#[derive(Serialize, Debug)]
struct ResourceServers {
    pds_server: String,
    auth_server: String,
}

#[tauri::command]
pub fn get_servers(did: &str) -> Result<ResourceServers, MyError> {
    let response: DidDocument = get(format!("https://plc.directory/{}", did))
        .map_err(MyError::from)?
        .json()
        .map_err(MyError::from)?;
    let pds_server = response.service[0].service_endpoint.to_string();

    let response: PdsResponse = get(format!(
        "{}/.well-known/oauth-protected-resource",
        pds_server
    ))
    .map_err(MyError::from)?
    .json()
    .map_err(MyError::from)?;

    Ok(ResourceServers {
        pds_server,
        auth_server: response.authorization_servers[0].to_string(),
    })
}

#[derive(Serialize, Debug)]
pub struct PushedAuthorizationRequest {
    client_id: String,
    state: String,
    code_challenge: String,
    code_challenge_method: String,
    scope: String,
    redirect_uri: String,
    response_type: String,
    // login_hint: String,
}

#[derive(Deserialize, Debug)]
pub struct ParResponseSuccess {
    pub expires_in: u32,
    pub request_uri: String,
}
#[derive(Deserialize, Debug)]
pub struct ParResponseError {
    pub error: String,
    pub error_description: String,
}
#[derive(Deserialize, Debug)]
#[serde[untagged]]
pub enum ParResponse {
    Success(ParResponseSuccess),
    Error(ParResponseError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_auth_server_success() {
        let result = get_servers("did:plc:ukgwapa3bceculh4cobcopg3");
        let servers = ResourceServers {
            pds_server: String::from("https://shiitake.us-east.host.bsky.network"),
            auth_server: String::from("https://bsky.social"),
        };
        match result {
            Ok(result) => {
                assert_eq!(result.pds_server, servers.pds_server);
                assert_eq!(result.auth_server, servers.auth_server);
            }
            Err(e) => panic!("Error: {}", e),
        }
    }
}
