//! Base58 text form of account ids, the only address format the app exposes.

use arrhabon_domain::AccountId;

use crate::AppError;

pub fn to_base58(id: AccountId) -> String {
    bs58::encode(id.0).into_string()
}

/// Rejects anything that is not exactly 32 bytes of base58, and the zero account.
pub fn parse_account(text: &str) -> Result<AccountId, AppError> {
    let mut bytes = [0u8; 32];
    match bs58::decode(text).onto(&mut bytes) {
        Ok(32) if bytes != [0; 32] => Ok(AccountId(bytes)),
        _ => Err(AppError::InvalidInput(
            "account must be a non-zero 32-byte base58 address",
        )),
    }
}
