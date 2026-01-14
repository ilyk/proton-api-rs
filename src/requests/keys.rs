//! Key management API endpoints.
//!
//! This module provides request types for fetching and managing cryptographic keys
//! in the Proton Mail API.

use crate::domain::{AddressId, Keys, PublicKeys, RecipientType};
use crate::http;
use crate::http::RequestData;
use serde::Deserialize;

/// Request to get a user's private keys.
///
/// Endpoint: GET /core/v4/keys
pub struct GetUserKeysRequest;

#[doc(hidden)]
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetUserKeysResponse {
    pub keys: Keys,
}

impl http::RequestDesc for GetUserKeysRequest {
    type Output = GetUserKeysResponse;
    type Response = http::JsonResponse<Self::Output>;

    fn build(&self) -> RequestData {
        RequestData::new(http::Method::Get, "core/v4/keys".to_string())
    }
}

/// Request to get keys for a specific address.
///
/// Endpoint: GET /core/v4/keys/address/{addressId}
pub struct GetAddressKeysRequest<'a> {
    address_id: &'a AddressId,
}

#[doc(hidden)]
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetAddressKeysResponse {
    pub keys: Keys,
}

impl<'a> GetAddressKeysRequest<'a> {
    pub fn new(address_id: &'a AddressId) -> Self {
        Self { address_id }
    }
}

impl<'a> http::RequestDesc for GetAddressKeysRequest<'a> {
    type Output = GetAddressKeysResponse;
    type Response = http::JsonResponse<Self::Output>;

    fn build(&self) -> RequestData {
        RequestData::new(
            http::Method::Get,
            format!("core/v4/keys/address/{}", self.address_id),
        )
    }
}

/// Request to get public keys for an email address.
///
/// Endpoint: GET /core/v4/keys?Email={email}
pub struct GetPublicKeysRequest<'a> {
    email: &'a str,
}

#[doc(hidden)]
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetPublicKeysResponse {
    /// The recipient's public keys.
    pub keys: PublicKeys,
    /// The recipient type (internal or external).
    pub recipient_type: RecipientType,
}

impl<'a> GetPublicKeysRequest<'a> {
    pub fn new(email: &'a str) -> Self {
        Self { email }
    }
}

impl<'a> http::RequestDesc for GetPublicKeysRequest<'a> {
    type Output = GetPublicKeysResponse;
    type Response = http::JsonResponse<Self::Output>;

    fn build(&self) -> RequestData {
        RequestData::new(
            http::Method::Get,
            format!("core/v4/keys?Email={}", self.email),
        )
    }
}

/// Request to get all keys for all addresses of the user.
///
/// Endpoint: GET /core/v4/keys/all
pub struct GetAllKeysRequest;

#[doc(hidden)]
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetAllKeysResponse {
    /// Map of address ID to keys.
    #[serde(rename = "Address")]
    pub address_keys: std::collections::HashMap<String, AddressKeys>,
    /// User keys.
    #[serde(rename = "User")]
    pub user_keys: UserKeys,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AddressKeys {
    pub keys: Keys,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct UserKeys {
    pub keys: Keys,
}

impl http::RequestDesc for GetAllKeysRequest {
    type Output = GetAllKeysResponse;
    type Response = http::JsonResponse<Self::Output>;

    fn build(&self) -> RequestData {
        RequestData::new(http::Method::Get, "core/v4/keys/all".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_user_keys_request_build() {
        let req = GetUserKeysRequest;
        let data = req.build();
        assert_eq!(data.url, "core/v4/keys");
    }

    #[test]
    fn test_get_address_keys_request_build() {
        let address_id = AddressId("test-address-id".to_string());
        let req = GetAddressKeysRequest::new(&address_id);
        let data = req.build();
        assert_eq!(data.url, "core/v4/keys/address/test-address-id");
    }

    #[test]
    fn test_get_public_keys_request_build() {
        let req = GetPublicKeysRequest::new("test@example.com");
        let data = req.build();
        assert_eq!(data.url, "core/v4/keys?Email=test@example.com");
    }

    #[test]
    fn test_get_all_keys_request_build() {
        let req = GetAllKeysRequest;
        let data = req.build();
        assert_eq!(data.url, "core/v4/keys/all");
    }
}
