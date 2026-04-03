use crate::db::Db;
use crate::error::DbError;
use crate::models::section::Section;

pub async fn add_child(db: &Db, parent_id: &str, child_id: &str) -> Result<(), DbError> {
    sqlx::query(
        "INSERT INTO Subsections (parent_section_id, child_section_id) VALUES (?, ?)",
    )
    .bind(parent_id)
    .bind(child_id)
    .execute(&db.pool)
    .await?;
    Ok(())
}

pub async fn remove_child(db: &Db, parent_id: &str, child_id: &str) -> Result<(), DbError> {
    sqlx::query(
        "DELETE FROM Subsections WHERE parent_section_id = ? AND child_section_id = ?",
    )
    .bind(parent_id)
    .bind(child_id)
    .execute(&db.pool)
    .await?;
    Ok(())
}

/// Returns the current revision of each child section, ordered by layout_order.
pub async fn get_children(db: &Db, parent_id: &str) -> Result<Vec<Section>, DbError> {
    let rows = sqlx::query_as::<_, Section>(
        "SELECT s.*
         FROM Sections s
         JOIN Subsections sub ON sub.child_section_id = s.section_id
         WHERE sub.parent_section_id = ?
           AND s.id = (
               SELECT MAX(s2.id)
               FROM Sections s2
               WHERE s2.section_id = s.section_id
           )
         ORDER BY s.layout_order",
    )
    .bind(parent_id)
    .fetch_all(&db.pool)
    .await?;
    Ok(rows)
}

/// Returns the current revision of the parent section, or None if top-level.
pub async fn get_parent(db: &Db, child_id: &str) -> Result<Option<Section>, DbError> {
    let row = sqlx::query_as::<_, Section>(
        "SELECT s.*
         FROM Sections s
         JOIN Subsections sub ON sub.parent_section_id = s.section_id
         WHERE sub.child_section_id = ?
           AND s.id = (
               SELECT MAX(s2.id)
               FROM Sections s2
               WHERE s2.section_id = s.section_id
           )
         LIMIT 1",
    )
    .bind(child_id)
    .fetch_optional(&db.pool)
    .await?;
    Ok(row)
}

