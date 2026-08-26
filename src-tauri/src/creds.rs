//! Passwords live in the OS keychain (macOS Keychain, Windows Credential
//! Manager, Linux secret-service); only {host, username} metadata is kept in
//! config.json for listing.

use crate::error::{AppError, AppResult};
use crate::svn::client::Credentials;

const SERVICE: &str = "SVN Manager";

fn entry(host: &str) -> AppResult<keyring::Entry> {
    keyring::Entry::new(SERVICE, host).map_err(|e| AppError::Config(e.to_string()))
}

pub fn save(host: &str, username: &str, password: &str) -> AppResult<()> {
    // secret encodes both so a username change can't desync from the password
    let secret = serde_json::json!({ "username": username, "password": password });
    entry(host)?
        .set_password(&secret.to_string())
        .map_err(|e| AppError::Config(format!("keychain save failed: {e}")))
}

pub fn get(host: &str) -> Option<Credentials> {
    let raw = entry(host).ok()?.get_password().ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    Some(Credentials {
        username: v.get("username")?.as_str()?.to_string(),
        password: v.get("password")?.as_str()?.to_string(),
    })
}

pub fn delete(host: &str) -> AppResult<()> {
    match entry(host)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Config(format!("keychain delete failed: {e}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Touches the real OS keychain — run locally with:
    //   cargo test keychain_roundtrip -- --ignored
    #[test]
    #[ignore]
    fn keychain_roundtrip() {
        let host = "svn-manager-test.invalid";
        save(host, "tester", "s3cret").unwrap();
        let c = get(host).expect("credential should round-trip");
        assert_eq!(c.username, "tester");
        assert_eq!(c.password, "s3cret");
        delete(host).unwrap();
        assert!(get(host).is_none());
        // deleting a missing entry is not an error
        delete(host).unwrap();
    }
}
