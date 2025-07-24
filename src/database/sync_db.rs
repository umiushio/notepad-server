use chrono::{DateTime, Utc};
use crate::sync::{error::SyncError, model::{Note, NoteCreate, NoteImport, NoteRow, NoteUpdate}};
use super::Database;

pub(crate) trait SyncDatabase {
    async fn create_note(&self, user_id: &str, note_id: &str, note: &NoteCreate) -> Result<(), SyncError>;
    async fn import_note(&self, user_id: &str, note_id: &str, note: &NoteImport) -> Result<(), SyncError>;
    async fn get_note(&self, user_id: &str, note_id: &str) -> Result<Note, SyncError>;
    async fn update_note(&self, user_id: &str, note_id: &str, update: NoteUpdate) -> Result<Note, SyncError>;
    async fn delete_note(&self, user_id: &str, note_id: &str) -> Result<(), SyncError>;
    async fn get_sync_notes(&self, user_id: &str, time: DateTime<Utc>) -> Result<(Vec<Note>, Vec<String>), SyncError>;
}

impl SyncDatabase for Database {
    async fn create_note(&self, user_id: &str, note_id: &str, note: &NoteCreate) -> Result<(), SyncError> {
        let mut tx = self.db.begin().await?;
        // 插入主表
        sqlx::query(
            r#"
            INSERT INTO notes (id, user_id, title, content, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(note_id)
        .bind(user_id)
        .bind(&note.title)
        .bind("")
        .bind(note.created_at)
        .bind(note.created_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        println!("create note db success");

        Ok(())
    }

    async fn import_note(&self, user_id: &str, note_id: &str, note: &NoteImport) -> Result<(), SyncError> {
        let mut tx = self.db.begin().await?;
        // 插入主表
        sqlx::query(
            r#"
            INSERT INTO notes (id, user_id, title, content, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE SET
                user_id = EXCLUDED.user_id,
                title = EXCLUDED.title,
                content = EXCLUDED.content,
                created_at = EXCLUDED.created_at,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(note_id)
        .bind(user_id)
        .bind(&note.title)
        .bind(&note.content)
        .bind(note.created_at)
        .bind(note.updated_at)
        .execute(&mut *tx)
        .await?;

        // 先删除旧标签
        sqlx::query(
            "DELETE FROM note_tags WHERE note_id = $1"
        )
        .bind(note_id)
        .execute(&mut *tx)
        .await?;

        // 插入新标签
        for tag in note.tags.iter() {
            sqlx::query(
                "INSERT INTO note_tags (note_id, tag) VALUES ($1, $2)"
            )
            .bind(note_id)
            .bind(tag)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        println!("import note db success");

        Ok(())
    }

    async fn get_note(&self, user_id: &str, note_id: &str) -> Result<Note, SyncError> {
        let note_row = sqlx::query_as::<_, NoteRow>(
            "SELECT * FROM notes WHERE id = $1 AND user_id = $2"
        )
        .bind(note_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;
        
        self.add_note_with_tags(note_row).await
    }

    async fn update_note(&self, user_id: &str, note_id: &str, update: NoteUpdate) -> Result<Note, SyncError> {
        let mut tx = self.db.begin().await?;

        // 更新主表
        let note_row = sqlx::query_as::<_, NoteRow>(
            r#"
            UPDATE notes
            SET
                title = COALESCE($1, title),
                content = COALESCE($2, content),
                updated_at = $3
            WHERE id = $4 AND user_id = $5
            RETURNING *
            "#,
        )
        .bind(update.title)
        .bind(update.content)
        .bind(update.updated_at)
        .bind(note_id)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        // 更新标签
        let updated_note = if let Some(tags) = update.tags {
            let mut note = Note {
                id: note_row.id,
                user_id: note_row.user_id,
                title: note_row.title,
                content: note_row.content,
                tags: Vec::new(),
                created_at: note_row.created_at,
                updated_at: note_row.updated_at,
            };
            
            // 先删除旧标签
            sqlx::query(
                "DELETE FROM note_tags WHERE note_id = $1"
            )
            .bind(note_id)
            .execute(&mut *tx)
            .await?;

            // 插入新标签
            for tag in tags.iter() {
                note.tags.push(tag.clone());
                sqlx::query(
                    "INSERT INTO note_tags (note_id, tag) VALUES ($1, $2)"
                )
                .bind(note_id)
                .bind(tag)
                .execute(&mut *tx)
                .await?;
            }
            note
        } else {
            self.add_note_with_tags(note_row).await?
        };

        tx.commit().await?;
        Ok(updated_note)
    }

    async fn delete_note(&self, user_id: &str, note_id: &str) -> Result<(), SyncError> {
        let mut tx = self.db.begin().await?;

        // 检查笔记是否存在
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM notes WHERE id = $1 AND user_id = $2"
        )
        .bind(note_id)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        if exists == 0 {
            // 不存在直接返回
            tx.commit().await?;
            println!("note {} not exist, skip delete", note_id);
            return Ok(());
        }

        // 记录删除
        sqlx::query(
            "INSERT INTO deleted_notes (note_id, user_id, deleted_at) VALUES ($1, $2, $3)"
        )
        .bind(note_id)
        .bind(user_id)
        .bind(Utc::now())
        .execute(&mut *tx)
        .await?;

        // 删除笔记
        sqlx::query(
            "DELETE FROM notes WHERE id = $1 AND user_id = $2"
        )
        .bind(note_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        println!("delete note db success");
        Ok(())
    }

    async fn get_sync_notes(&self, user_id: &str, time: DateTime<Utc>) -> Result<(Vec<Note>, Vec<String>), SyncError> {
        //  获取变更的笔记
        let notes = sqlx::query_as::<_, NoteRow>(
            r#"
            SELECT n.* FROM notes n
            WHERE n.user_id = $1 AND n.updated_at > $2
            ORDER BY n.updated_at ASC
            "#,
        )
        .bind(user_id)
        .bind(time)
        .fetch_all(&self.db)
        .await?;

        // 获取笔记标签
        let mut notes_with_tags = Vec::new();
        for note_row in notes {
            let note = self.add_note_with_tags(note_row).await?;
            notes_with_tags.push(note);
        }

        // 获取删除的笔记ID
        let deleted_note_ids = sqlx::query_scalar::<_, String>(
            r#"
            SELECT note_id FROM deleted_notes
            WHERE user_id = $1 AND deleted_at > $2
            "#,
        )
        .bind(user_id)
        .bind(time)
        .fetch_all(&self.db)
        .await?;
        
        Ok((notes_with_tags, deleted_note_ids))
    }
}

impl Database {
    async fn add_note_with_tags(&self, note_row: NoteRow) -> Result<Note, SyncError> {
        let tags = sqlx::query_scalar::<_, String>(
            "SELECT tag FROM note_tags WHERE note_id = $1"
        )
        .bind(&note_row.id)
        .fetch_all(&self.db)
        .await?;
        
        Ok(Note {
            id: note_row.id,
            user_id: note_row.user_id,
            title: note_row.title,
            content: note_row.content,
            tags,
            created_at: note_row.created_at,
            updated_at: note_row.updated_at,
        })
    }
}
