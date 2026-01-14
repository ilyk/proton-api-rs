//! Message domain types for Proton Mail API.
//!
//! This module contains comprehensive message types for the Proton Mail API.
//! Note: `MessageId` is defined in `event.rs` and re-exported from the domain module.

use crate::domain::{Boolean, LabelId, MessageId};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

/// Conversation API ID.
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Hash, Clone)]
pub struct ConversationId(pub String);

impl Display for ConversationId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Address ID (for sender/recipient address keys).
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Hash, Clone)]
pub struct AddressId(pub String);

impl Display for AddressId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Email address with optional display name.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct MessageAddress {
    /// Display name (e.g., "John Doe")
    pub name: String,
    /// Email address (e.g., "john@example.com")
    pub address: String,
}

impl MessageAddress {
    pub fn new(name: impl Into<String>, address: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            address: address.into(),
        }
    }
}

impl Display for MessageAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.name.is_empty() {
            write!(f, "{}", self.address)
        } else {
            write!(f, "{} <{}>", self.name, self.address)
        }
    }
}

/// Message flags (bitfield).
#[derive(Debug, Deserialize_repr, Serialize_repr, Eq, PartialEq, Copy, Clone, Default)]
#[repr(u64)]
pub enum MessageFlag {
    #[default]
    None = 0,
    /// Message has been received (as opposed to sent).
    Received = 1,
    /// Message has been sent.
    Sent = 2,
    /// Message is internal (sent within Proton).
    Internal = 4,
    /// Message is end-to-end encrypted.
    E2E = 8,
    /// Message is auto-reply.
    Auto = 16,
    /// Message has been replied to.
    Replied = 32,
    /// Message has been replied to all.
    RepliedAll = 64,
    /// Message has been forwarded.
    Forwarded = 128,
    /// Message is a receipt request.
    ReceiptRequest = 256,
    /// Message is a receipt.
    ReceiptSent = 512,
    /// Message is DKIM signed.
    DKIMSigned = 1024,
    /// Message failed DKIM verification.
    DKIMFailed = 2048,
    /// Message is spam.
    Spam = 4096,
    /// Message is phishing.
    Phishing = 8192,
    /// Message is virus.
    Virus = 16384,
    /// Message has been scheduled.
    Scheduled = 32768,
}

/// MIME type for message content.
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone, Default)]
pub struct MimeType(pub String);

impl MimeType {
    pub const TEXT_PLAIN: &'static str = "text/plain";
    pub const TEXT_HTML: &'static str = "text/html";
    pub const MULTIPART_MIXED: &'static str = "multipart/mixed";

    pub fn text_plain() -> Self {
        Self(Self::TEXT_PLAIN.to_string())
    }

    pub fn text_html() -> Self {
        Self(Self::TEXT_HTML.to_string())
    }

    pub fn is_html(&self) -> bool {
        self.0 == Self::TEXT_HTML
    }

    pub fn is_plain(&self) -> bool {
        self.0 == Self::TEXT_PLAIN
    }
}

impl Display for MimeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Message metadata (lightweight, for listing).
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MessageMetadata {
    /// Unique message ID.
    #[serde(rename = "ID")]
    pub id: MessageId,

    /// Conversation this message belongs to.
    #[serde(rename = "ConversationID")]
    pub conversation_id: ConversationId,

    /// Address ID of the sender address (for key lookup).
    #[serde(rename = "AddressID")]
    pub address_id: AddressId,

    /// Subject line.
    pub subject: String,

    /// Sender address.
    pub sender: MessageAddress,

    /// Primary recipients (To:).
    #[serde(rename = "ToList", default)]
    pub to_list: Vec<MessageAddress>,

    /// Carbon copy recipients (CC:).
    #[serde(rename = "CCList", default)]
    pub cc_list: Vec<MessageAddress>,

    /// Blind carbon copy recipients (BCC:).
    #[serde(rename = "BCCList", default)]
    pub bcc_list: Vec<MessageAddress>,

    /// Reply-to addresses.
    #[serde(rename = "ReplyTos", default)]
    pub reply_tos: Vec<MessageAddress>,

    /// Message flags.
    #[serde(default)]
    pub flags: MessageFlag,

    /// Unix timestamp when the message was sent/received.
    pub time: i64,

    /// Size in bytes.
    pub size: u64,

    /// Whether the message is unread.
    #[serde(default)]
    pub unread: Boolean,

    /// Whether the message has been replied to.
    #[serde(rename = "IsReplied", default)]
    pub is_replied: Boolean,

    /// Whether the message has been replied to all.
    #[serde(rename = "IsRepliedAll", default)]
    pub is_replied_all: Boolean,

    /// Whether the message has been forwarded.
    #[serde(rename = "IsForwarded", default)]
    pub is_forwarded: Boolean,

    /// Number of attachments.
    #[serde(default)]
    pub num_attachments: u32,

    /// Labels/folders this message is in.
    #[serde(rename = "LabelIDs", default)]
    pub label_ids: Vec<LabelId>,

    /// External message ID (from headers).
    #[serde(rename = "ExternalID", default)]
    pub external_id: Option<String>,
}

