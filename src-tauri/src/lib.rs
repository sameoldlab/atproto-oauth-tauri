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
use tauri::{command, Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_oauth::OauthConfig;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
          println!("a new app instance was opened with {args:?} and the deep link event was already triggered");
          let _ = app.get_webview_window("main")
           .expect("no main window")
           .set_focus();
        }));
    }
    builder
        .plugin(tauri_plugin_deep_link::init())
        // .plugin(tauri_plugin_sql::Builder::new().build())
        // .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_oauth::init())
        // .plugin(tauri_plugin_fs::init())
        // .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(any(target_os = "linux", all(debug_assertions, windows)))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                app.deep_link().register_all()?;
            }
            app.deep_link().on_open_url(|event| {
                println!("deep link URLs: {:?}", event.urls());
            });
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
fn authenticate(auth_url: &str, handle: Option<&str>) -> Result<(), MyError> {
    let state: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    let auth_server = Url::parse(auth_url).unwrap();
    let auth_url2 = auth_url.to_string();
    let state2 = state.to_string();
    let (sender, recv) = std::sync::mpsc::channel::<Result<String, MyError>>();
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
            match query_pairs.get("iss") {
                Some(issuer) => {
                    if issuer != &auth_url2 {
                        sender
                            .send(Err(MyError::ValidationError(
                                "Invalid issuer: {iss}".to_string(),
                            )))
                            .unwrap();
                        return;
                    }
                }
                None => {
                    sender
                        .send(Err(MyError::ValidationError("No issuer found".to_string())))
                        .unwrap();
                    return;
                }
            }
            match query_pairs.get("state") {
                Some(qstate) => {
                    if qstate != &state2 {
                        sender
                            .send(Err(MyError::ValidationError(
                                "Mismatched state".to_string(),
                            )))
                            .unwrap();
                        return;
                    }
                }
                None => {
                    sender
                        .send(Err(MyError::ValidationError("No state found".to_string())))
                        .unwrap();
                    return;
                }
            }
            if let (Some(error), Some(description)) = (
                query_pairs.get("error"),
                query_pairs.get("error_description"),
            ) {
                sender
                    .send(Err(MyError::AuthError {
                        err_type: error.to_owned(),
                        message: description.to_owned(),
                    }))
                    .unwrap();
                return;
            }
            if let Some(code) = query_pairs.get("code") {
                sender.send(Ok(code.to_owned())).unwrap();
                return;
            }
        },
    )
    .unwrap();

    let client_id = String::from("http://localhost");
    let (code_challenge, code_verify) = PkceCodeChallenge::new_random_sha256();
    let client = Client::new();
    let scope = "atproto"; //transtition:generic";

    let redirect_uri = format!("http://127.0.0.1:{port}");
    let request_body = serde_json::json!({
        "client_id": format!("{client_id}"),
        "state": &state,
        "code_challenge": code_challenge.as_str(),
        "redirect_uri": redirect_uri,
        "code_challenge_method": code_challenge.method(),
        "scope": scope,
        "response_type": "code",
        "application_type": "native",
        "dpop_bound_access_tokens": true,
        "grant_types": ["authorization_code", "refresh_token"],
        "login_hint": handle
    });

    let par_endpoint = auth_server.join("/oauth/par").unwrap();
    let par_response = client
        .post(par_endpoint)
        .json(&request_body)
        .send()
        .map_err(MyError::from)?
        .json()
        .map_err(MyError::from)?;

    match par_response {
        ParResponse::Success(data) => {
            let mut auth_endpoint = auth_server.join("oauth/authorize").unwrap();
            auth_endpoint
                .query_pairs_mut()
                .append_pair("client_id", &client_id)
                .append_pair("redirect_uri", &redirect_uri)
                .append_pair("scope", scope)
                .append_pair("request_uri", &data.request_uri);
            println!("{:?}", auth_endpoint.as_str());
            open::that(auth_endpoint.as_str()).unwrap();
        }
        ParResponse::Error(error) => return Err(MyError::from(error)),
    }
    let message = recv.recv().unwrap();
    drop(recv);
    match message {
        Err(err) => return Err(err),
        Ok(code) => {
            // make initial token requet to auth server token endpoint with `code` and pkce code verification
            // this request uses DPoP, with the previous authorization server nonce
            // returns JSON {
            // access_token, refresh_token, sub, scope
            // }
            // sub field must match user provided did
            Ok(())
        }
    }
}
