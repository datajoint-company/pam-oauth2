// general
use std::str::FromStr;
// crate metadata
extern crate pkg_version;
use pkg_version::{pkg_version_major, pkg_version_minor, pkg_version_patch};
// pam
#[macro_use]
extern crate pamsm;
use pamsm::{Pam, PamError, PamFlags, PamLibExt, PamServiceModule};
struct PamCustom;
// yaml
extern crate yaml_rust;
use std::fs::File;
use std::io::prelude::*;
use yaml_rust::yaml::Yaml;
use yaml_rust::ScanError;
use yaml_rust::YamlLoader;
// json
extern crate serde_json;
use serde_json::Value;
// oauth2
extern crate base64;
extern crate oauth2;
// extern crate url;
use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use oauth2::{
    AuthUrl, ClientId, ClientSecret, ResourceOwnerPassword, ResourceOwnerUsername, Scope,
    TokenResponse, TokenUrl,
};
// logging
extern crate log;
extern crate log4rs;
extern crate log_panics;
extern crate rand;
use log::{debug, error, info, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use rand::Rng;
// interface
impl PamServiceModule for PamCustom {
    fn authenticate(_pamh: Pam, _flags: PamFlags, _args: Vec<String>) -> PamError {
        log_panics::init();
        // Load libpam_oidc.so's YAML config file
        let crate_version = format!(
            "{}.{}.{}",
            pkg_version_major!(),
            pkg_version_minor!(),
            pkg_version_patch!()
        );
        let config_file = &_args[0];
        let config: AppConfig = match load_config(config_file) {
            Some(c) => c,
            None => {
                error!("Error loading config file at '{}'.", config_file);
                return PamError::AUTH_ERR;
            }
        };

        // Initiate logger
        let log_config = match get_log_config(&config, &crate_version) {
            Some(c) => c,
            None => {
                error!("Error loading log config.");
                return PamError::AUTH_ERR;
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
                    return PamError::USER_UNKNOWN;
                }
            },
            Ok(None) => return PamError::USER_UNKNOWN,
            Err(e) => return e,
        };
        debug!("pam_user: {}", pam_user);
        let pam_password = match _pamh.get_authtok(None) {
            Ok(Some(p)) => match p.to_str() {
                Ok(s) => s,
                Err(_) => {
                    error!("Error converting password to string.");
                    return PamError::AUTH_ERR;
                }
            },
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
            access_token = match get_token_oidc(&config, &pam_password, &pam_user) {
                Some(tok) => tok,
                None => {
                    info!("Wrong password provided.");
                    return PamError::AUTH_ERR;
                }
            };
        }
        // Determine assigned scopes
        let assigned_scopes = match get_assigned_scopes(&access_token) {
            Some(s) => s,
            None => {
                error!("Token is missing scopes.");
                return PamError::AUTH_ERR;
            }
        };
        let assigned_scopes: &str = &assigned_scopes;
        debug!("assigned_scopes: {}", assigned_scopes);
        // Verify token
        let username: String = match verify_token(&config, &access_token) {
            Some(u) => u,
            None => {
                info!("Token invalid.");
                return PamError::AUTH_ERR;
            }
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

pub fn load_file(file: &str) -> Result<std::vec::Vec<Yaml>, ScanError> {
    let mut file = File::open(file).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");
    YamlLoader::load_from_str(&contents)
}

#[derive(Debug)]
struct AppConfig {
    client_id: String,
    client_secret: String,
    url_auth: String,
    url_token: String,
    url_userinfo: String,
    scopes: String,
    username_key: String,
    token_min_size: i64,
    log_level: String,
    log_path: String,
}

fn load_config(file: &str) -> Option<AppConfig> {
    let contents = match load_file(file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "[{}:{}] ERROR: Error loading config file at '{}'. Details: {:?}",
                file!(),
                line!(),
                file,
                e
            );
            return None;
        }
    };
    if contents.len() == 0 {
        eprintln!(
            "[{}:{}] ERROR: Config file at '{file}' is empty.",
            file!(),
            line!()
        );
        return None;
    }
    let conf = AppConfig {
        client_id: match contents[0]["client.id"].as_str() {
            Some(s) => s.to_string(),
            None => {
                eprintln!(
                    "[{}:{}] ERROR: Config file at '{file}' is missing 'client.id'.",
                    file!(),
                    line!()
                );
                return None;
            }
        },
        client_secret: match contents[0]["client.secret"].as_str() {
            Some(s) => s.to_string(),
            None => {
                eprintln!(
                    "[{}:{}] ERROR: Config file at '{file}' is missing 'client.secret'.",
                    file!(),
                    line!()
                );
                return None;
            }
        },
        url_auth: match contents[0]["url.auth"].as_str() {
            Some(s) => s.to_string(),
            None => {
                eprintln!(
                    "[{}:{}] ERROR: Config file at '{file}' is missing 'url.auth'.",
                    file!(),
                    line!()
                );
                return None;
            }
        },
        url_token: match contents[0]["url.token"].as_str() {
            Some(s) => s.to_string(),
            None => {
                eprintln!(
                    "[{}:{}] ERROR: Config file at '{file}' is missing 'url.token'.",
                    file!(),
                    line!()
                );
                return None;
            }
        },
        url_userinfo: match contents[0]["url.userinfo"].as_str() {
            Some(s) => s.to_string(),
            None => {
                eprintln!(
                    "[{}:{}] ERROR: Config file at '{file}' is missing 'url.userinfo'.",
                    file!(),
                    line!()
                );
                return None;
            }
        },
        scopes: match contents[0]["scopes"].as_str() {
            Some(s) => s.to_string(),
            None => {
                eprintln!(
                    "[{}:{}] ERROR: Config file at '{file}' is missing 'scopes'.",
                    file!(),
                    line!()
                );
                return None;
            }
        },
        username_key: match contents[0]["username.key"].as_str() {
            Some(s) => s.to_string(),
            None => {
                eprintln!(
                    "[{}:{}] ERROR: Config file at '{file}' is missing 'username.key'.",
                    file!(),
                    line!()
                );
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
                eprintln!(
                    "[{}:{}] ERROR: Config file at '{file}' is missing 'log.level'.",
                    file!(),
                    line!()
                );
                return None;
            }
        },
        log_path: match contents[0]["log.path"].as_str() {
            Some(s) => s.to_string(),
            None => {
                eprintln!(
                    "[{}:{}] ERROR: Config file at '{file}' is missing 'log.path'.",
                    file!(),
                    line!()
                );
                return None;
            }
        },
    };
    Some(conf)
}

/// Initiate logger
fn get_log_config(config: &AppConfig, crate_version: &str) -> Option<Config> {
    let mut rng = rand::thread_rng();
    let log_id: u32 = rng.gen();
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            format!(
                "[{{d(%Y-%m-%d %H:%M:%S%.3f)}}][pam-oidc][{}][{{l}}][{}]: {{m}}{{n}}",
                crate_version, log_id,
            )
            .as_str(),
        )))
        .build();
    let file = match FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            format!(
                "[{{d(%Y-%m-%d %H:%M:%S%.3f)}}][{}][{{l}}][{}]: {{m}}{{n}}",
                crate_version, log_id,
            )
            .as_str(),
        )))
        .build(config.log_path.clone())
    {
        Ok(f) => f,
        Err(error) => {
            error!("Encountered error initializing log file: {:?}", error);
            return None;
        }
    };
    let level = match LevelFilter::from_str(&config.log_level) {
        Ok(l) => l,
        Err(error) => {
            error!("Encountered error initializing log level: {:?}", error);
            return None;
        }
    };
    let log_config = match Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("file")
                .build(level),
        ) {
        Ok(c) => c,
        Err(error) => {
            error!("Encountered error initializing log config: {:?}", error);
            return None;
        }
    };
    Some(log_config)
}

