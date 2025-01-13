use std::fmt::Display;
use std::str::FromStr;
use sqlx::Encode;
use crate::error::{Error, Result};
use crate::config;
use crate::crypt::{encrypt, EncryptContent};
use crate::utils::{decode, encode, now, now_plus, parse};

pub struct Token {
    pub identifier: String,
    pub expiration:String,
    pub signature:String,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", encode(&self.identifier),encode(&self.expiration), self.signature)
    }
}
impl FromStr for Token {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return Err(Error::TokenInvalidFormat);
        }

        let encoded_identifier = parts[0];
        let encoded_expiration = parts[1];
        let signature = parts[2];

        let identifier = decode(encoded_identifier).map_err(|_| Error::TokenCannotDecodeIdentifier)?;
        let expiration = decode(encoded_expiration).map_err(|_| Error::TokenCannotDecodeExpiration)?;

        Ok(Token {
            identifier: identifier.to_string(),
            expiration: expiration.to_string(),
            signature: signature.to_string(),
        })
    }
}

pub fn generate_token(identifier:&str, salt:&str) -> Result<Token> {
    let config = &config();
    _generate_token(identifier, config.TOKEN_DURATION_SEC, salt, &config.TOKEN_KEY)
}

pub fn validate_token(token:&Token, salt:&str) -> Result<()> {
    let config = &config();
    _validate_token(token, salt, &config.TOKEN_KEY)?;
    Ok(())
}

fn _generate_token(identifier:&str, duration_sec:f64,salt:&str, key:&[u8]) -> Result <Token> {
    let identifier = identifier.to_string();
    let expiration = now_plus(duration_sec);
    let signature = _token_signature_gen(&identifier, &expiration, salt, key)?;

    Ok(Token {
        identifier,
        expiration,
        signature,
    })
}

fn _validate_token(token:&Token, salt:&str, key:&[u8]) -> Result<()> {
    let new_signature = _token_signature_gen(&token.identifier,&token.expiration, salt, key)?;
    if new_signature != token.signature {
        return Err(Error::TokenSignatureNotMatching);
    }
    let expiration = parse(&token.expiration).map_err(|_| Error::TokenExpirationNotIso)?;
    let now = now();
    if now > expiration {
        return Err(Error::TokenExpired);
    }
    Ok(())
}

fn _token_signature_gen(identifier:&str, expiration:&str, salt:&str, key:&[u8]) -> Result<String> {
    let content = format!("{}:{}", encode(identifier), encode(expiration));
    let signature = encrypt(key, &EncryptContent {
        content,
        salt: salt.to_string(),
    })?;
    Ok(signature)
}