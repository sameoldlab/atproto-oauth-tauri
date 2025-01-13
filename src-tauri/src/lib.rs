use rand::Rng;
use rand::distributions::Alphanumeric;
use std::borrow::Cow;

use oauth2::PkceCodeChallenge;
use reqwest::{blocking::Client, Url};
use serde::Serialize;
use tauri::{command, Window};
use tauri_plugin_oauth::OauthConfig;

const ATPROTO_CLIENT_ID: &str = "https://tirekick.same.supply";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_oauth::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![resolve_handle, authenticate])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[command]
fn resolve_handle(handle: &str) -> &str {
    todo!()
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

#[command]
fn authenticate(window: Window, pds_url: &str) {
    let (code_challenge, code_verify) = PkceCodeChallenge::new_random_sha256();
    let client_id = "https://localhost";
    let client = Client::new();
    let state: String = rand::thread_rng().sample_iter(&Alphanumeric).take(30).map(char::from).collect();
    let port = tauri_plugin_oauth::start_with_config(
        OauthConfig {
            ports: None,
            response: Some(Cow::Borrowed(include_str!(
                "../../static/oauth_response.html"
            ))),
        },
        move |url| {
            let _url = Url::parse(&url).unwrap();
        },
    )
    .unwrap();
    let request_body = PushedAuthorizationRequest {
        client_id: "http://localhost".to_string(),
        state,
        code_challenge: code_challenge.as_str().to_string(),
        code_challenge_method: "S256".to_string(),
        scope: "atproto".to_string(),
        redirect_uri: "http://[::1]/".to_string(),
        response_type: "code".to_string(),
    };
    println!("{:#?}", request_body);

    let body = client
        .post(format!("{}/oauth/par", pds_url))
        .json(&request_body)
        .send()
        .unwrap();
    println!("{:#?}", body);

    // let tokens: serde_json::Value = body.json().unwrap();
    // println!("{:#?}", tokens);

    // let redirect_url = format!("http://localhost:{}/callback", port);
    println!("server started at {}", port);
    let auth_url = format!("{}oauth/authorize?client_id={}", pds_url, client_id,);

    // open::that(auth_url).unwrap();
}
