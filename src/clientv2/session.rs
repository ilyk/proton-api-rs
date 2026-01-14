use crate::clientv2::TotpSession;
use crate::domain::{
    AddressId, Event, EventId, HumanVerification, HumanVerificationLoginData, Keys, Label,
    LabelType, PublicKeys, RecipientType, SecretString, TwoFactorAuth, User, UserUid,
};
use crate::http;
use crate::http::{OwnedRequest, RequestDesc, Sequence, SequenceFromState, X_PM_UID_HEADER};
use crate::requests::{
    AddressKeys, AuthInfoRequest, AuthInfoResponse, AuthRefreshRequest, AuthRequest, AuthResponse,
    GetAddressKeysRequest, GetAllKeysRequest, GetEventRequest, GetLabelsRequest,
    GetLatestEventRequest, GetPublicKeysRequest, GetUserKeysRequest, LogoutRequest, TFAStatus,
    TOTPRequest, UserAuth, UserInfoRequest, UserKeys,
};
use proton_srp::{SRPAuth, SRPProofB64};
use secrecy::{ExposeSecret, Secret};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("{0}")]
    Request(
        #[from]
        #[source]
        http::Error,
    ),
    #[error("Server SRP proof verification failed: {0}")]
    ServerProof(String),
    #[error("Account 2FA method ({0})is not supported")]
    Unsupported2FA(TwoFactorAuth),
    #[error("Human Verification Required'")]
    HumanVerificationRequired(HumanVerification),
    #[error("Failed to calculate SRP Proof: {0}")]
    SRPProof(String),
}

/// Data which can be used to save a session and restore it later.
pub struct SessionRefreshData {
    pub user_uid: Secret<UserUid>,
    pub token: Secret<String>,
}

impl PartialEq for SessionRefreshData {
    fn eq(&self, other: &Self) -> bool {
        self.user_uid.expose_secret() == other.user_uid.expose_secret()
            && self.token.expose_secret() == other.token.expose_secret()
    }
}

impl Eq for SessionRefreshData {}

#[derive(Debug)]
pub enum SessionType {
    Authenticated(Session),
    AwaitingTotp(TotpSession),
}

/// Authenticated Session from which one can access data/functionality restricted to authenticated
/// users.
#[derive(Debug, Clone)]
pub struct Session {
    pub(super) user_auth: Arc<parking_lot::RwLock<UserAuth>>,
}

impl Session {
    fn new(user: UserAuth) -> Self {
        Self {
            user_auth: Arc::new(parking_lot::RwLock::new(user)),
        }
    }

