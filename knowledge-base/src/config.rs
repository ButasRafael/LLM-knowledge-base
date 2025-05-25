use crate::{Error, Result};
use std::env;
use std::str::FromStr;
use std::sync::OnceLock;

pub fn config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        Config::load_from_env().unwrap_or_else(|ex| {
            panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}")
        })
    })
}

#[allow(non_snake_case)]
pub struct Config {
    pub PWD_KEY: Vec<u8>,
    pub TOKEN_KEY: Vec<u8>,
    pub TOKEN_DURATION_SEC: f64,

    // -- Db
    pub DB_URL: String,

    pub UPLOAD_DIR: String,

    pub REDIS_URL: String,
    pub QDRANT_URL: String,
    pub JAEGER_ENDPOINT: String,
    pub OLLAMA_URL: String,
    pub DATABASE_URL: String,
}

impl Config {
    fn load_from_env() -> Result<Config> {
        Ok(Config {
            PWD_KEY: get_env_b64u_as_u8s("SERVICE_PWD_KEY")?,
            TOKEN_KEY: get_env_b64u_as_u8s("SERVICE_TOKEN_KEY")?,
            TOKEN_DURATION_SEC: get_env_parse("SERVICE_TOKEN_DURATION_SEC")?,

            DB_URL: get_env("SERVICE_DB_URL")?,

            UPLOAD_DIR: get_env("SERVICE_UPLOAD_DIR")?,

            REDIS_URL: get_env("SERVICE_REDIS_URL")?,

            QDRANT_URL: get_env("SERVICE_QDRANT_URL")?,

            JAEGER_ENDPOINT: get_env("SERVICE_JAEGER_ENDPOINT")?,

            OLLAMA_URL: get_env("SERVICE_OLLAMA_URL")?,

            DATABASE_URL: get_env("SERVICE_DATABASE_URL")?,


        })
    }
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}

fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T> {
    let val = get_env(name)?;
    val.parse::<T>().map_err(|_| Error::ConfigWrongFormat(name))
}

fn get_env_b64u_as_u8s(name: &'static str) -> Result<Vec<u8>> {
    base64_url::decode(&get_env(name)?).map_err(|_| Error::ConfigWrongFormat(name))
}
