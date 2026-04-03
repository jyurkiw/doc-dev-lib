use std::future::Future;
use std::pin::Pin;

use crate::db::Db;
use crate::error::DbError;
use crate::ops::{sections, subsections};

/// Normalizes layout_order for every sibling group within a document.
/// Each group (top-level sections, or children of a given parent) is
/// renumbered 1.0, 2.0, 3.0, … in ascending layout_order order.
/// Sections whose layout_order is already correct are not revised.
pub async fn reorder_cleanup(db: &Db, document_id: i64) -> Result<(), DbError> {
    let top_level = subsections::get_top_level(db, document_id).await?;
    for (i, section) in top_level.iter().enumerate() {
        let new_order = (i + 1) as f64;
        if section.layout_order != new_order {
            sections::revise_section(
                db,
                &section.section_id,
                &section.name,
                new_order,
                &section.content,
            )
            .await?;
        }
        reorder_children(db, &section.section_id).await?;
    }
    Ok(())
}

fn reorder_children<'a>(
    db: &'a Db,
    parent_id: &'a str,
) -> Pin<Box<dyn Future<Output = Result<(), DbError>> + Send + 'a>> {
    Box::pin(async move {
        let children = subsections::get_children(db, parent_id).await?;
        for (i, child) in children.iter().enumerate() {
            let new_order = (i + 1) as f64;
            if child.layout_order != new_order {
                sections::revise_section(
                    db,
                    &child.section_id,
                    &child.name,
                    new_order,
                    &child.content,
                )
                .await?;
            }
            reorder_children(db, &child.section_id).await?;
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;
    use crate::ops::{documents, sections, subsections};

    async fn setup() -> Db {
        let db = Db::open_in_memory().await.unwrap();
        db.initialize().await.unwrap();
        db
    }

    #[tokio::test]
    async fn reorder_cleanup_normalizes_top_level_sections() {
        let db = setup().await;
        let doc_id = documents::create(&db, "Doc", None).await.unwrap().id;
        // Create sections with non-integer layout_orders
        sections::create_section(&db, doc_id, None, 1.5, "A", "").await.unwrap();
        sections::create_section(&db, doc_id, None, 2.7, "B", "").await.unwrap();
        sections::create_section(&db, doc_id, None, 4.1, "C", "").await.unwrap();

        reorder_cleanup(&db, doc_id).await.unwrap();

        let top = subsections::get_top_level(&db, doc_id).await.unwrap();
        assert_eq!(top.len(), 3);
        assert_eq!(top[0].layout_order, 1.0);
        assert_eq!(top[1].layout_order, 2.0);
        assert_eq!(top[2].layout_order, 3.0);
    }

    #[tokio::test]
    async fn reorder_cleanup_normalizes_children() {
        let db = setup().await;
        let doc_id = documents::create(&db, "Doc", None).await.unwrap().id;
        let (parent, _) =
            sections::create_section(&db, doc_id, None, 1.0, "Parent", "").await.unwrap();
        let (c1, _) =
            sections::create_section(&db, doc_id, Some(&parent.section_id), 1.3, "C1", "")
                .await
                .unwrap();
        let (_c2, _) =
            sections::create_section(&db, doc_id, Some(&parent.section_id), 2.8, "C2", "")
                .await
                .unwrap();

        reorder_cleanup(&db, doc_id).await.unwrap();

        let children = subsections::get_children(&db, &parent.section_id).await.unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].layout_order, 1.0);
        assert_eq!(children[1].layout_order, 2.0);
        // Verify names are preserved
        assert_eq!(children[0].section_id, c1.section_id);
    }

    #[tokio::test]
    async fn reorder_cleanup_skips_already_normalized() {
        let db = setup().await;
        let doc_id = documents::create(&db, "Doc", None).await.unwrap().id;
        let (id_a, first_rev) =
            sections::create_section(&db, doc_id, None, 1.0, "A", "").await.unwrap();
        sections::create_section(&db, doc_id, None, 2.0, "B", "").await.unwrap();

        reorder_cleanup(&db, doc_id).await.unwrap();

        // A was already at 1.0 so no new revision should have been created
        let history = sections::get_history(&db, &id_a.section_id).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, first_rev.id);
    }
}
