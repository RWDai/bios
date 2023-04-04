use std::collections::HashMap;

use tardis::{
    basic::{error::TardisError, result::TardisResult},
    crypto::crypto_sm2_4::{TardisCryptoSm2PrivateKey, TardisCryptoSm2PublicKey},
    tokio::sync::RwLock,
    TardisFuns,
};

use lazy_static::lazy_static;

use crate::{
    auth_config::AuthConfig,
    auth_constants::DOMAIN_CODE,
    dto::auth_crypto_dto::{AuthEncryptReq, AuthEncryptResp},
};

lazy_static! {
    static ref SM2_KEYS: RwLock<Option<(TardisCryptoSm2PublicKey, TardisCryptoSm2PrivateKey)>> = RwLock::new(None);
}

pub(crate) async fn init() -> TardisResult<()> {
    let pri_key = TardisFuns::crypto.sm2.new_private_key()?;
    let pub_key = TardisFuns::crypto.sm2.new_public_key(&pri_key)?;
    let mut sm_keys = SM2_KEYS.write().await;
    *sm_keys = Some((pub_key, pri_key));
    Ok(())
}

pub(crate) async fn fetch_public_key() -> TardisResult<String> {
    let sm_keys = SM2_KEYS.read().await;
    Ok(sm_keys.as_ref().unwrap().0.serialize()?)
}

pub(crate) async fn decrypt_req(
    headers: &HashMap<String, String>,
    body: &Option<String>,
    need_crypto_req: bool,
    need_crypto_resp: bool,
    config: &AuthConfig,
) -> TardisResult<(Option<String>, Option<HashMap<String, String>>)> {
    let input_keys = headers.get(&config.head_key_crypto).ok_or_else(|| {
        TardisError::bad_request(
            &format!("[Auth] Encrypted request: {} field is not in header.", config.head_key_crypto),
            "401-auth-req-crypto-error",
        )
    })?;
    let input_keys = TardisFuns::crypto.base64.decode(input_keys).map_err(|_| {
        TardisError::bad_request(
            &format!("[Auth] Encrypted request: {} field in header is not base64 format.", config.head_key_crypto),
            "401-auth-req-crypto-error",
        )
    })?;
    let sm_keys = SM2_KEYS.read().await;
    let input_keys = sm_keys
        .as_ref()
        .unwrap()
        .1
        .decrypt(&input_keys)
        .map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted request: decrypt error:{e}"), "401-auth-req-crypto-error"))?;
    let input_keys = input_keys.split(" ").collect::<Vec<&str>>();

    if need_crypto_req && need_crypto_resp {
        if input_keys.len() != 3 {
            return Err(TardisError::bad_request(
                &format!("[Auth] Encrypted request: {} field in header is illegal.", config.head_key_crypto),
                "401-auth-req-crypto-error",
            ));
        }
        let body = body.as_ref().ok_or_else(|| TardisError::bad_request("[Auth] Encrypted request: body is empty.", "401-auth-req-crypto-error"))?;

        let input_sm2_key = input_keys[0];
        let input_sm2_iv = input_keys[1];
        let input_pub_key = input_keys[2];

        let data = TardisFuns::crypto
            .sm4
            .decrypt_cbc(body, input_sm2_key, input_sm2_iv)
            .map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted request: key decrypt error:{e}"), "401-auth-req-crypto-error"))?;
        Ok((
            Some(data),
            Some(HashMap::from([(config.head_key_crypto.to_string(), TardisFuns::crypto.base64.encode(input_pub_key))])),
        ))
    } else if need_crypto_req {
        if input_keys.len() < 2 {
            return Err(TardisError::bad_request(
                &format!("[Auth] Encrypted request: {} field in header is illegal.", config.head_key_crypto),
                "401-auth-req-crypto-error",
            ));
        }
        let body = body.as_ref().ok_or_else(|| TardisError::bad_request("[Auth] Encrypted request: body is empty.", "401-auth-req-crypto-error"))?;

        let input_sm2_key = input_keys[0];
        let input_sm2_iv = input_keys[1];
        let data = TardisFuns::crypto
            .sm4
            .decrypt_cbc(body, input_sm2_key, input_sm2_iv)
            .map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted request: body decrypt error:{e}"), "401-auth-req-crypto-error"))?;
        Ok((Some(data), None))
    } else {
        if input_keys.len() < 1 {
            return Err(TardisError::bad_request(
                &format!("[Auth] Encrypted request: {} field in header is illegal.", config.head_key_crypto),
                "401-auth-req-crypto-error",
            ));
        }
        let input_pub_key = input_keys[0];
        Ok((
            None,
            Some(HashMap::from([(config.head_key_crypto.to_string(), TardisFuns::crypto.base64.encode(input_pub_key))])),
        ))
    }
}

pub(crate) async fn encrypt_body(req: &AuthEncryptReq) -> TardisResult<AuthEncryptResp> {
    let config = TardisFuns::cs_config::<AuthConfig>(DOMAIN_CODE);
    let pub_key = req.headers.get(&config.head_key_crypto).ok_or_else(|| {
        TardisError::bad_request(
            &format!("[Auth] Encrypted response: {} field is not in header.", config.head_key_crypto),
            "401-auth-req-crypto-error",
        )
    })?;
    let pub_key = TardisFuns::crypto.base64.decode(pub_key).map_err(|_| {
        TardisError::bad_request(
            &format!("[Auth] Encrypted response: {} field in header is not base64 format.", config.head_key_crypto),
            "401-auth-req-crypto-error",
        )
    })?;
    let pub_key = TardisFuns::crypto
        .sm2
        .new_public_key_from_public_key(&pub_key)
        .map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted response: generate public key error:{e}"), "401-auth-req-crypto-error"))?;

    let sm4_key = TardisFuns::crypto.key.rand_16_hex()?;
    let sm4_iv = TardisFuns::crypto.key.rand_16_hex()?;

    let data = TardisFuns::crypto
        .sm4
        .encrypt_cbc(&req.body, &sm4_key, &sm4_iv)
        .map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted response: body encrypt error:{e}"), "401-auth-req-crypto-error"))?;

    let pub_key = pub_key
        .encrypt(&format!("{sm4_key} {sm4_iv}"))
        .map_err(|e| TardisError::bad_request(&format!("[Auth] Encrypted response: key encrypt error:{e}"), "401-auth-req-crypto-error"))?;
    Ok(AuthEncryptResp {
        headers: HashMap::from([(config.head_key_crypto.to_string(), TardisFuns::crypto.base64.encode(&pub_key))]),
        body: data,
    })
}