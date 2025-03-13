use crate::errors::MyError;
use reqwest::blocking::get;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
pub fn get_oauth_auth_server(server_uri: &str) -> Result<OauthAuthorizationServer, MyError> {
    let response = get(format!(
        "{server_uri}/.well-known/oauth-authorization-server"
    ))
    .map_err(MyError::from)?
    .json::<OauthAuthorizationServer>()
    .map_err(MyError::from)?;
    if response.issuer != server_uri {

        return Err(MyError::ValidationError(
            format!("Invalid issuer: {} is not equal to {}", response.issuer, server_uri),
        ));
    }
    Ok(response)
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

    #[serde(flatten)]
    pub _rest: BTreeMap<String, serde_json::Value>
}
#[derive(Deserialize, Debug)]
pub struct PdsResponse {
    pub authorization_servers: Vec<String>,
    pub resource: String,
    pub scopes_supported: Vec<String>,
    pub bearer_methods_supported: Vec<String>,
    pub resource_documentation: String,
    #[serde(flatten)]
    pub _rest: BTreeMap<String, serde_json::Value>
}

#[derive(Serialize, Debug)]
pub struct ResourceServers {
    pub pds_server: String,
    pub auth_server: String,
}

#[derive(Serialize, Debug)]
pub struct PushedAuthorizationRequest {
    pub client_id: String,
    pub state: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub scope: String,
    pub redirect_uri: String,
    pub response_type: String,
    // login_hint: String,
}

#[derive(Deserialize, Debug)]
pub struct ParResponseSuccess {
    // pub expires_in: u32,
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

#[derive(Deserialize, Debug)]
pub struct OauthAuthorizationServer {
    pub issuer: String,
    pub scopes_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub response_modes_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>,
    pub ui_locales_supported: Vec<String>,
    pub display_values_supported: Vec<String>,
    pub authorization_response_iss_parameter_supported: bool,
    pub request_object_signing_alg_values_supported: Vec<String>,
    pub request_object_encryption_alg_values_supported: Vec<String>,
    pub request_object_encryption_enc_values_supported: Vec<String>,
    pub request_parameter_supported: bool,
    pub request_uri_parameter_supported: bool,
    pub require_request_uri_registration: bool,
    pub jwks_uri: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub token_endpoint_auth_signing_alg_values_supported: Vec<String>,
    pub revocation_endpoint: String,
    pub introspection_endpoint: String,
    pub pushed_authorization_request_endpoint: String,
    pub require_pushed_authorization_requests: bool,
    pub dpop_signing_alg_values_supported: Vec<String>,
    pub client_id_metadata_document_supported: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_path_to_error::deserialize;
    use serde_json::{self, Deserializer};

    #[test]
    fn decode_did_doc() {
        let response = get(format!("https://plc.directory/did:plc:ukgwapa3bceculh4cobcopg3"))
            .map_err(MyError::from).unwrap()
            .text().unwrap();
        let d = &mut Deserializer::from_str(response.as_str());
        let result: Result<DidDocument, _> = deserialize(d);
        match result {
                Ok(_) => (),
                Err(err) => {
                    let path = err.path().to_string();
                    let json_value: serde_json::Value = serde_json::from_str(response.as_str()).unwrap();
                    let value_at_path = path.split('.')
                        .fold(Some(&json_value), |acc, key| {
                            acc.and_then(|v| v.get(key))
                        });

                    panic!("Parse error at path '{:?}': {}\nValue at path: {:?}",
                        path, err, value_at_path);
                }
            }
    }

    #[test]
    fn decode_pds() {
        let response: DidDocument = get(format!("https://plc.directory/did:plc:ukgwapa3bceculh4cobcopg3"))
            .map_err(MyError::from).unwrap()
            .json()
            .map_err(MyError::from).unwrap();
        let pds_server = response.service[0].service_endpoint.to_string();

        let response: PdsResponse = get(format!(
            "{}/.well-known/oauth-protected-resource",
            pds_server
        ))
        .map_err(MyError::from).unwrap()
        .json()
        .map_err(MyError::from).unwrap();
    }

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
