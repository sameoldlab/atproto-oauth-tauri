use oauth2::PkceCodeChallenge;
use rand::distributions::Alphanumeric;
use rand::Rng;
use reqwest::{
    blocking::{get, Client},
    Url,
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
};
use tauri::command;
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_oauth::OauthConfig;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        // .plugin(tauri_plugin_sql::Builder::new().build())
        // .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_oauth::init())
        // .plugin(tauri_plugin_fs::init())
        // .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(desktop)]
            app.deep_link().register("tirekick")?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            resolve_did,
            authenticate,
            get_servers
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Deserialize, Debug)]
pub struct DNSLookupAnswer {
    pub data: String,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DNSLookup {
    pub status: u8,
    pub answer: Option<Vec<DNSLookupAnswer>>,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Request failed: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Failed to parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("No answer found in DNS response")]
    NoAnswerFound(u8),
}
impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::JsonError {
            message: error.to_string(),
            classify: error.classify(),
            column: error.column(),
            line: error.line(),
        }
    }
}

#[command]
fn resolve_did(handle: &str) -> Result<String, Error> {
    // Url::try_from(handle)

    let _atproto = format!(
        "https://dns.google/resolve?name=_atproto.{}&type=TXT",
        handle
    );
    let well_known = format!("https://{}/.well-known/atproto-did", handle);

    let response2 = get(well_known)
        .map_err(Error::from)?
        .text()
        .map_err(Error::from)?;
    if valid_did(&response2) {
        return Ok(response2);
    }
    let response = get(_atproto).map_err(Error::from)?;
    let lookup = response.json::<DNSLookup>().map_err(Error::from)?;

    if let Some(mut answer) = lookup.answer {
        let did = answer[0].data.split_off(4);
        if valid_did(&did) {
            return Ok(did);
        }
    }
    Err(Error::NoAnswerFound(lookup.status))
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

#[command]
fn get_servers(did: &str) -> Result<ResourceServers, Error> {
    let response: DidDocument = get(format!("https://plc.directory/{}", did))
        .map_err(Error::from)?
        .json()
        .map_err(Error::from)?;
    let pds_server = response.service[0].service_endpoint.to_string();

    let req2: PdsResponse = get(format!(
        "{}/.well-known/oauth-protected-resource",
        pds_server
    ))
    .map_err(Error::from)?
    .json()
    .map_err(Error::from)?;

    Ok(ResourceServers {
        pds_server,
        auth_server: req2.authorization_servers[0].to_string(),
    })
}

fn valid_did(string: &str) -> bool {
    let parts: Vec<&str> = string.split_terminator(':').collect();
    if parts[0] != "did" {
        return false;
    }
    if parts[1].starts_with("plc") || parts[1].starts_with("web") {
        return true;
    }
    false
}
#[derive(Serialize, Debug)]
struct PushedAuthorizationRequest {
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
pub struct ParResponse {
    pub expires_in: u32,
    pub request_uri: String,
}

#[command]
fn authenticate(auth_url: &str) -> Result<(), Error> {
    let port = tauri_plugin_oauth::start_with_config(
        OauthConfig {
            ports: None,
            response: Some(Cow::Borrowed(include_str!(
                "../../static/oauth_response.html"
            ))),
        },
        move |url| {
            let url = Url::parse(&url).unwrap();
            let query_pairs: HashMap<String, String> = url.query_pairs().into_owned().collect();
            println!("{:?}", url);
        },
    )
    .unwrap();

    let auth_server = Url::parse(auth_url).unwrap();
    let client_id = String::from("http://localhost");
    let (code_challenge, code_verify) = PkceCodeChallenge::new_random_sha256();
    let client = Client::new();
    let state: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    let redirect_uri = format!("http://127.0.0.1:{port}");
    let request_body = serde_json::json!({
        "client_id": format!("{client_id}"),
        "state": state,
        "code_challenge": code_challenge.as_str(),
        "redirect_uri": redirect_uri,
        "code_challenge_method": code_challenge.method(),
        "scope": "atproto",
        "response_type": "code",
        "application_type": "native",
        "dpop_bound_access_tokens": true,
        "grant_types": ["authorization_code", "refresh_token"],
    });
    println!("{:#?}", request_body);

    let par_endpoint = auth_server.join("/oauth/par").unwrap();
    let par_response = client
        .post(par_endpoint)
        .json(&request_body)
        .send()
        .unwrap()
        .json::<ParResponse>()
        .map_err(Error::from)?;

    let request_uri = par_response.request_uri;

    let mut auth_endpoint = auth_server.join("oauth/authorize").unwrap();
    auth_endpoint
        .query_pairs_mut()
        .append_pair("client_id", &client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("scope", &"atproto")
        .append_pair("request_uri", &request_uri);
    println!("{:?}", auth_endpoint.as_str());
    open::that(auth_endpoint.as_str()).unwrap();
    Ok(())
}


}
