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
use oauth2::basic::{
    BasicClient
};
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
        // Load libpam_oidc.so's YAML config file
        let crate_version = format!("{}.{}.{}", pkg_version_major!(), pkg_version_minor!(),
                                    pkg_version_patch!());
        let config_file = &_args[0];
        let config = load_file(config_file);
        // Initiate logger
        let mut rng = rand::thread_rng();
        let log_id: u32 = rng.gen();
        log_panics::init();
        let stdout = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new(format!(
                "[{{d(%Y-%m-%d %H:%M:%S%.3f)}}][pam-oidc][{}][{{l}}][{}]: {{m}}{{n}}",
                crate_version,
                log_id,
            )
            .as_str()
            )))
            .build();
        let file = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(format!(
                "[{{d(%Y-%m-%d %H:%M:%S%.3f)}}][{}][{{l}}][{}]: {{m}}{{n}}",
                crate_version,
                log_id,
            )
            .as_str()
            )))
            .build(config[0]["log.path"].as_str().unwrap().to_string())
            .unwrap();
        let log_config = Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .appender(Appender::builder().build("file", Box::new(file)))
            .build(
                Root::builder().appender("stdout").appender("file")
                .build(LevelFilter::from_str(&config[0]["log.level"].as_str().unwrap()
                .to_string()).unwrap())
            )
            .unwrap();
        match log4rs::init_config(log_config) {
            Ok(_) => debug!("Logging successfully initialized."),
            Err(error) => debug!("Encountered error initializing log file: {}", error),
        };
        // Initialize user and password supplied
        info!("Auth detected. Proceeding...");
        let pam_user = match _pamh.get_user(None) {
            Ok(Some(u)) => u.to_str().unwrap(),
            Ok(None) => return PamError::USER_UNKNOWN,
            Err(e) => return e,
        };
        debug!("pam_user: {}", pam_user);
        let pam_password = match _pamh.get_authtok(None) {
            Ok(Some(p)) => p.to_str().unwrap(),
            Ok(None) => return PamError::AUTH_ERR,
            Err(e) => return e,
        };
        debug!("config_file: {}", config_file);
        debug!("client id: {}", config[0]["client.id"].as_str().unwrap());
        debug!("client secret: {}", config[0]["client.secret"].as_str().unwrap());
        debug!("url auth: {}", config[0]["url.auth"].as_str().unwrap());
        debug!("url token: {}", config[0]["url.token"].as_str().unwrap());
        debug!("url userinfo: {}", config[0]["url.userinfo"].as_str().unwrap());
        debug!("input min_size: {}", config[0]["token.min_size"].as_i64().unwrap());
        debug!("requested scopes: {}", config[0]["scopes"].as_str().unwrap().to_string());
        debug!("pam_password: {}", pam_password);
        debug!("actual pass_size: {}", pam_password.len() as i64);
        info!("Inputs read.");
        let mut access_token = pam_password.to_string();
        if pam_password.len() as i64 > config[0]["token.min_size"].as_i64().unwrap() {
            // If passing bearer token as password
            info!("Check as token.");
        } else {
            // If passing password directly
            info!("Check as password.");
            let client =
                BasicClient::new(
                    ClientId::new(config[0]["client.id"].as_str().unwrap().to_string()),
                    Some(ClientSecret::new(
                        config[0]["client.secret"].as_str().unwrap().to_string()
                    )),
                    AuthUrl::new(String::from(config[0]["url.auth"].as_str().unwrap()))
                    .unwrap(),
                    Some(TokenUrl::new(
                        String::from(config[0]["url.token"].as_str().unwrap())
                    )
                    .unwrap()),
                );
            let token_result = client.exchange_password(
                &ResourceOwnerUsername::new(pam_user.to_string().clone()),
                &ResourceOwnerPassword::new(pam_password.to_string())
                )
                .add_scope(Scope::new(config[0]["scopes"].as_str().unwrap().to_string()))
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
        let jwt_payload_decoded = base64::decode(jwt_payload).unwrap();
        let jwt_payload_str = std::str::from_utf8(&jwt_payload_decoded).unwrap();
        debug!("jwt_payload_str: {}", jwt_payload_str);
        let jwt_payload: Value = serde_json::from_str(&jwt_payload_str).unwrap();
        let assigned_scopes = jwt_payload.get("scope").unwrap().as_str().unwrap();
        debug!("assigned_scopes: {}", assigned_scopes);
        // Verify token
        info!("Verifying token.");
        let body = reqwest::blocking::Client::new()
            .get(config[0]["url.userinfo"].as_str().unwrap())
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .unwrap()
            .text()
            .unwrap();
        debug!("body: {}", body);
        let json: Value = serde_json::from_str(&body).unwrap();
        debug!("token's user: {:?}", json.get("sub"));
        if json.get(config[0]["username.key"].as_str().unwrap().to_string()) != None &&
                pam_user == json[config[0]["username.key"].as_str().unwrap().to_string()]
                    .as_str().unwrap() &&
                config[0]["scopes"].as_str().unwrap().to_string() == assigned_scopes {
            //  If username defined in token, it matches pam_user, and the scopes satisfy
            info!("Auth success!");
            PamError::SUCCESS
        } else if json.get(config[0]["username.key"].as_str().unwrap().to_string()) == None {
            // If username not defined in token
            error!("Token invalid error.");
            PamError::AUTH_ERR
        } else {
            debug!("user defined? {}", json.get(
                config[0]["username.key"].as_str().unwrap().to_string()) != None);
            debug!("user matches? {}", pam_user == json[
                config[0]["username.key"].as_str().unwrap().to_string()].as_str().unwrap());
            debug!("scopes satisfied? {}", config[0]["scopes"].as_str().unwrap()
                .to_string() == assigned_scopes);
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

pub fn load_file(file: &str) -> std::vec::Vec<Yaml> {
    let mut file = File::open(file).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read file");
    YamlLoader::load_from_str(&contents).unwrap()
}

pam_module!(PamCustom);
