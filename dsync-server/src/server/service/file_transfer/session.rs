use dsync_proto::services::file_transfer::TransferInitRequest;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SessionId(pub i32);

impl SessionId {
    pub(crate) const INVALID: Self = SessionId(-1);

    pub(crate) fn is_valid(&self) -> bool {
        self.0 >= 0
    }

    pub(crate) fn raw_id(&self) -> i32 {
        self.0
    }
}

impl From<SessionId> for i32 {
    fn from(val: SessionId) -> Self {
        val.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FileTransferSession {
    pub session_id: SessionId,
    pub transfer_init_request: TransferInitRequest,
}
