//! Key domain types for cryptographic operations.
//!
//! This module provides types for managing user and address keys used in
//! Proton Mail's end-to-end encryption system. Note that `KeyId` and related
//! basic key types are defined in the `user` module and re-exported here.

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::Boolean;

// Re-export key types from user module to avoid conflicts
pub use super::user::{Key, KeyState};

/// Collection of keys (user or address keys).
pub type Keys = Vec<Key>;

/// Public key information for a recipient.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PublicKey {
    /// Key state flags (optional, may be None for external keys).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<KeyState>,
    /// Armored PGP public key.
    pub public_key: String,
}

/// Collection of public keys.
pub type PublicKeys = Vec<PublicKey>;

/// Signed key list for key operations.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct KeyList {
    /// JSON-encoded key list data.
    pub data: String,
    /// Signature of the key list.
    pub signature: String,
}

/// Entry in a key list.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct KeyListEntry {
    /// PGP fingerprint.
    pub fingerprint: String,
    /// SHA256 fingerprints.
    #[serde(rename = "SHA256Fingerprints")]
    pub sha256_fingerprints: Vec<String>,
    /// Key state flags.
    pub flags: KeyState,
    /// Whether this is the primary key.
    pub primary: Boolean,
}

/// Recipient type for public key lookups.
#[derive(Debug, Deserialize_repr, Serialize_repr, Eq, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum RecipientType {
    /// Internal Proton recipient.
    Internal = 1,
    /// External recipient.
    External = 2,
}

/// Request payload for creating an address key.
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CreateAddressKeyReq {
    /// Address ID to associate the key with.
    #[serde(rename = "AddressID")]
    pub address_id: String,
    /// Armored PGP private key.
    pub private_key: String,
    /// Whether this should be the primary key.
    pub primary: Boolean,
    /// Signed key list.
    pub signed_key_list: KeyList,
    /// Token for migrated accounts (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// Signature for migrated accounts (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

/// Request payload for making an address key primary.
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MakeAddressKeyPrimaryReq {
    /// Signed key list with the new primary key.
    pub signed_key_list: KeyList,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_state_values() {
        assert_eq!(KeyState::None as u8, 0);
        assert_eq!(KeyState::Trusted as u8, 1);
        assert_eq!(KeyState::Active as u8, 2);
    }

    #[test]
    fn test_recipient_type_values() {
        assert_eq!(RecipientType::Internal as u8, 1);
        assert_eq!(RecipientType::External as u8, 2);
    }
}
