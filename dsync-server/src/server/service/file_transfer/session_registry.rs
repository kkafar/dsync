use crate::server::service::file_transfer::session::{FileTransferSession, SessionId};

#[derive(Debug, Clone)]
pub(crate) struct FileTransferSessionRegistry {
    registry: Vec<FileTransferSession>,
}

impl FileTransferSessionRegistry {
    pub(crate) fn new() -> Self {
        Self {
            registry: Vec::new(),
        }
    }
}

impl FileTransferSessionRegistry {
    pub(crate) fn register(&mut self, session: FileTransferSession) {
        self.registry.push(session);
    }

    pub(crate) fn unregister(&mut self, session_id: SessionId) -> bool {
        let maybe_position = self
            .registry
            .iter()
            .position(|session| session.session_id == session_id);

        if let Some(position) = maybe_position {
            self.registry.swap_remove(position);
            return true;
        }
        false
    }

    pub(crate) fn get_session(&self, session_id: SessionId) -> Option<FileTransferSession> {
        self.registry
            .iter()
            .find(|session| session.session_id == session_id)
            .cloned()
    }
}

impl Default for FileTransferSessionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
