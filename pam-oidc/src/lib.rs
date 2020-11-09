#[macro_use]
extern crate pamsm;
use pamsm::{PamServiceModule, PamLibExt, Pam, PamFlag, PamError};
struct PamCustom;
// YAML
extern crate yaml_rust;
use std::fs::File;
use std::io::prelude::*;
use yaml_rust::yaml::{Yaml};
use yaml_rust::YamlLoader;
// OAUTH2
extern crate base64;
extern crate oauth2;
extern crate rand;
extern crate url;
// use oauth2::prelude::*;
use oauth2::{
    AuthUrl,
    ClientId,
    ClientSecret,
    ResourceOwnerPassword,
    ResourceOwnerUsername,
    Scope,
    TokenResponse,
    TokenUrl
};
use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use url::Url;
// CURL
// extern crate curl;
// use curl::easy::Easy;
extern crate serde_json;
use serde_json::Value;
// LOGGING
extern crate log;
use log::{error, info, warn, LevelFilter};
extern crate log4rs;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};
extern crate log_panics;
impl PamServiceModule for PamCustom {
    fn authenticate(_pamh: Pam, _flags: PamFlag, _args: Vec<String>) -> PamError {

        log_panics::init();
        let stdout = ConsoleAppender::builder().build();
        let file = FileAppender::builder()
            // .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
            .build("/tmp/pam_oidc.log")
            .unwrap();
        let config = Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .appender(Appender::builder().build("file", Box::new(file)))
            .build(Root::builder().appender("stdout").appender("file").build(LevelFilter::Info))
            .unwrap();
        log4rs::init_config(config);
        
        info!("Auth detected. Proceeding...\n");
        let pam_user = match _pamh.get_user(None) {
            Ok(Some(u)) => u.to_str().unwrap(),
            Ok(None) => return PamError::USER_UNKNOWN,
            Err(e) => return e,
        };
        // println!("pam_user: {}", pam_user);
        let pam_password = match _pamh.get_authtok(None) {
            Ok(Some(p)) => p.to_str().unwrap(),
            Ok(None) => return PamError::AUTH_ERR,
            Err(e) => return e,
        };
        // let pam_password = String::from("shh");
        // println!("pam_password: {}", pam_password);
        // println!("pam_password Length: {}", pam_password.len());
        let config_file = &_args[0];
        let config = load_file(config_file);
        // println!("config_file: {}", config_file);
        // println!("client id: {}", config[0]["client.id"].as_str().unwrap());
        // println!("client secret: {}", config[0]["client.secret"].as_str().unwrap());
        // println!("url auth: {}", config[0]["url.auth"].as_str().unwrap());
        // println!("url token: {}", config[0]["url.token"].as_str().unwrap());
        // println!("url userinfo: {}", config[0]["url.userinfo"].as_str().unwrap());
        // println!("token min_size: {}", config[0]["token.min_size"].as_i64().unwrap());
        info!("Inputs read.\n");
        let mut token = pam_password.to_string();
        info!("Input min_size: {}.", config[0]["token.min_size"].as_i64().unwrap());
        info!("Actual pass_size: {}.", pam_password.len() as i64);
        if pam_password.len() as i64 <= config[0]["token.min_size"].as_i64().unwrap() {
            info!("Check as password.");
            let client =
                BasicClient::new(
                    ClientId::new(config[0]["client.id"].as_str().unwrap().to_string()),
                    Some(ClientSecret::new(config[0]["client.secret"].as_str().unwrap().to_string())),
                    // AuthUrl::new(Url::parse(config[0]["url.auth"].as_str().unwrap()).unwrap()),
                    AuthUrl::new(String::from(config[0]["url.auth"].as_str().unwrap())).unwrap(),
                    // Some(TokenUrl::new(Url::parse(config[0]["url.token"].as_str().unwrap()).unwrap()))
                    Some(TokenUrl::new(String::from(config[0]["url.token"].as_str().unwrap())).unwrap())
                );
                // .add_scope(Scope::new("openid".to_string()));
            let token_result =
                client.exchange_password(
                    &ResourceOwnerUsername::new(pam_user.to_string().clone()),
                    &ResourceOwnerPassword::new(pam_password.to_string())
                ).request(http_client);
            token = match token_result {
                Ok(tok) => tok.access_token().secret().to_string(),
                Err(_) => {
                    info!("Wrong password provided...");
                    return PamError::AUTH_ERR
                },
            }
        } else {
            info!("Check as token.");
        }
        // println!("token: {}", token);
        info!("Token ready for verification.\n");
        let userinfo_url = format!("{}?access_token={}", config[0]["url.userinfo"].as_str().unwrap(), token);
        // println!("userinfo: {}", userinfo_url);
        // let mut data = Vec::new();
        // let mut handle = Easy::new();
        // handle.url(&userinfo_url).unwrap();
        // {
        //     let mut transfer = handle.transfer();
        //     transfer.write_function(|new_data| {
        //         data.extend_from_slice(new_data);
        //         Ok(new_data.len())
        //     }).unwrap();
        //     transfer.perform().unwrap();
        // }
        // let body = String::from_utf8(data).expect("body is not valid UTF8!");
        let body = reqwest::blocking::get(&userinfo_url).unwrap().text().unwrap();
        // println!("body: {}", body);
        let json: Value = serde_json::from_str(&body).unwrap();
        // assert_ne!(json.get("sub"), None, "Token invalid error...");
        // println!("test: {}", json.get("sub").unwrap());
        // println!("sub: {}", json["sub"].as_str().unwrap());
        if json.get(config[0]["username.key"].as_str().unwrap().to_string()) != None && pam_user == json[config[0]["username.key"].as_str().unwrap().to_string()].as_str().unwrap() {
            info!("auth success!");
            PamError::SUCCESS
        } else if json.get(config[0]["username.key"].as_str().unwrap().to_string()) == None {
            info!("Token invalid error...");
            PamError::AUTH_ERR
        } else {
            info!("auth failed!");
            PamError::AUTH_ERR
        }
    }

    fn chauthtok(_pamh: Pam, _flags: PamFlag, _args: Vec<String>) -> PamError {
        PamError::SUCCESS
    }

    fn open_session(_pamh: Pam, _flags: PamFlag, _args: Vec<String>) -> PamError {
        PamError::SUCCESS
    }

    fn close_session(_pamh: Pam, _flags: PamFlag, _args: Vec<String>) -> PamError {
        PamError::SUCCESS
    }
    
    fn setcred(_pamh: Pam, _flags: PamFlag, _args: Vec<String>) -> PamError {
        PamError::CRED_UNAVAIL
    }
    
    fn acct_mgmt(_pamh: Pam, _flags: PamFlag, _args: Vec<String>) -> PamError {
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