use crate::server::service::file_transfer::session::{FileTransferSession, SessionId};

pub(crate) struct FileTransferSessionFactory {
    next_session_id: SessionId,
}

impl FileTransferSessionFactory {
    pub(crate) fn new() -> Self {
        Self {
            next_session_id: SessionId(0),
        }
    }
}

impl FileTransferSessionFactory {
    pub(crate) fn create_session(&mut self) -> FileTransferSession {
        let new_session = FileTransferSession {
            session_id: self.next_session_id,
        };

        self.next_session_id = SessionId(self.next_session_id.0 + 1);

        new_session
    }
}