    pub fn login<'a>(
        username: &'a str,
        password: &'a SecretString,
        human_verification: Option<HumanVerificationLoginData>,
    ) -> impl Sequence<Output = SessionType, Error = LoginError> + 'a {
        let state = State {
            username,
            password,
            hv: human_verification,
        };

        SequenceFromState::new(state, login_sequence_1)
    }

    pub fn submit_totp<'a>(
        &'a self,
        code: &'a str,
    ) -> impl Sequence<Output = (), Error = http::Error> + 'a {
        //self.wrap_request(TOTPRequest::new(code).to_request())
        self.wrap_request2(TOTPRequest::new(code))
    }

    pub fn refresh<'a>(
        user_uid: &'a UserUid,
        token: &'a str,
    ) -> impl Sequence<Output = Self, Error = http::Error> + 'a {
        AuthRefreshRequest::new(user_uid, token)
            .to_request()
            .map(|r| {
                let user = UserAuth::from_auth_refresh_response(r);
                Ok(Session::new(user))
            })
    }

    pub fn get_user(&self) -> impl Sequence<Output = User> + '_ {
        //self.wrap_request(UserInfoRequest {}.to_request())
        //    .map(|r| -> Result<User, http::Error> { Ok(r.user) })
        self.wrap_request2(UserInfoRequest {})
            .map(|r| -> Result<User, http::Error> { Ok(r.user) })
    }

    pub fn logout(&self) -> impl Sequence<Output = (), Error = http::Error> + '_ {
        //self.wrap_request(LogoutRequest {}.to_request())
        self.wrap_request2(LogoutRequest {})
    }

    pub fn get_latest_event(&self) -> impl Sequence<Output = EventId, Error = http::Error> + '_ {
        //self.wrap_request(GetLatestEventRequest {}.to_request())
        //    .map(|r| Ok(r.event_id))
        self.wrap_request2(GetLatestEventRequest {})
            .map(|r| Ok(r.event_id))
    }

    pub fn get_event<'a, 'b: 'a>(
        &'b self,
        id: &'a EventId,
    ) -> impl Sequence<Output = Event, Error = http::Error> + 'a {
        //self.wrap_request(GetEventRequest::new(id).to_request())
        self.wrap_request2(GetEventRequest::new(id))
    }

    pub fn get_refresh_data(&self) -> SessionRefreshData {
        let reader = self.user_auth.read();
        SessionRefreshData {
            user_uid: reader.uid.clone(),
            token: reader.refresh_token.clone(),
        }
    }

    pub fn get_labels(
        &self,
        label_type: LabelType,
    ) -> impl Sequence<Output = Vec<Label>, Error = http::Error> + '_ {
        //self.wrap_request(GetLabelsRequest::new(label_type).to_request())
        //    .map(|r| Ok(r.labels))
        self.wrap_request2(GetLabelsRequest::new(label_type))
            .map(|r| Ok(r.labels))
    }

    // ========================================================================
    // Message API methods
    // ========================================================================

    /// Get messages with optional filtering.
    pub fn get_messages(
        &self,
        filter: crate::domain::MessageFilter,
    ) -> impl Sequence<Output = (Vec<crate::domain::MessageMetadata>, u32), Error = http::Error> + '_ {
        self.wrap_request2(crate::requests::GetMessagesRequest::new(filter))
            .map(|r| Ok((r.messages, r.total)))
    }

    /// Get messages in a specific label/folder.
    pub fn get_messages_in_label(
        &self,
        label_id: &str,
    ) -> impl Sequence<Output = (Vec<crate::domain::MessageMetadata>, u32), Error = http::Error> + '_ {
        self.wrap_request2(crate::requests::GetMessagesRequest::for_label(label_id))
            .map(|r| Ok((r.messages, r.total)))
    }

    /// Get a single message with full body content.
    pub fn get_message<'a, 'b: 'a>(
        &'b self,
        id: &'a crate::domain::MessageId,
    ) -> impl Sequence<Output = crate::domain::FullMessage, Error = http::Error> + 'a {
        self.wrap_request2(crate::requests::GetMessageRequest::new(id))
            .map(|r| Ok(r.message))
    }

    /// Mark messages as read.
    pub fn mark_messages_read<'a, 'b: 'a>(
        &'b self,
        ids: &'a [crate::domain::MessageId],
    ) -> impl Sequence<Output = (), Error = http::Error> + 'a {
        self.wrap_request2(crate::requests::MarkMessagesReadRequest::new(ids))
    }

    /// Mark messages as unread.
    pub fn mark_messages_unread<'a, 'b: 'a>(
        &'b self,
        ids: &'a [crate::domain::MessageId],
    ) -> impl Sequence<Output = (), Error = http::Error> + 'a {
        self.wrap_request2(crate::requests::MarkMessagesUnreadRequest::new(ids))
    }

    /// Add a label to messages.
    pub fn label_messages<'a, 'b: 'a>(
        &'b self,
        label_id: &'a str,
        ids: &'a [crate::domain::MessageId],
    ) -> impl Sequence<Output = crate::requests::LabelMessagesResponse, Error = http::Error> + 'a {
        self.wrap_request2(crate::requests::LabelMessagesRequest::new(label_id, ids))
    }

    /// Remove a label from messages.
    pub fn unlabel_messages<'a, 'b: 'a>(
        &'b self,
        label_id: &'a str,
        ids: &'a [crate::domain::MessageId],
    ) -> impl Sequence<Output = crate::requests::LabelMessagesResponse, Error = http::Error> + 'a {
        self.wrap_request2(crate::requests::UnlabelMessagesRequest::new(label_id, ids))
    }

    /// Permanently delete messages.
    pub fn delete_messages<'a, 'b: 'a>(
        &'b self,
        ids: &'a [crate::domain::MessageId],
    ) -> impl Sequence<Output = (), Error = http::Error> + 'a {
        self.wrap_request2(crate::requests::DeleteMessagesRequest::new(ids))
    }

    /// Get message counts per label.
    pub fn get_message_counts(
        &self,
    ) -> impl Sequence<Output = Vec<crate::domain::MessageCount>, Error = http::Error> + '_ {
        self.wrap_request2(crate::requests::GetMessageCountsRequest)
            .map(|r| Ok(r.counts))
    }

    /// Download an attachment by ID.
    /// Returns the encrypted attachment data as raw bytes.
    pub fn get_attachment<'a, 'b: 'a>(
        &'b self,
        attachment_id: &'a str,
    ) -> impl Sequence<Output = Vec<u8>, Error = http::Error> + 'a {
        self.wrap_request2(crate::requests::GetAttachmentRequest::new(attachment_id))
    }

    // ========================================================================
    // Key API methods
    // ========================================================================

    /// Get the user's private keys.
    ///
    /// These keys are used to decrypt messages and verify signatures.
    pub fn get_user_keys(&self) -> impl Sequence<Output = Keys, Error = http::Error> + '_ {
        self.wrap_request2(GetUserKeysRequest)
            .map(|r| Ok(r.keys))
    }

    /// Get keys for a specific address.
    ///
    /// Address keys are used to encrypt/decrypt messages for specific email addresses.
    pub fn get_address_keys<'a, 'b: 'a>(
        &'b self,
        address_id: &'a AddressId,
    ) -> impl Sequence<Output = Keys, Error = http::Error> + 'a {
        self.wrap_request2(GetAddressKeysRequest::new(address_id))
            .map(|r| Ok(r.keys))
    }

    /// Get public keys for an email address.
    ///
    /// Returns the recipient's public keys and whether they are an internal or external recipient.
    pub fn get_public_keys<'a, 'b: 'a>(
        &'b self,
        email: &'a str,
    ) -> impl Sequence<Output = (PublicKeys, RecipientType), Error = http::Error> + 'a {
        self.wrap_request2(GetPublicKeysRequest::new(email))
            .map(|r| Ok((r.keys, r.recipient_type)))
    }

    /// Get all keys for all addresses and the user.
    ///
    /// This is a comprehensive endpoint that returns both user keys and all address keys
    /// in a single request.
    pub fn get_all_keys(
        &self,
    ) -> impl Sequence<
        Output = (
            UserKeys,
            std::collections::HashMap<String, AddressKeys>,
        ),
        Error = http::Error,
    > + '_ {
        self.wrap_request2(GetAllKeysRequest)
            .map(|r| Ok((r.user_keys, r.address_keys)))
    }

    // ========================================================================
    // Message sending API methods
    // ========================================================================

    /// Create a new draft message.
    ///
    /// This creates a draft that can be edited before sending. The draft is stored
    /// on the server and assigned a message ID.
    pub fn create_draft<'a, 'b: 'a>(
        &'b self,
        draft: &'a crate::domain::DraftMessage,
    ) -> impl Sequence<Output = crate::domain::FullMessage, Error = http::Error> + 'a {
        self.wrap_request2(crate::requests::CreateDraftRequest::new(draft))
            .map(|r| Ok(r.message))
    }

    /// Update an existing draft message.
    ///
    /// This replaces the content of an existing draft. The draft must have been
    /// created previously using `create_draft`.
    pub fn update_draft<'a, 'b: 'a>(
        &'b self,
        id: &'a crate::domain::MessageId,
        draft: &'a crate::domain::DraftMessage,
    ) -> impl Sequence<Output = crate::domain::FullMessage, Error = http::Error> + 'a {
        self.wrap_request2(crate::requests::UpdateDraftRequest::new(id, draft))
            .map(|r| Ok(r.message))
    }

    /// Send a draft message.
    ///
    /// This sends a previously created draft to its recipients. The `packages`
    /// parameter contains the encrypted message body and session keys for each
    /// recipient type (internal/external).
    ///
    /// Note: Message encryption must be performed before calling this method.
    /// Each package should contain the encrypted body and key packets for the
    /// recipient addresses.
    pub fn send_message<'a, 'b: 'a>(
        &'b self,
        id: &'a crate::domain::MessageId,
        packages: &'a [crate::domain::MessagePackage],
    ) -> impl Sequence<Output = crate::domain::MessageId, Error = http::Error> + 'a {
        self.wrap_request2(crate::requests::SendMessageRequest::new(id, packages))
            .map(|r| Ok(r.sent.id))
    }

    #[inline(always)]
    fn wrap_request2<'a, 'b: 'a, R: RequestDesc + 'a>(
        &'b self,
        r: R,
    ) -> impl Sequence<Output = R::Output, Error = http::Error> + 'a {
        SequenceFromState::new(self, move |s| wrap_session_request(s, r))
    }
}

