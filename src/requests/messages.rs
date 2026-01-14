//! Message API request types.

use crate::domain::{
    DraftMessage, FullMessage, MessageCount, MessageFilter, MessageId, MessageMetadata,
    MessagePackage,
};
use crate::http;
use crate::http::{NoResponse, RequestData};
use serde::{Deserialize, Serialize};

// ============================================================================
// GET /mail/v4/messages - List messages
// ============================================================================

/// Request to list messages with optional filtering.
pub struct GetMessagesRequest {
    filter: MessageFilter,
}

#[doc(hidden)]
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetMessagesResponse {
    pub messages: Vec<MessageMetadata>,
    pub total: u32,
}

impl GetMessagesRequest {
    pub fn new(filter: MessageFilter) -> Self {
        Self { filter }
    }

    /// Create request to list messages in a specific label/folder.
    pub fn for_label(label_id: impl Into<String>) -> Self {
        Self::new(MessageFilter::new().with_label(label_id))
    }
}

impl http::RequestDesc for GetMessagesRequest {
    type Output = GetMessagesResponse;
    type Response = http::JsonResponse<Self::Output>;

    fn build(&self) -> RequestData {
        let mut query_parts = Vec::new();

        if let Some(ref label_id) = self.filter.label_id {
            query_parts.push(format!("LabelID={}", label_id));
        }
        if let Some(page) = self.filter.page {
            query_parts.push(format!("Page={}", page));
        }
        if let Some(page_size) = self.filter.page_size {
            query_parts.push(format!("PageSize={}", page_size));
        }
        if let Some(ref end_id) = self.filter.end_id {
            query_parts.push(format!("EndID={}", end_id));
        }
        if self.filter.desc.is_some() {
            query_parts.push("Desc=1".to_string());
        }

        let path = if query_parts.is_empty() {
            "mail/v4/messages".to_string()
        } else {
            format!("mail/v4/messages?{}", query_parts.join("&"))
        };

        RequestData::new(http::Method::Get, path)
    }
}

// ============================================================================
// GET /mail/v4/messages/{id} - Get single message
// ============================================================================

/// Request to get a single message by ID.
pub struct GetMessageRequest<'a> {
    id: &'a MessageId,
}

#[doc(hidden)]
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetMessageResponse {
    pub message: FullMessage,
}

impl<'a> GetMessageRequest<'a> {
    pub fn new(id: &'a MessageId) -> Self {
        Self { id }
    }
}

impl<'a> http::RequestDesc for GetMessageRequest<'a> {
    type Output = GetMessageResponse;
    type Response = http::JsonResponse<Self::Output>;

    fn build(&self) -> RequestData {
        RequestData::new(http::Method::Get, format!("mail/v4/messages/{}", self.id))
    }
}

// ============================================================================
// PUT /mail/v4/messages/read - Mark messages as read
// ============================================================================

/// Request to mark messages as read.
pub struct MarkMessagesReadRequest<'a> {
    ids: &'a [MessageId],
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct MarkReadBody<'a> {
    #[serde(rename = "IDs")]
    ids: &'a [MessageId],
}

impl<'a> MarkMessagesReadRequest<'a> {
    pub fn new(ids: &'a [MessageId]) -> Self {
        Self { ids }
    }
}

impl<'a> http::RequestDesc for MarkMessagesReadRequest<'a> {
    type Output = ();
    type Response = NoResponse;

    fn build(&self) -> RequestData {
        let body = MarkReadBody { ids: self.ids };
        RequestData::new(http::Method::Put, "mail/v4/messages/read")
            .json(&body)
    }
}

// ============================================================================
// PUT /mail/v4/messages/unread - Mark messages as unread
// ============================================================================

/// Request to mark messages as unread.
pub struct MarkMessagesUnreadRequest<'a> {
    ids: &'a [MessageId],
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct MarkUnreadBody<'a> {
    #[serde(rename = "IDs")]
    ids: &'a [MessageId],
}

impl<'a> MarkMessagesUnreadRequest<'a> {
    pub fn new(ids: &'a [MessageId]) -> Self {
        Self { ids }
    }
}