/// Returns the current revision of all top-level sections in a document
/// (sections that do not appear as a child_section_id in Subsections),
/// ordered by layout_order.
pub async fn get_top_level(db: &Db, document_id: i64) -> Result<Vec<Section>, DbError> {
    let rows = sqlx::query_as::<_, Section>(
        "SELECT s.*
         FROM Sections s
         JOIN SectionIdentities si ON si.section_id = s.section_id
         WHERE si.document_id = ?
           AND s.section_id NOT IN (
               SELECT child_section_id FROM Subsections
           )
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
    use crate::ops::{documents, sections};

    async fn setup() -> Db {
        let db = Db::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        db
    }

    /// Creates a document and two unrelated top-level sections; returns
    /// (doc_id, parent_section_id, child_section_id).
    async fn make_parent_child(db: &Db) -> (i64, String, String) {
        let doc_id = documents::create(db, "Doc", None).await.unwrap().id;
        let (parent, _) = sections::create_section(db, doc_id, None, 1.0, "Parent", "")
            .await
            .unwrap();
        let (child, _) = sections::create_section(db, doc_id, None, 2.0, "Child", "")
            .await
            .unwrap();
        (doc_id, parent.section_id, child.section_id)
    }

    #[tokio::test]
    async fn add_child_creates_relationship() {
        let db = setup().await;
        let (_, parent_id, child_id) = make_parent_child(&db).await;
        add_child(&db, &parent_id, &child_id).await.unwrap();

        let children = get_children(&db, &parent_id).await.unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].section_id, child_id);
    }

    #[tokio::test]
    async fn remove_child_deletes_relationship() {
        let db = setup().await;
        let (_, parent_id, child_id) = make_parent_child(&db).await;
        add_child(&db, &parent_id, &child_id).await.unwrap();
        remove_child(&db, &parent_id, &child_id).await.unwrap();

        let children = get_children(&db, &parent_id).await.unwrap();
        assert!(children.is_empty());
    }

    #[tokio::test]
    async fn remove_child_nonexistent_is_ok() {
        let db = setup().await;
        let (_, parent_id, child_id) = make_parent_child(&db).await;
        remove_child(&db, &parent_id, &child_id).await.unwrap();
    }

    #[tokio::test]
    async fn get_children_returns_current_revision_ordered_by_layout_order() {
        let db = setup().await;
        let doc_id = documents::create(&db, "Doc", None).await.unwrap().id;
        let (parent, _) =
            sections::create_section(&db, doc_id, None, 1.0, "Parent", "").await.unwrap();
        let (c1, _) =
            sections::create_section(&db, doc_id, None, 2.0, "Child2", "").await.unwrap();
        let (c2, _) =
            sections::create_section(&db, doc_id, None, 1.0, "Child1", "").await.unwrap();

        add_child(&db, &parent.section_id, &c1.section_id).await.unwrap();
        add_child(&db, &parent.section_id, &c2.section_id).await.unwrap();

        // Revise c1 so we confirm current version is returned
        sections::revise_section(&db, &c1.section_id, "Child2 revised", 2.0, "").await.unwrap();

        let children = get_children(&db, &parent.section_id).await.unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].name, "Child1");     // layout_order 1.0
        assert_eq!(children[1].name, "Child2 revised"); // layout_order 2.0, latest rev
    }

    #[tokio::test]
    async fn get_children_returns_empty_for_leaf() {
        let db = setup().await;
        let doc_id = documents::create(&db, "Doc", None).await.unwrap().id;
        let (section, _) =
            sections::create_section(&db, doc_id, None, 1.0, "Leaf", "").await.unwrap();

        let children = get_children(&db, &section.section_id).await.unwrap();
        assert!(children.is_empty());
    }

    #[tokio::test]
    async fn get_parent_returns_parent_section() {
        let db = setup().await;
        let (_, parent_id, child_id) = make_parent_child(&db).await;
        add_child(&db, &parent_id, &child_id).await.unwrap();

        let parent = get_parent(&db, &child_id).await.unwrap();
        assert!(parent.is_some());
        assert_eq!(parent.unwrap().section_id, parent_id);
    }

    #[tokio::test]
    async fn get_parent_returns_none_for_top_level() {
        let db = setup().await;
        let doc_id = documents::create(&db, "Doc", None).await.unwrap().id;
        let (section, _) =
            sections::create_section(&db, doc_id, None, 1.0, "Top", "").await.unwrap();

        let parent = get_parent(&db, &section.section_id).await.unwrap();
        assert!(parent.is_none());
    }

    #[tokio::test]
    async fn get_top_level_excludes_children() {
        let db = setup().await;
        let doc_id = documents::create(&db, "Doc", None).await.unwrap().id;
        let (p, _) = sections::create_section(&db, doc_id, None, 1.0, "P", "").await.unwrap();
        // Create child via create_section so the Subsections row is inserted atomically
        let (c, _) =
            sections::create_section(&db, doc_id, Some(&p.section_id), 1.0, "C", "")
                .await
                .unwrap();
        let (q, _) = sections::create_section(&db, doc_id, None, 2.0, "Q", "").await.unwrap();

        let top = get_top_level(&db, doc_id).await.unwrap();
        let ids: Vec<&str> = top.iter().map(|s| s.section_id.as_str()).collect();
        assert!(ids.contains(&p.section_id.as_str()));
        assert!(ids.contains(&q.section_id.as_str()));
        assert!(!ids.contains(&c.section_id.as_str()));
        assert_eq!(top.len(), 2);
    }

    #[tokio::test]
    async fn get_top_level_ordered_by_layout_order() {
        let db = setup().await;
        let doc_id = documents::create(&db, "Doc", None).await.unwrap().id;
        sections::create_section(&db, doc_id, None, 3.0, "C", "").await.unwrap();
        sections::create_section(&db, doc_id, None, 1.0, "A", "").await.unwrap();
        sections::create_section(&db, doc_id, None, 2.0, "B", "").await.unwrap();

        let top = get_top_level(&db, doc_id).await.unwrap();
        assert_eq!(top[0].name, "A");
        assert_eq!(top[1].name, "B");
        assert_eq!(top[2].name, "C");
    }
}