/// Attachment metadata.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Attachment {
    /// Unique attachment ID.
    #[serde(rename = "ID")]
    pub id: String,

    /// Filename.
    pub name: String,

    /// Size in bytes.
    pub size: u64,

    /// MIME type.
    #[serde(rename = "MIMEType")]
    pub mime_type: String,

    /// Content-ID for inline attachments.
    #[serde(rename = "ContentID", default)]
    pub content_id: Option<String>,

    /// Attachment headers (encrypted).
    #[serde(default)]
    pub headers: Option<serde_json::Value>,

    /// Key packets for decryption.
    #[serde(rename = "KeyPackets", default)]
    pub key_packets: Option<String>,
}

/// Full message with body content (from GET /mail/v4/messages/{id}).
/// Note: Named `FullMessage` to distinguish from `Message` in `event.rs` which is
/// a lightweight version used in event streams.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct FullMessage {
    /// Unique message ID.
    #[serde(rename = "ID")]
    pub id: MessageId,

    /// Conversation this message belongs to.
    #[serde(rename = "ConversationID")]
    pub conversation_id: ConversationId,

    /// Address ID of the sender address (for key lookup).
    #[serde(rename = "AddressID")]
    pub address_id: AddressId,

    /// Subject line.
    pub subject: String,

    /// Sender address.
    pub sender: MessageAddress,

    /// Primary recipients (To:).
    #[serde(rename = "ToList", default)]
    pub to_list: Vec<MessageAddress>,

    /// Carbon copy recipients (CC:).
    #[serde(rename = "CCList", default)]
    pub cc_list: Vec<MessageAddress>,

    /// Blind carbon copy recipients (BCC:).
    #[serde(rename = "BCCList", default)]
    pub bcc_list: Vec<MessageAddress>,

    /// Reply-to addresses.
    #[serde(rename = "ReplyTos", default)]
    pub reply_tos: Vec<MessageAddress>,

    /// Message flags.
    #[serde(default)]
    pub flags: MessageFlag,

    /// Unix timestamp when the message was sent/received.
    pub time: i64,

    /// Size in bytes.
    pub size: u64,

    /// Whether the message is unread.
    #[serde(default)]
    pub unread: Boolean,

    /// Whether the message has been replied to.
    #[serde(rename = "IsReplied", default)]
    pub is_replied: Boolean,

    /// Whether the message has been replied to all.
    #[serde(rename = "IsRepliedAll", default)]
    pub is_replied_all: Boolean,

    /// Whether the message has been forwarded.
    #[serde(rename = "IsForwarded", default)]
    pub is_forwarded: Boolean,

    /// Number of attachments.
    #[serde(default)]
    pub num_attachments: u32,

    /// Labels/folders this message is in.
    #[serde(rename = "LabelIDs", default)]
    pub label_ids: Vec<LabelId>,

    /// External message ID (from headers).
    #[serde(rename = "ExternalID", default)]
    pub external_id: Option<String>,

    /// Raw email headers.
    #[serde(default)]
    pub header: String,

    /// Encrypted message body.
    pub body: String,

    /// MIME type of the body.
    #[serde(rename = "MIMEType", default)]
    pub mime_type: MimeType,

    /// Message attachments.
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

impl FullMessage {
    /// Get metadata view of this message.
    pub fn metadata(&self) -> MessageMetadata {
        MessageMetadata {
            id: self.id.clone(),
            conversation_id: self.conversation_id.clone(),
            address_id: self.address_id.clone(),
            subject: self.subject.clone(),
            sender: self.sender.clone(),
            to_list: self.to_list.clone(),
            cc_list: self.cc_list.clone(),
            bcc_list: self.bcc_list.clone(),
            reply_tos: self.reply_tos.clone(),
            flags: self.flags,
            time: self.time,
            size: self.size,
            unread: self.unread,
            is_replied: self.is_replied,
            is_replied_all: self.is_replied_all,
            is_forwarded: self.is_forwarded,
            num_attachments: self.num_attachments,
            label_ids: self.label_ids.clone(),
            external_id: self.external_id.clone(),
        }
    }
}