impl<'a> http::RequestDesc for MarkMessagesUnreadRequest<'a> {
    type Output = ();
    type Response = NoResponse;

    fn build(&self) -> RequestData {
        let body = MarkUnreadBody { ids: self.ids };
        RequestData::new(http::Method::Put, "mail/v4/messages/unread")
            .json(&body)
    }
}

// ============================================================================
// PUT /mail/v4/messages/label - Add label to messages
// ============================================================================

/// Request to add a label to messages.
pub struct LabelMessagesRequest<'a> {
    label_id: &'a str,
    ids: &'a [MessageId],
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct LabelMessagesBody<'a> {
    #[serde(rename = "LabelID")]
    label_id: &'a str,
    #[serde(rename = "IDs")]
    ids: &'a [MessageId],
}

/// Response for label messages operation.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LabelMessagesResponse {
    pub responses: Vec<LabelMessageResult>,
}

/// Result for individual message labeling.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LabelMessageResult {
    #[serde(rename = "ID")]
    pub id: MessageId,
    pub response: LabelMessageStatus,
}

/// Status of label operation.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LabelMessageStatus {
    pub code: u32,
}

impl<'a> LabelMessagesRequest<'a> {
    pub fn new(label_id: &'a str, ids: &'a [MessageId]) -> Self {
        Self { label_id, ids }
    }
}

impl<'a> http::RequestDesc for LabelMessagesRequest<'a> {
    type Output = LabelMessagesResponse;
    type Response = http::JsonResponse<Self::Output>;

    fn build(&self) -> RequestData {
        let body = LabelMessagesBody {
            label_id: self.label_id,
            ids: self.ids,
        };
        RequestData::new(http::Method::Put, "mail/v4/messages/label")
            .json(&body)
    }
}

// ============================================================================
// PUT /mail/v4/messages/unlabel - Remove label from messages
// ============================================================================

/// Request to remove a label from messages.
pub struct UnlabelMessagesRequest<'a> {
    label_id: &'a str,
    ids: &'a [MessageId],
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct UnlabelMessagesBody<'a> {
    #[serde(rename = "LabelID")]
    label_id: &'a str,
    #[serde(rename = "IDs")]
    ids: &'a [MessageId],
}

impl<'a> UnlabelMessagesRequest<'a> {
    pub fn new(label_id: &'a str, ids: &'a [MessageId]) -> Self {
        Self { label_id, ids }
    }
}

impl<'a> http::RequestDesc for UnlabelMessagesRequest<'a> {
    type Output = LabelMessagesResponse;
    type Response = http::JsonResponse<Self::Output>;

    fn build(&self) -> RequestData {
        let body = UnlabelMessagesBody {
            label_id: self.label_id,
            ids: self.ids,
        };
        RequestData::new(http::Method::Put, "mail/v4/messages/unlabel")
            .json(&body)
    }
}

// ============================================================================
// PUT /mail/v4/messages/delete - Delete messages
// ============================================================================

/// Request to permanently delete messages.
pub struct DeleteMessagesRequest<'a> {
    ids: &'a [MessageId],
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct DeleteMessagesBody<'a> {
    #[serde(rename = "IDs")]
    ids: &'a [MessageId],
}

impl<'a> DeleteMessagesRequest<'a> {
    pub fn new(ids: &'a [MessageId]) -> Self {
        Self { ids }
    }
}

impl<'a> http::RequestDesc for DeleteMessagesRequest<'a> {
    type Output = ();
    type Response = NoResponse;

    fn build(&self) -> RequestData {
        let body = DeleteMessagesBody { ids: self.ids };
        RequestData::new(http::Method::Put, "mail/v4/messages/delete")
            .json(&body)
    }
}

// ============================================================================
// GET /mail/v4/messages/count - Get message counts
// ============================================================================

/// Request to get message counts per label.
pub struct GetMessageCountsRequest;

#[doc(hidden)]
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetMessageCountsResponse {
    pub counts: Vec<MessageCount>,
}

