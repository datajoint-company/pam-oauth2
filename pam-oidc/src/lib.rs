// general
use std::str::FromStr;
// crate metadata
extern crate pkg_version;
use pkg_version::{
    pkg_version_major,
    pkg_version_minor,
    pkg_version_patch,
};
// pam
#[macro_use]
extern crate pamsm;
use pamsm::{
    PamServiceModule,
    PamLibExt,
    Pam,
    PamFlags,
    PamError,
};
struct PamCustom;
// yaml
extern crate yaml_rust;
use std::fs::File;
use std::io::prelude::*;
use yaml_rust::yaml::Yaml;
use yaml_rust::YamlLoader;
// json
extern crate serde_json;
use serde_json::Value;
// oauth2
extern crate base64;
extern crate oauth2;
// extern crate url;
use oauth2::{
    AuthUrl,
    ClientId,
    ClientSecret,
    ResourceOwnerPassword,
    ResourceOwnerUsername,
    Scope,
    TokenResponse,
    TokenUrl,
};
use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
// logging
extern crate log;
extern crate log4rs;
extern crate log_panics;
extern crate rand;
use log::{
    error,
    info,
    debug,
    LevelFilter,
};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{
    Appender,
    Config,
    Root,
};
use rand::Rng;
// interface
impl PamServiceModule for PamCustom {
    fn authenticate(_pamh: Pam, _flags: PamFlags, _args: Vec<String>) -> PamError {
        log_panics::init();
        // Load libpam_oidc.so's YAML config file
        let crate_version = format!("{}.{}.{}", pkg_version_major!(), pkg_version_minor!(),
                                    pkg_version_patch!());
        let config_file = &_args[0];
        let config: AppConfig = match load_config(config_file) {
            Some(c) => c,
            None => {
                error!("Error loading config file at '{}'.", config_file);
                return PamError::AUTH_ERR
            }
        };

        // Initiate logger
        let log_config = match get_log_config(&config, &crate_version) {
            Some(c) => c,
            None => {
                error!("Error loading log config.");
                return PamError::AUTH_ERR
            }
        };
        match log4rs::init_config(log_config) {
            Ok(_) => debug!("Logging successfully initialized."),
            Err(error) => debug!("Encountered error initializing log file: {}", error),
        };

        // Initialize user and password supplied
        info!("Auth detected. Proceeding...");

        let pam_user = match _pamh.get_user(None) {
            Ok(Some(u)) => match u.to_str() {
                Ok(s) => s,
                Err(e) => {
                    error!("Error converting user to string. Details: {:?}", e);
                    return PamError::USER_UNKNOWN
                }
            }
            Ok(None) => return PamError::USER_UNKNOWN,
            Err(e) => return e,
        };
        debug!("pam_user: {}", pam_user);
        let pam_password = match _pamh.get_authtok(None) {
            Ok(Some(p)) => match p.to_str() {
                Ok(s) => s,
                Err(_) => {
                    error!("Error converting password to string.");
                    return PamError::AUTH_ERR
                }
            }
            Ok(None) => return PamError::AUTH_ERR,
            Err(e) => return e,
        };
        info!("Inputs read.");
        let mut access_token = pam_password.to_string();
        if pam_password.len() as i64 > config.token_min_size {
            // If passing bearer token as password
            info!("Check as token.");
        } else {
            // If passing password directly
            info!("Check as password.");
            let client =
                BasicClient::new(
                    ClientId::new(config.client_id.clone()),
                    Some(ClientSecret::new(config.client_secret.clone())),
                    AuthUrl::new(config.url_auth.clone())
                    .unwrap(),
                    Some(TokenUrl::new(config.url_token.clone())
                    .unwrap()),
                );
            let token_result = client.exchange_password(
                &ResourceOwnerUsername::new(pam_user.to_string().clone()),
                &ResourceOwnerPassword::new(access_token.clone())
                )
                .add_scope(Scope::new(config.scopes.clone()))
                .request(http_client);
            access_token = match token_result {
                Ok(tok) => tok.access_token().secret().to_string(),
                Err(e) => {
                    error!("Wrong password provided. Details: {:?}", e);
                    return PamError::AUTH_ERR
                },
            };
        }
        // Determine assigned scopes
        debug!("access_token: {}", access_token);
        let jwt_payload = access_token.split('.').collect::<Vec<&str>>()[1];
        debug!("jwt_payload: {}", jwt_payload);
        let jwt_payload_decoded = match base64::decode(jwt_payload) {
            Ok(decoded) => decoded,
            Err(e) => {
                error!("Error decoding token. Details: {:?}", e);
                return PamError::AUTH_ERR
            },
        };
        let jwt_payload_str = match std::str::from_utf8(&jwt_payload_decoded) {
            Ok(decoded) => decoded,
            Err(e) => {
                error!("Error decoding token. Details: {:?}", e);
                return PamError::AUTH_ERR
            },
        };
        debug!("jwt_payload_str: {}", jwt_payload_str);
        let jwt_payload: Value = serde_json::from_str(&jwt_payload_str).unwrap();
        let assigned_scopes = jwt_payload.get("scope").unwrap().as_str().unwrap();
        debug!("assigned_scopes: {}", assigned_scopes);
        // Verify token
        let username: String = match verify_token(&config, &access_token) {
            Some(u) => u,
            None => {
                error!("Token invalid error.");
                return PamError::AUTH_ERR
            },
        };
        let username: &str = &username;
        let scopes_satisfied = subset(assigned_scopes, &config.scopes.clone());
        if pam_user == username && scopes_satisfied {
            //  If username defined in token, it matches pam_user, and the scopes satisfy
            info!("Auth success!");
            PamError::SUCCESS
        } else {
            debug!("username: {username}");
            debug!("pam_user: {pam_user}");
            debug!("scopes satisfied? {}", scopes_satisfied);
            error!("Auth failed!");
            PamError::AUTH_ERR
        }
    }