fn validate_server_proof(
    proof: &SRPProofB64,
    auth_response: AuthResponse,
) -> Result<SessionType, LoginError> {
    if !proof.compare_server_proof(&auth_response.server_proof) {
        return Err(LoginError::ServerProof(
            "Server Proof does not match".to_string(),
        ));
    }

    let tfa_enabled = auth_response.tfa.enabled;
    let user = UserAuth::from_auth_response(auth_response);

    let session = Session::new(user);

    match tfa_enabled {
        TFAStatus::None => Ok(SessionType::Authenticated(session)),
        TFAStatus::Totp => Ok(SessionType::AwaitingTotp(TotpSession(session))),
        TFAStatus::FIDO2 => Err(LoginError::Unsupported2FA(TwoFactorAuth::FIDO2)),
        TFAStatus::TotpOrFIDO2 => Ok(SessionType::AwaitingTotp(TotpSession(session))),
    }
}

fn map_human_verification_err(e: LoginError) -> LoginError {
    if let LoginError::Request(http::Error::API(e)) = &e {
        if let Ok(hv) = e.try_get_human_verification_details() {
            return LoginError::HumanVerificationRequired(hv);
        }
    }

    e
}

struct State<'a> {
    username: &'a str,
    password: &'a SecretString,
    hv: Option<HumanVerificationLoginData>,
}

