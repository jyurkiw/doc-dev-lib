use uuid::Uuid;

use crate::db::Db;
use crate::error::DbError;
use crate::models::section::{Section, SectionIdentity};

/// Creates a new logical section (SectionIdentity + first Sections revision),
/// optionally attaching it as a child of `parent_section_id`. All in one transaction.
pub async fn create_section(
    db: &Db,
    document_id: i64,
    parent_section_id: Option<&str>,
    layout_order: f64,
    name: &str,
    content: &str,
) -> Result<(SectionIdentity, Section), DbError> {
    let section_id = Uuid::new_v4().to_string();

    let mut tx = db.pool.begin().await?;

    sqlx::query("INSERT INTO SectionIdentities (section_id, document_id) VALUES (?, ?)")
        .bind(&section_id)
        .bind(document_id)
        .execute(&mut *tx)
        .await?;

    let section = sqlx::query_as::<_, Section>(
        "INSERT INTO Sections (section_id, layout_order, name, content)
         VALUES (?, ?, ?, ?) RETURNING *",
    )
    .bind(&section_id)
    .bind(layout_order)
    .bind(name)
    .bind(content)
    .fetch_one(&mut *tx)
    .await?;

    if let Some(parent_id) = parent_section_id {
        sqlx::query(
            "INSERT INTO Subsections (parent_section_id, child_section_id) VALUES (?, ?)",
        )
        .bind(parent_id)
        .bind(&section_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let identity = SectionIdentity {
        section_id,
        document_id,
    };

    Ok((identity, section))
}

/// Appends a new revision of an existing section. Never updates; always inserts.
pub async fn revise_section(
    db: &Db,
    section_id: &str,
    name: &str,
    layout_order: f64,
    content: &str,
) -> Result<Section, DbError> {
    let row = sqlx::query_as::<_, Section>(
        "INSERT INTO Sections (section_id, layout_order, name, content)
         VALUES (?, ?, ?, ?) RETURNING *",
    )
    .bind(section_id)
    .bind(layout_order)
    .bind(name)
    .bind(content)
    .fetch_one(&db.pool)
    .await?;
    Ok(row)
}

/// Returns the most recent revision of a section, or None if not found.
pub async fn get_current(db: &Db, section_id: &str) -> Result<Option<Section>, DbError> {
    let row = sqlx::query_as::<_, Section>(
        "SELECT * FROM Sections WHERE section_id = ?
         ORDER BY id DESC LIMIT 1",
    )
    .bind(section_id)
    .fetch_optional(&db.pool)
    .await?;
    Ok(row)
}

/// Returns all revisions of a section, newest first.
pub async fn get_history(db: &Db, section_id: &str) -> Result<Vec<Section>, DbError> {
    let rows = sqlx::query_as::<_, Section>(
        "SELECT * FROM Sections WHERE section_id = ? ORDER BY id DESC",
    )
    .bind(section_id)
    .fetch_all(&db.pool)
    .await?;
    Ok(rows)
}

/// Returns the current revision for every section belonging to a document.
pub async fn list_current_for_document(
    db: &Db,
    document_id: i64,
) -> Result<Vec<Section>, DbError> {
    let rows = sqlx::query_as::<_, Section>(
        "SELECT s.*
         FROM Sections s
         JOIN SectionIdentities si ON si.section_id = s.section_id
         WHERE si.document_id = ?
           AND s.id = (
               SELECT MAX(s2.id)
               FROM Sections s2
               WHERE s2.section_id = s.section_id
           )
         ORDER BY s.layout_order",
    )
    .bind(document_id)
    .fetch_all(&db.pool)
    .await?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;
    use crate::ops::documents;

    async fn setup() -> Db {
        let db = Db::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        db
    }

    async fn make_doc(db: &Db) -> i64 {
        documents::create(db, "Test Doc", None).await.unwrap().id
    }

    #[tokio::test]
    async fn create_section_top_level() {
        let db = setup().await;
        let doc_id = make_doc(&db).await;
        let (identity, section) =
            create_section(&db, doc_id, None, 1.0, "Intro", "body").await.unwrap();
        assert_eq!(identity.document_id, doc_id);
        assert!(!identity.section_id.is_empty());
        assert_eq!(section.name, "Intro");
        assert_eq!(section.layout_order, 1.0);
        assert_eq!(section.content, "body");
        assert!(section.id > 0);
    }

    #[tokio::test]
    async fn create_section_with_parent_inserts_subsection_row() {
        let db = setup().await;
        let doc_id = make_doc(&db).await;
        let (parent_identity, _) =
            create_section(&db, doc_id, None, 1.0, "Parent", "").await.unwrap();
        let (child_identity, _) =
            create_section(&db, doc_id, Some(&parent_identity.section_id), 1.0, "Child", "")
                .await
                .unwrap();

        // Verify the Subsections row was created by querying through get_children
        let children = crate::ops::subsections::get_children(&db, &parent_identity.section_id)
            .await
            .unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].section_id, child_identity.section_id);
    }

    #[tokio::test]
    async fn revise_section_appends_new_row() {
        let db = setup().await;
        let doc_id = make_doc(&db).await;
        let (identity, first) =
            create_section(&db, doc_id, None, 1.0, "Original", "v1").await.unwrap();

        let revised =
            revise_section(&db, &identity.section_id, "Updated", 1.0, "v2").await.unwrap();

        assert_ne!(revised.id, first.id);
        assert_eq!(revised.name, "Updated");
        assert_eq!(revised.content, "v2");
        assert_eq!(revised.section_id, identity.section_id);
    }

    #[tokio::test]
    async fn get_current_returns_latest_revision() {
        let db = setup().await;
        let doc_id = make_doc(&db).await;
        let (identity, _) =
            create_section(&db, doc_id, None, 1.0, "First", "v1").await.unwrap();
        revise_section(&db, &identity.section_id, "Second", 1.0, "v2").await.unwrap();
        revise_section(&db, &identity.section_id, "Third", 1.0, "v3").await.unwrap();

        let current = get_current(&db, &identity.section_id).await.unwrap().unwrap();
        assert_eq!(current.name, "Third");
        assert_eq!(current.content, "v3");
    }

    #[tokio::test]
    async fn get_current_returns_none_for_unknown_id() {
        let db = setup().await;
        let result = get_current(&db, "00000000-0000-0000-0000-000000000000").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_history_returns_all_revisions_newest_first() {
        let db = setup().await;
        let doc_id = make_doc(&db).await;
        let (identity, _) =
            create_section(&db, doc_id, None, 1.0, "Rev1", "").await.unwrap();
        revise_section(&db, &identity.section_id, "Rev2", 1.0, "").await.unwrap();
        revise_section(&db, &identity.section_id, "Rev3", 1.0, "").await.unwrap();

        let history = get_history(&db, &identity.section_id).await.unwrap();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].name, "Rev3");
        assert_eq!(history[2].name, "Rev1");
    }

    #[tokio::test]
    async fn list_current_for_document_returns_one_per_section() {
        let db = setup().await;
        let doc_id = make_doc(&db).await;
        let (id_a, _) = create_section(&db, doc_id, None, 1.0, "A", "").await.unwrap();
        let (_id_b, _) = create_section(&db, doc_id, None, 2.0, "B", "").await.unwrap();
        // Revise A — list should still return only 2 rows
        revise_section(&db, &id_a.section_id, "A revised", 1.0, "").await.unwrap();

        let current = list_current_for_document(&db, doc_id).await.unwrap();
        assert_eq!(current.len(), 2);
        let names: Vec<&str> = current.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"A revised"));
        assert!(names.contains(&"B"));
    }

    #[tokio::test]
    async fn list_current_for_document_ordered_by_layout_order() {
        let db = setup().await;
        let doc_id = make_doc(&db).await;
        create_section(&db, doc_id, None, 3.0, "C", "").await.unwrap();
        create_section(&db, doc_id, None, 1.0, "A", "").await.unwrap();
        create_section(&db, doc_id, None, 2.0, "B", "").await.unwrap();

        let current = list_current_for_document(&db, doc_id).await.unwrap();
        assert_eq!(current[0].name, "A");
        assert_eq!(current[1].name, "B");
        assert_eq!(current[2].name, "C");
    }

    #[tokio::test]
    async fn list_current_for_document_excludes_other_documents() {
        let db = setup().await;
        let doc1 = make_doc(&db).await;
        let doc2 = make_doc(&db).await;
        create_section(&db, doc1, None, 1.0, "Doc1 Section", "").await.unwrap();
        create_section(&db, doc2, None, 1.0, "Doc2 Section", "").await.unwrap();

        let current = list_current_for_document(&db, doc1).await.unwrap();
        assert_eq!(current.len(), 1);
        assert_eq!(current[0].name, "Doc1 Section");
    }
}
