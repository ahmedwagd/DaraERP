use keyring::Entry;

use crate::error::AppError;

const SERVICE_NAME: &str = "daraerp";
const SECRET_KEY: &str = "jwt-secret";
const REFRESH_TOKEN_KEY: &str = "refresh-token";

/// Loads the JWT signing secret from the OS keychain.
///
/// On first run, generates a 256-bit random secret and persists it via keyring.
/// On subsequent runs, loads the existing secret from keyring.
pub fn load_jwt_secret() -> Result<String, AppError> {
    let entry = Entry::new(SERVICE_NAME, SECRET_KEY)
        .map_err(|e| AppError::new("IO_ERROR", format!("keyring entry failed: {e}")))?;

    match entry.get_password() {
        Ok(secret) => Ok(secret),
        Err(keyring::Error::NoEntry) => {
            let secret = generate_random_secret();
            entry
                .set_password(&secret)
                .map_err(|e| AppError::new("IO_ERROR", format!("keyring write failed: {e}")))?;
            Ok(secret)
        }
        Err(e) => Err(AppError::new(
            "IO_ERROR",
            format!("keyring read failed: {e}"),
        )),
    }
}

/// Reads the stored raw refresh token from the OS keychain.
pub fn get_refresh_token() -> Result<String, AppError> {
    let entry = Entry::new(SERVICE_NAME, REFRESH_TOKEN_KEY)
        .map_err(|e| AppError::new("IO_ERROR", format!("keyring entry failed: {e}")))?;

    match entry.get_password() {
        Ok(token) => Ok(token),
        Err(keyring::Error::NoEntry) => {
            Err(AppError::new("NOT_FOUND", "No refresh token in keychain"))
        }
        Err(e) => Err(AppError::new(
            "IO_ERROR",
            format!("keyring read failed: {e}"),
        )),
    }
}

/// Stores a raw refresh token in the OS keychain.
pub fn set_refresh_token(token: &str) -> Result<(), AppError> {
    let entry = Entry::new(SERVICE_NAME, REFRESH_TOKEN_KEY)
        .map_err(|e| AppError::new("IO_ERROR", format!("keyring entry failed: {e}")))?;

    entry
        .set_password(token)
        .map_err(|e| AppError::new("IO_ERROR", format!("keyring write failed: {e}")))?;
    Ok(())
}

/// Removes the stored refresh token from the OS keychain.
pub fn clear_refresh_token() -> Result<(), AppError> {
    let entry = Entry::new(SERVICE_NAME, REFRESH_TOKEN_KEY)
        .map_err(|e| AppError::new("IO_ERROR", format!("keyring entry failed: {e}")))?;

    entry
        .delete_credential()
        .or_else(|e| match e {
            keyring::Error::NoEntry => Ok(()),
            _ => Err(AppError::new(
                "IO_ERROR",
                format!("keyring delete failed: {e}"),
            )),
        })
}

/// Generates a 256-bit (32-byte) random hex string for use as a JWT secret.
fn generate_random_secret() -> String {
    use rand::RngCore;
    let mut bytes = vec![0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(&bytes)
}