struct LoginState<'a> {
    username: &'a str,
    proof: SRPProofB64,
    session: String,
    hv: Option<HumanVerificationLoginData>,
}

fn generate_login_state(
    state: State,
    auth_info_response: AuthInfoResponse,
) -> Result<LoginState, LoginError> {
    // Create SRP auth and generate proofs using pure Rust proton-srp
    let srp_auth = SRPAuth::with_pgp(
        state.password.expose_secret(),
        auth_info_response.version as u8,
        &auth_info_response.salt,
        &auth_info_response.modulus,
        &auth_info_response.server_ephemeral,
    )
    .map_err(|e| LoginError::SRPProof(e.to_string()))?;

    let proof: SRPProofB64 = srp_auth
        .generate_proofs()
        .map_err(|e| LoginError::SRPProof(e.to_string()))?
        .into();

    Ok(LoginState {
        username: state.username,
        proof,
        session: auth_info_response.srp_session,
        hv: state.hv,
    })
}

fn login_sequence_2(
    login_state: LoginState,
) -> impl Sequence<Output = SessionType, Error = LoginError> + '_ {
    AuthRequest {
        username: login_state.username,
        client_ephemeral: &login_state.proof.client_ephemeral,
        client_proof: &login_state.proof.client_proof,
        srp_session: &login_state.session,
        human_verification: &login_state.hv,
    }
    .to_request()
    .map(move |auth_response| {
        validate_server_proof(&login_state.proof, auth_response).map_err(map_human_verification_err)
    })
}

fn login_sequence_1(st: State) -> impl Sequence<Output = SessionType, Error = LoginError> + '_ {
    AuthInfoRequest {
        username: st.username,
    }
    .to_request()
    .map(move |auth_info_response| generate_login_state(st, auth_info_response))
    .state(login_sequence_2)
}

fn wrap_session_request<'a, R: RequestDesc + 'a>(
    session: &'a Session,
    r: R,
) -> impl Sequence<Output = R::Output, Error = http::Error> + 'a {
    let data = {
        let borrow = session.user_auth.read();
        r.build()
            .header(X_PM_UID_HEADER, borrow.uid.expose_secret().as_str())
            .bearer_token(borrow.access_token.expose_secret())
    };

    // While we clone headers and url, the body clone is handled efficiently.
    OwnedRequest::<R::Response>::new(data.clone()).chain_err(move |e| {
        if let http::Error::API(api_err) = &e {
            if api_err.http_code == 401 {
                log::debug!("Account session expired, attempting refresh");
                return Ok({
                    let borrow = session.user_auth.read();
                    AuthRefreshRequest::new(
                        borrow.uid.expose_secret(),
                        borrow.refresh_token.expose_secret(),
                    )
                    .to_request()
                }
                .chain(move |resp| {
                    let data = {
                        let mut writer = session.user_auth.write();
                        *writer = UserAuth::from_auth_refresh_response(resp);
                        data.header(X_PM_UID_HEADER, writer.uid.expose_secret().as_str())
                            .bearer_token(writer.access_token.expose_secret())
                    };
                    Ok(OwnedRequest::<R::Response>::new(data))
                }));
            }
        }

        Err(e)
    })
}
