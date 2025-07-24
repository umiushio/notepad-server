use actix_web::{web, FromRequest, HttpMessage, HttpResponse, Responder};

use crate::sync::{error::SyncError, model::{NoteCreate, NoteImport, NoteUpdate, SyncRequest}, service::SyncService};
use crate::log_error;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/notes")
            .route("/sync", web::post().to(sync_notes))
            .service(
                web::resource("/{note_id}")
                    .post(create_note)
                    .get(get_note)
                    .put(update_note)
                    .delete(delete_note)
            )
            .service(
                web::resource("/{note_id}/import")
                .post(import_note)
            )
    );
}

async fn create_note(
    sync_service: web::Data<SyncService>,
    user: AuthenticatedUser,
    note_id: web::Path<String>,
    note: web::Json<NoteCreate>,
) -> Result<impl Responder, SyncError> {
    tracing::debug!("Creating new note {} for user {}", note_id, user.0);

    match sync_service.create_note(&user.0, &note_id, note.into_inner()).await {
        Ok(_) => {
            tracing::info!(note_id = %note_id, "Note created successfully");
            Ok(HttpResponse::Created().finish())
        }
        Err(e) => {
            log_error!(e, "Failed to create note");
            Err(e)
        }
    }
}

async fn import_note(
    sync_service: web::Data<SyncService>,
    user: AuthenticatedUser,
    note_id: web::Path<String>,
    note: web::Json<NoteImport>,
) -> Result<impl Responder, SyncError> {
    tracing::debug!("Import note {} for user {}", note_id, user.0);

    match sync_service.import_note(&user.0, &note_id, note.into_inner()).await {
        Ok(_) => {
            tracing::info!(note_id = %note_id, "Note imported successfully");
            Ok(HttpResponse::Ok().finish())
        }
        Err(e) => {
            log_error!(e, "Failed to import note");
            Err(e)
        }
    }
    
}

async fn get_note(
    sync_service: web::Data<SyncService>,
    user: AuthenticatedUser,
    note_id: web::Path<String>,
) -> Result<impl Responder, SyncError> {
    tracing::debug!("Get note {} for user {}", note_id, user.0);

    match sync_service.get_note(&user.0, &note_id).await {
        Ok(note) => {
            tracing::info!(note_id = %note_id, "Note gotten successfully");
            Ok(HttpResponse::Ok().json(note))
        }
        Err(e) => {
            log_error!(e, "Failed to get note");
            Err(e)
        }
    }
}

async fn update_note(
    sync_service: web::Data<SyncService>,
    user: AuthenticatedUser,
    note_id: web::Path<String>,
    update: web::Json<NoteUpdate>,
) -> Result<impl Responder, SyncError> {
    tracing::debug!("Update note {} for user {}", note_id, user.0);

    match sync_service.update_note(&user.0, &note_id, update.into_inner()).await {
        Ok(note) => {
            tracing::info!(note_id = %note_id, "Note updated succuessfully");
            Ok(HttpResponse::Ok().json(note))
        }
        Err(e) => {
            log_error!(e, "Update note failed");
            Err(e)
        }
    }
}

async fn delete_note(
    sync_service: web::Data<SyncService>,
    user: AuthenticatedUser,
    note_id: web::Path<String>,
) -> Result<impl Responder, SyncError> {
    tracing::debug!("Delete note {} for user {}", note_id, user.0);

    match sync_service.delete_note(&user.0, &note_id).await {
        Ok(_) => {
            tracing::info!(note_id = %note_id, "Note deleted successfully");
            Ok(HttpResponse::Ok().finish())
        }
        Err(e) => {
            log_error!(e, "Delete note failed");
            Err(e)
        }
    }
}

async fn sync_notes(
    sync_service: web::Data<SyncService>,
    user: AuthenticatedUser,
    sync_request: web::Json<SyncRequest>,
) -> Result<impl Responder, SyncError> {
    tracing::info!(
        user_id = %user.0,
        last_sync = ?sync_request.last_sync_time,
        "Starting notes sync"
    );

    match sync_service.sync_notes(&user.0, sync_request.into_inner()).await {
        Ok(response ) => {
            tracing::info!(
                notes_count = response.notes.len(),
                deleted_count = response.deleted_note_ids.len(),
                "Sync completed"
            );
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            log_error!(e, "Sync failed");
            Err(e)
        }
    }
}

use std::future::{ready, Ready};

struct AuthenticatedUser(String);

impl FromRequest for AuthenticatedUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let user_id = req.extensions().get::<String>().cloned();
        match user_id {
            Some(id) => ready(Ok(Self(id))),
            None => ready(Err(Self::Error::from(SyncError::Unauthorized))),
        }
    }
}