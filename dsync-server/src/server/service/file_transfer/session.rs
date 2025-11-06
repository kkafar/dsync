#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SessionId(pub i32);

impl Into<i32> for SessionId {
    fn into(self) -> i32 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FileTransferSession {
    pub session_id: SessionId,
}