    fn chauthtok(_pamh: Pam, _flags: PamFlags, _args: Vec<String>) -> PamError {
        PamError::SUCCESS
    }

    fn open_session(_pamh: Pam, _flags: PamFlags, _args: Vec<String>) -> PamError {
        PamError::SUCCESS
    }

    fn close_session(_pamh: Pam, _flags: PamFlags, _args: Vec<String>) -> PamError {
        PamError::SUCCESS
    }

    fn setcred(_pamh: Pam, _flags: PamFlags, _args: Vec<String>) -> PamError {
        PamError::SUCCESS
    }

    fn acct_mgmt(_pamh: Pam, _flags: PamFlags, _args: Vec<String>) -> PamError {
        PamError::SUCCESS
    }
}

pub fn subset(parent: &str, child: &str) -> bool {
    let assigned_scopes = parent.split_whitespace().collect::<Vec<&str>>();
    let mut required_scopes = child.split_whitespace();
    let scopes_satisfied = required_scopes.all(|item| assigned_scopes.contains(&item));
    return scopes_satisfied;
}

pub fn load_file(file: &str) -> std::vec::Vec<Yaml> {
    let mut file = File::open(file).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read file");
    YamlLoader::load_from_str(&contents).unwrap()
}

#[derive(Debug)]
pub struct AppConfig {
    pub client_id: String,
    pub client_secret: String,
    pub url_auth: String,
    pub url_token: String,
    pub url_userinfo: String,
    pub scopes: String,
    pub username_key: String,
    pub token_min_size: i64,
    pub log_level: String,
    pub log_path: String,
}

pub fn parse_field(contents: &Yaml, field: &str) -> String {
    match contents[field].as_str() {
        Some(s) => s.to_string(),
        None => {
            error!("Config file is missing field '{}'. Setting to empty string.", field);
            return String::new();
        },
    }
}

