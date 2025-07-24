use crate::{database::{Database, SyncDatabase}, sync::{error::SyncError, model::{Note, NoteCreate, NoteImport, NoteUpdate, SyncRequest, SyncResponse}}};



pub struct SyncService {
    db: Database,
}

impl SyncService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create_note(&self, user_id: &str, note_id: &str, note: NoteCreate) -> Result<(), SyncError> {
        self.db.create_note(user_id, note_id, &note).await
    }

    pub async fn import_note(&self, user_id: &str, note_id: &str, note: NoteImport) -> Result<(), SyncError> {
        self.db.import_note(user_id, note_id, &note).await
    }

    pub async fn get_note(&self, user_id: &str, note_id: &str) -> Result<Note, SyncError> {
        self.db.get_note(user_id, note_id).await
    }

    pub async fn update_note(&self, user_id: &str, note_id: &str, update: NoteUpdate) -> Result<Note, SyncError> {
        self.db.update_note(user_id, note_id, update).await
    }

    pub async fn delete_note(&self, user_id: &str, note_id: &str) -> Result<(), SyncError> {
        self.db.delete_note(user_id, note_id).await
    }

    pub async fn sync_notes(&self, user_id: &str, sync_request: SyncRequest) -> Result<SyncResponse, SyncError> {
        let last_sync_time = sync_request.last_sync_time.unwrap_or(chrono::Utc::now() - chrono::Duration::days(365));
        let now = chrono::Utc::now();

        let (notes, deleted_note_ids) = self.db.get_sync_notes(user_id, last_sync_time).await?;

        Ok(SyncResponse {
            notes, deleted_note_ids, current_time: now
        })
    }
}