fn verify_token(config: &AppConfig, access_token: &str) -> Option<String> {
    info!("Verifying token.");
    let resp = match reqwest::blocking::Client::new()
        .get(config.url_userinfo.clone())
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
    {
        Ok(r) => r,
        Err(e) => {
            error!("Error sending request. Details: {:?}", e);
            return None;
        }
    };
    let body = match resp.text() {
        Ok(b) => b,
        Err(e) => {
            error!("Error reading response. Details: {:?}", e);
            return None;
        }
    };
    debug!("body: {}", body);
    let json: Value = match serde_json::from_str(&body) {
        Ok(j) => j,
        Err(e) => {
            if !e.to_string().contains("EOF while parsing a value") {
                error!("Error parsing body as JSON. Details: {:?}", e);
            }
            return None;
        }
    };
    debug!("token's user: {:?}", json.get("sub"));
    match json.get(config.username_key.clone()) {
        Some(u) => match u.as_str() {
            Some(s) => return Some(s.to_string()),
            None => {
                error!("Error parsing username from JSON.");
                return None;
            }
        },
        None => {
            error!("Error parsing username from JSON.");
            return None;
        }
    }
}

fn get_token_oidc(config: &AppConfig, passwd: &str, pam_user: &str) -> Option<String> {
    let client = BasicClient::new(
        ClientId::new(config.client_id.clone()),
        Some(ClientSecret::new(config.client_secret.clone())),
        match AuthUrl::new(config.url_auth.clone()) {
            Ok(u) => u,
            Err(e) => {
                error!("Error parsing auth url. Details: {:?}", e);
                return None;
            }
        },
        match TokenUrl::new(config.url_token.clone()) {
            Ok(u) => Some(u),
            Err(e) => {
                error!("Error parsing token url. Details: {:?}", e);
                return None;
            }
        },
    );
    let token_result = client
        .exchange_password(
            &ResourceOwnerUsername::new(pam_user.to_string()),
            &ResourceOwnerPassword::new(passwd.to_string()),
        )
        .add_scope(Scope::new(config.scopes.clone()))
        .request(http_client);
    match token_result {
        Ok(tok) => Some(tok.access_token().secret().to_string()),
        Err(e) => {
            if !e.to_string().contains("Server returned error response") {
                error!("Error getting token. Details: {}", e.to_string());
            }
            return None;
        }
    }
}