pub fn load_config(file: &str) -> Option<AppConfig> {
    let contents = load_file(file);
    if contents.len() == 0 {
        error!("Config file at '{file}' is empty.");
        return None;
    }
    let conf = AppConfig {
        client_id: match contents[0]["client.id"].as_str() {
            Some(s) => s.to_string(),
            None => {
                error!("Config file at '{file}' is missing 'client.id'.");
                return None;
            },
        },
        client_secret: match contents[0]["client.secret"].as_str() {
            Some(s) => s.to_string(),
            None => {
                error!("Config file at '{file}' is missing 'client.secret'.");
                return None;
            }
        },
        url_auth: match contents[0]["url.auth"].as_str() {
            Some(s) => s.to_string(),
            None => {
                error!("Config file at '{file}' is missing 'url.auth'.");
                return None;
            }
        },
        url_token: match contents[0]["url.token"].as_str() {
            Some(s) => s.to_string(),
            None => {
                error!("Config file at '{file}' is missing 'url.token'.");
                return None;
            }
        },
        url_userinfo: match contents[0]["url.userinfo"].as_str() {
            Some(s) => s.to_string(),
            None => {
                error!("Config file at '{file}' is missing 'url.userinfo'.");
                return None;
            }
        },
        scopes: match contents[0]["scopes"].as_str() {
            Some(s) => s.to_string(),
            None => {
                error!("Config file at '{file}' is missing 'scopes'.");
                return None;
            }
        },
        username_key: match contents[0]["username.key"].as_str() {
            Some(s) => s.to_string(),
            None => {
                error!("Config file at '{file}' is missing 'username.key'.");
                return None;
            }
        },
        token_min_size: match contents[0]["token.min_size"].as_i64() {
            Some(s) => s,
            None => return None,
        },
        log_level: match contents[0]["log.level"].as_str() {
            Some(s) => s.to_string(),
            None => {
                error!("Config file at '{file}' is missing 'log.level'.");
                return None;
            }
        },
        log_path: match contents[0]["log.path"].as_str() {
            Some(s) => s.to_string(),
            None => {
                error!("Config file at '{file}' is missing 'log.path'.");
                return None;
            }
        },
    };
    Some(conf)
}


/// Initiate logger
pub fn get_log_config(config: &AppConfig, crate_version: &str) -> Option<Config> {
    let mut rng = rand::thread_rng();
    let log_id: u32 = rng.gen();
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(format!(
            "[{{d(%Y-%m-%d %H:%M:%S%.3f)}}][pam-oidc][{}][{{l}}][{}]: {{m}}{{n}}",
            crate_version,
            log_id,
        )
        .as_str()
        )))
        .build();
    let file = match FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(format!(
            "[{{d(%Y-%m-%d %H:%M:%S%.3f)}}][{}][{{l}}][{}]: {{m}}{{n}}",
            crate_version,
            log_id,
        )
        .as_str()
        )))
        .build(config.log_path.clone()) {
            Ok(f) => f,
            Err(error) => {
                error!("Encountered error initializing log file: {:?}", error);
                return None
            }
        };
    let log_config = match Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .build(
            Root::builder().appender("stdout").appender("file")
            .build(LevelFilter::from_str(&config.log_level).unwrap())
        ) {
            Ok(c) => c,
            Err(error) => {
                error!("Encountered error initializing log config: {:?}", error);
                return None
            },
        };
    Some(log_config)
}


pub fn verify_token<'a>(config: &AppConfig, access_token: &'a str) -> Option<String> {
    info!("Verifying token.");
    let resp = match reqwest::blocking::Client::new()
        .get(config.url_userinfo.clone())
        .header("Authorization", format!("Bearer {}", access_token))
        .send() {
            Ok(r) => r,
            Err(e) => {
                error!("Error sending request. Details: {:?}", e);
                return None
            },
        };
    let body = match resp.text() {
        Ok(b) => b,
        Err(e) => {
            error!("Error reading response. Details: {:?}", e);
            return None
        },
    };
    debug!("body: {}", body);
    let json: Value = match serde_json::from_str(&body) {
        Ok(j) => j,
        Err(e) => {
            error!("Error parsing JSON. Details: {:?}", e);
            return None
        },
    };
    debug!("token's user: {:?}", json.get("sub"));
    match json.get(config.username_key.clone()) {
        Some(u) => match u.as_str() {
            Some(s) => return Some(s.to_string()),
            None => {
                error!("Error parsing username from JSON.");
                return None
            },
        },
        None => {
            error!("Error parsing username from JSON.");
            return None
        },
    }
}

pam_module!(PamCustom);
