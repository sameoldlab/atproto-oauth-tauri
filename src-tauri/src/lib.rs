mod atproto_oauth;
mod errors;

use atproto_oauth::{get_servers, resolve_did, valid_did, ParResponse};
use errors::MyError;
use oauth2::{PkceCodeChallenge, RedirectUrl};
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
    let scope = "atproto transtition:generic";
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
        "scope": scope,
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
        .append_pair("scope", scope)
        .append_pair("request_uri", &request_uri);
    println!("{:?}", auth_endpoint.as_str());
    open::that(auth_endpoint.as_str()).unwrap();
    Ok(())
}


}