fn get_assigned_scopes(access_token: &str) -> Option<String> {
    debug!("access_token: {}", access_token);
    let parts = access_token.split('.').collect::<Vec<&str>>();
    if parts.len() < 2 {
        error!("Token is improperly formatted.");
        return None;
    }
    let jwt_payload = parts[1];
    debug!("jwt_payload: {}", jwt_payload);
    let jwt_payload_decoded = match base64::decode(jwt_payload) {
        Ok(decoded) => decoded,
        Err(e) => {
            error!("Error decoding token. Details: {:?}", e);
            return None;
        }
    };
    let jwt_payload_str = match std::str::from_utf8(&jwt_payload_decoded) {
        Ok(decoded) => decoded,
        Err(e) => {
            error!("Error decoding token. Details: {:?}", e);
            return None;
        }
    };
    debug!("jwt_payload_str: {}", jwt_payload_str);
    let jwt_payload: Value = match serde_json::from_str(&jwt_payload_str) {
        Ok(decoded) => decoded,
        Err(e) => {
            error!("Error parsing JSON. Details: {:?}", e);
            return None;
        }
    };
    match jwt_payload.get("scope") {
        Some(s) => match s.as_str() {
            Some(s) => return Some(s.to_string()),
            None => {
                error!("Error parsing scopes from token.");
                return None;
            }
        },
        None => {
            error!("Error parsing scopes from token.");
            return None;
        }
    }
}

pam_module!(PamCustom);