impl http::RequestDesc for GetMessageCountsRequest {
    type Output = GetMessageCountsResponse;
    type Response = http::JsonResponse<Self::Output>;

    fn build(&self) -> RequestData {
        RequestData::new(http::Method::Get, "mail/v4/messages/count")
    }
}

// ============================================================================
// GET /mail/v4/attachments/{id} - Download attachment
// ============================================================================

/// Request to download an attachment by ID.
/// Returns the raw encrypted attachment data.
pub struct GetAttachmentRequest<'a> {
    attachment_id: &'a str,
}

impl<'a> GetAttachmentRequest<'a> {
    pub fn new(attachment_id: &'a str) -> Self {
        Self { attachment_id }
    }
}

impl<'a> http::RequestDesc for GetAttachmentRequest<'a> {
    type Output = Vec<u8>;
    type Response = http::BinaryResponse;

    fn build(&self) -> RequestData {
        RequestData::new(
            http::Method::Get,
            format!("mail/v4/attachments/{}", self.attachment_id),
        )
    }
}

// ============================================================================
// POST /mail/v4/messages - Create draft message
// ============================================================================

/// Request to create a draft message.
/// This creates a new draft that can be edited and later sent.
pub struct CreateDraftRequest<'a> {
    draft: &'a DraftMessage,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct CreateDraftBody<'a> {
    message: &'a DraftMessage,
}

#[doc(hidden)]
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateDraftResponse {
    pub message: FullMessage,
}

impl<'a> CreateDraftRequest<'a> {
    pub fn new(draft: &'a DraftMessage) -> Self {
        Self { draft }
    }
}

impl<'a> http::RequestDesc for CreateDraftRequest<'a> {
    type Output = CreateDraftResponse;
    type Response = http::JsonResponse<Self::Output>;

    fn build(&self) -> RequestData {
        let body = CreateDraftBody {
            message: self.draft,
        };
        RequestData::new(http::Method::Post, "mail/v4/messages").json(&body)
    }
}

// ============================================================================
// PUT /mail/v4/messages/{id} - Update draft message
// ============================================================================

/// Request to update an existing draft message.
pub struct UpdateDraftRequest<'a> {
    id: &'a MessageId,
    draft: &'a DraftMessage,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct UpdateDraftBody<'a> {
    message: &'a DraftMessage,
}

#[doc(hidden)]
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateDraftResponse {
    pub message: FullMessage,
}

impl<'a> UpdateDraftRequest<'a> {
    pub fn new(id: &'a MessageId, draft: &'a DraftMessage) -> Self {
        Self { id, draft }
    }
}

impl<'a> http::RequestDesc for UpdateDraftRequest<'a> {
    type Output = UpdateDraftResponse;
    type Response = http::JsonResponse<Self::Output>;

    fn build(&self) -> RequestData {
        let body = UpdateDraftBody {
            message: self.draft,
        };
        RequestData::new(http::Method::Put, format!("mail/v4/messages/{}", self.id)).json(&body)
    }
}

// ============================================================================
// POST /mail/v4/messages/{id} - Send message
// ============================================================================

/// Request to send a draft message.
/// The draft must already be created using CreateDraftRequest.
pub struct SendMessageRequest<'a> {
    id: &'a MessageId,
    packages: &'a [MessagePackage],
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendMessageBody<'a> {
    packages: &'a [MessagePackage],
}

#[doc(hidden)]
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendMessageResponse {
    pub sent: SentMessage,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SentMessage {
    #[serde(rename = "ID")]
    pub id: MessageId,
}

impl<'a> SendMessageRequest<'a> {
    pub fn new(id: &'a MessageId, packages: &'a [MessagePackage]) -> Self {
        Self { id, packages }
    }
}

impl<'a> http::RequestDesc for SendMessageRequest<'a> {
    type Output = SendMessageResponse;
    type Response = http::JsonResponse<Self::Output>;

    fn build(&self) -> RequestData {
        let body = SendMessageBody {
            packages: self.packages,
        };
        RequestData::new(http::Method::Post, format!("mail/v4/messages/{}", self.id)).json(&body)
    }
}
