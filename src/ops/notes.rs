use crate::db::Db;
use crate::error::DbError;
use crate::models::note::Note;

pub async fn create(
    db: &Db,
    author_id: i64,
    section_id: &str,
    creation_id: i64,
    content: &str,
) -> Result<Note, DbError> {
    let row = sqlx::query_as::<_, Note>(
        "INSERT INTO Notes (author_id, section_id, creation_id, content)
         VALUES (?, ?, ?, ?) RETURNING *",
    )
    .bind(author_id)
    .bind(section_id)
    .bind(creation_id)
    .bind(content)
    .fetch_one(&db.pool)
    .await?;
    Ok(row)
}

/// Returns a single note by its primary key, or None if not found.
pub async fn get(db: &Db, note_id: i64) -> Result<Option<Note>, DbError> {
    let row = sqlx::query_as::<_, Note>("SELECT * FROM Notes WHERE id = ?")
        .bind(note_id)
        .fetch_optional(&db.pool)
        .await?;
    Ok(row)
}

/// Marks a note as resolved by setting resolution_id to the given Sections.id.
pub async fn resolve(db: &Db, note_id: i64, resolution_id: i64) -> Result<Note, DbError> {
    let row = sqlx::query_as::<_, Note>(
        "UPDATE Notes SET resolution_id = ? WHERE id = ? RETURNING *",
    )
    .bind(resolution_id)
    .bind(note_id)
    .fetch_optional(&db.pool)
    .await?
    .ok_or(DbError::NotFound)?;
    Ok(row)
}

/// Returns all notes attached to a logical section (uses idx_notes_section).
pub async fn list_for_section(db: &Db, section_id: &str) -> Result<Vec<Note>, DbError> {
    let rows = sqlx::query_as::<_, Note>(
        "SELECT * FROM Notes WHERE section_id = ? ORDER BY note_date",
    )
    .bind(section_id)
    .fetch_all(&db.pool)
    .await?;
    Ok(rows)
}

/// Returns all unresolved notes (uses idx_notes_unresolved partial index).
pub async fn list_unresolved(db: &Db) -> Result<Vec<Note>, DbError> {
    let rows =
        sqlx::query_as::<_, Note>("SELECT * FROM Notes WHERE resolution_id IS NULL ORDER BY note_date")
            .fetch_all(&db.pool)
            .await?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;
    use crate::ops::{authors, documents, sections};

    async fn setup() -> Db {
        let db = Db::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        db
    }

    /// Inserts an author, document, and one section; returns
    /// (author_id, section_id string, sections_row_id).
    async fn make_note_context(db: &Db) -> (i64, String, i64) {
        let author_id = authors::create(db, "Tester", None).await.unwrap().id;
        let doc_id = documents::create(db, "Doc", None).await.unwrap().id;
        let (identity, section) =
            sections::create_section(db, doc_id, None, 1.0, "Section", "content")
                .await
                .unwrap();
        (author_id, identity.section_id, section.id)
    }

    #[tokio::test]
    async fn create_returns_note_with_id() {
        let db = setup().await;
        let (author_id, section_id, creation_id) = make_note_context(&db).await;
        let note = create(&db, author_id, &section_id, creation_id, "a question")
            .await
            .unwrap();
        assert!(note.id > 0);
        assert_eq!(note.author_id, author_id);
        assert_eq!(note.section_id, section_id);
        assert_eq!(note.creation_id, creation_id);
        assert!(note.resolution_id.is_none());
        assert_eq!(note.content, "a question");
    }

    #[tokio::test]
    async fn resolve_sets_resolution_id() {
        let db = setup().await;
        let (author_id, section_id, creation_id) = make_note_context(&db).await;
        let note = create(&db, author_id, &section_id, creation_id, "resolve me")
            .await
            .unwrap();

        let resolved = resolve(&db, note.id, creation_id).await.unwrap();
        assert_eq!(resolved.id, note.id);
        assert_eq!(resolved.resolution_id, Some(creation_id));
    }

    #[tokio::test]
    async fn resolve_returns_not_found_for_missing_note() {
        let db = setup().await;
        let result = resolve(&db, 999, 1).await;
        assert!(matches!(result, Err(DbError::NotFound)));
    }

    #[tokio::test]
    async fn list_for_section_returns_notes_on_that_section() {
        let db = setup().await;
        let (author_id, section_id, creation_id) = make_note_context(&db).await;
        create(&db, author_id, &section_id, creation_id, "note 1").await.unwrap();
        create(&db, author_id, &section_id, creation_id, "note 2").await.unwrap();

        let notes = list_for_section(&db, &section_id).await.unwrap();
        assert_eq!(notes.len(), 2);
    }

    #[tokio::test]
    async fn list_for_section_excludes_other_sections() {
        let db = setup().await;
        let (author_id, section_id, creation_id) = make_note_context(&db).await;
        // Create a second section in the same document
        let doc_id = documents::create(&db, "Doc2", None).await.unwrap().id;
        let (other_identity, other_section) =
            sections::create_section(&db, doc_id, None, 1.0, "Other", "")
                .await
                .unwrap();

        create(&db, author_id, &section_id, creation_id, "note on section 1").await.unwrap();
        create(&db, author_id, &other_identity.section_id, other_section.id, "note on section 2")
            .await
            .unwrap();

        let notes = list_for_section(&db, &section_id).await.unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].content, "note on section 1");
    }

    #[tokio::test]
    async fn list_unresolved_returns_only_open_notes() {
        let db = setup().await;
        let (author_id, section_id, creation_id) = make_note_context(&db).await;
        let open = create(&db, author_id, &section_id, creation_id, "open").await.unwrap();
        let closed = create(&db, author_id, &section_id, creation_id, "closed").await.unwrap();
        resolve(&db, closed.id, creation_id).await.unwrap();

        let unresolved = list_unresolved(&db).await.unwrap();
        assert_eq!(unresolved.len(), 1);
        assert_eq!(unresolved[0].id, open.id);
    }

    #[tokio::test]
    async fn list_unresolved_returns_empty_when_all_resolved() {
        let db = setup().await;
        let (author_id, section_id, creation_id) = make_note_context(&db).await;
        let note = create(&db, author_id, &section_id, creation_id, "done").await.unwrap();
        resolve(&db, note.id, creation_id).await.unwrap();

        let unresolved = list_unresolved(&db).await.unwrap();
        assert!(unresolved.is_empty());
    }

    #[tokio::test]
    async fn list_unresolved_returns_empty_when_no_notes() {
        let db = setup().await;
        let unresolved = list_unresolved(&db).await.unwrap();
        assert!(unresolved.is_empty());
    }
}