/// Message count for a label/folder.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MessageCount {
    /// Label ID.
    #[serde(rename = "LabelID")]
    pub label_id: LabelId,

    /// Total message count.
    pub total: u32,

    /// Unread message count.
    pub unread: u32,
}

/// Filter for message listing.
#[derive(Debug, Serialize, Default, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MessageFilter {
    /// Filter by label/folder.
    #[serde(rename = "LabelID", skip_serializing_if = "Option::is_none")]
    pub label_id: Option<String>,

    /// Page number (0-indexed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,

    /// Page size (default 50, max 150).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,

    /// Sort descending by time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desc: Option<Boolean>,

    /// End ID for pagination.
    #[serde(rename = "EndID", skip_serializing_if = "Option::is_none")]
    pub end_id: Option<String>,

    /// Filter by subject.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,

    /// Filter by address ID.
    #[serde(rename = "AddressID", skip_serializing_if = "Option::is_none")]
    pub address_id: Option<String>,
}

impl MessageFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_label(mut self, label_id: impl Into<String>) -> Self {
        self.label_id = Some(label_id.into());
        self
    }

    pub fn with_page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    pub fn with_page_size(mut self, size: u32) -> Self {
        self.page_size = Some(size.min(150)); // API max is 150
        self
    }

    pub fn descending(mut self) -> Self {
        self.desc = Some(Boolean::True);
        self
    }
}

// ============================================================================
// Message sending types
// ============================================================================

/// Draft message for creation (POST /mail/v4/messages).
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DraftMessage {
    /// Subject line.
    pub subject: String,

    /// Sender address.
    pub sender: MessageAddress,

    /// Primary recipients (To:).
    #[serde(rename = "ToList")]
    pub to_list: Vec<MessageAddress>,

    /// Carbon copy recipients (CC:).
    #[serde(rename = "CCList")]
    pub cc_list: Vec<MessageAddress>,

    /// Blind carbon copy recipients (BCC:).
    #[serde(rename = "BCCList")]
    pub bcc_list: Vec<MessageAddress>,

    /// Message body (encrypted for sending).
    pub body: String,

    /// MIME type of the body.
    #[serde(rename = "MIMEType")]
    pub mime_type: String,
}

impl DraftMessage {
    /// Create a new draft message.
    pub fn new(
        subject: impl Into<String>,
        sender: MessageAddress,
        to_list: Vec<MessageAddress>,
        body: impl Into<String>,
        is_html: bool,
    ) -> Self {
        Self {
            subject: subject.into(),
            sender,
            to_list,
            cc_list: Vec::new(),
            bcc_list: Vec::new(),
            body: body.into(),
            mime_type: if is_html {
                MimeType::TEXT_HTML.to_string()
            } else {
                MimeType::TEXT_PLAIN.to_string()
            },
        }
    }

    /// Add CC recipients.
    pub fn with_cc(mut self, cc: Vec<MessageAddress>) -> Self {
        self.cc_list = cc;
        self
    }

    /// Add BCC recipients.
    pub fn with_bcc(mut self, bcc: Vec<MessageAddress>) -> Self {
        self.bcc_list = bcc;
        self
    }
}

/// Message package for sending (contains encrypted message for recipients).
/// This is used when sending a message via POST /mail/v4/messages/{id}.
///
/// Note: The actual encryption and package creation requires PGP key operations
/// which will be implemented separately. For now, this provides the structure.
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MessagePackage {
    /// Map of recipient email addresses to their encrypted body key packets.
    #[serde(rename = "Addresses")]
    pub addresses: HashMap<String, MessagePackageAddress>,

    /// Type of recipient (1=internal, 2=external, 4=encrypted_to_outside).
    #[serde(rename = "Type")]
    pub package_type: u8,

    /// Encrypted message body for this package type.
    pub body: String,

    /// MIME type of the body.
    #[serde(rename = "MIMEType")]
    pub mime_type: String,
}

/// Address-specific package data (key packets for a recipient).
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MessagePackageAddress {
    /// Encrypted body session key packet for this recipient.
    #[serde(rename = "BodyKeyPacket")]
    pub body_key_packet: String,

    /// Attachment key packets (if any).
    #[serde(rename = "AttachmentKeyPackets", skip_serializing_if = "HashMap::is_empty")]
    pub attachment_key_packets: HashMap<String, String>,
}

impl MessagePackageAddress {
    pub fn new(body_key_packet: impl Into<String>) -> Self {
        Self {
            body_key_packet: body_key_packet.into(),
            attachment_key_packets: HashMap::new(),
        }
    }
}
