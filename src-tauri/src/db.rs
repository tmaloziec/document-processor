use crate::parser::ProcessedDocument;
use crate::Stats;
use rusqlite::{Connection, Result, params};
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                filename TEXT NOT NULL,
                original_path TEXT NOT NULL,
                doc_type TEXT,
                classification_confidence REAL,
                pages INTEGER,
                word_count INTEGER,
                size INTEGER,
                full_text TEXT,
                text_preview TEXT,
                metadata TEXT,
                processed_at TEXT NOT NULL,
                status TEXT DEFAULT 'processed'
            );

            CREATE TABLE IF NOT EXISTS images (
                id TEXT PRIMARY KEY,
                document_id TEXT NOT NULL,
                filename TEXT NOT NULL,
                page INTEGER,
                position_marker TEXT,
                context_before TEXT,
                context_after TEXT,
                ocr_text TEXT,
                ai_description TEXT,
                image_path TEXT,
                thumbnail_path TEXT,
                width INTEGER,
                height INTEGER,
                FOREIGN KEY (document_id) REFERENCES documents(id)
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_documents_processed_at ON documents(processed_at DESC);
            CREATE INDEX IF NOT EXISTS idx_documents_doc_type ON documents(doc_type);
            CREATE INDEX IF NOT EXISTS idx_images_document_id ON images(document_id);
            "#,
        )?;

        Ok(Self { conn })
    }

    pub fn document_exists(&self, original_path: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM documents WHERE original_path = ?1",
            params![original_path],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn clear_duplicates(&self) -> Result<usize> {
        // Keep only the newest entry for each original_path
        let deleted = self.conn.execute(
            r#"
            DELETE FROM documents WHERE id NOT IN (
                SELECT id FROM documents d1
                WHERE processed_at = (
                    SELECT MAX(processed_at) FROM documents d2
                    WHERE d2.original_path = d1.original_path
                )
            )
            "#,
            [],
        )?;
        Ok(deleted)
    }

    pub fn save_document(&self, doc: &ProcessedDocument) -> Result<()> {
        // Check for duplicate by path - update existing instead of creating new
        let existing_id: Option<String> = self.conn.query_row(
            "SELECT id FROM documents WHERE original_path = ?1",
            params![doc.original_path],
            |row| row.get(0),
        ).ok();

        let doc_id = existing_id.unwrap_or_else(|| doc.id.clone());

        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO documents
            (id, filename, original_path, doc_type, classification_confidence,
             pages, word_count, size, full_text, text_preview, metadata, processed_at, status)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
            params![
                doc_id,
                doc.filename,
                doc.original_path,
                doc.doc_type,
                doc.classification_confidence,
                doc.pages,
                doc.word_count,
                doc.size,
                doc.full_text,
                doc.text_preview,
                serde_json::to_string(&doc.metadata).unwrap_or_default(),
                doc.processed_at,
                "processed"
            ],
        )?;

        // Save images
        for img in &doc.images {
            self.conn.execute(
                r#"
                INSERT OR REPLACE INTO images
                (id, document_id, filename, page, position_marker,
                 context_before, context_after, ocr_text, ai_description,
                 image_path, thumbnail_path, width, height)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                "#,
                params![
                    img.id,
                    doc.id,
                    img.filename,
                    img.page,
                    img.position_marker,
                    img.context_before,
                    img.context_after,
                    img.ocr_text,
                    img.ai_description,
                    img.image_path,
                    img.thumbnail_path,
                    img.width,
                    img.height,
                ],
            )?;
        }

        Ok(())
    }

    pub fn get_document(&self, id: &str) -> Result<ProcessedDocument> {
        let mut stmt = self.conn.prepare(
            "SELECT id, filename, original_path, doc_type, classification_confidence,
                    pages, word_count, size, full_text, text_preview, metadata, processed_at
             FROM documents WHERE id = ?1",
        )?;

        let doc = stmt.query_row(params![id], |row| {
            let metadata_str: String = row.get(10)?;
            let metadata = serde_json::from_str(&metadata_str).unwrap_or_default();

            Ok(ProcessedDocument {
                id: row.get(0)?,
                filename: row.get(1)?,
                original_path: row.get(2)?,
                doc_type: row.get(3)?,
                classification_confidence: row.get(4)?,
                pages: row.get(5)?,
                word_count: row.get(6)?,
                size: row.get(7)?,
                full_text: row.get(8)?,
                text_preview: row.get(9)?,
                metadata,
                processed_at: row.get(11)?,
                images: vec![],
            })
        })?;

        // Load images
        let mut doc = doc;
        let mut img_stmt = self.conn.prepare(
            "SELECT id, filename, page, position_marker, context_before, context_after,
                    ocr_text, ai_description, image_path, thumbnail_path, width, height
             FROM images WHERE document_id = ?1",
        )?;

        let images = img_stmt.query_map(params![id], |row| {
            Ok(crate::parser::ExtractedImage {
                id: row.get(0)?,
                filename: row.get(1)?,
                page: row.get(2)?,
                position_marker: row.get(3)?,
                context_before: row.get(4)?,
                context_after: row.get(5)?,
                ocr_text: row.get(6)?,
                ai_description: row.get(7)?,
                image_path: row.get(8)?,
                thumbnail_path: row.get(9)?,
                width: row.get(10)?,
                height: row.get(11)?,
            })
        })?;

        doc.images = images.filter_map(|r| r.ok()).collect();

        Ok(doc)
    }

    pub fn get_recent_documents(&self, limit: u32) -> Result<Vec<ProcessedDocument>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, filename, original_path, doc_type, classification_confidence,
                    pages, word_count, size, NULL, text_preview, metadata, processed_at
             FROM documents
             ORDER BY processed_at DESC
             LIMIT ?1",
        )?;

        let docs = stmt.query_map(params![limit], |row| {
            let metadata_str: String = row.get(10)?;
            let metadata = serde_json::from_str(&metadata_str).unwrap_or_default();

            Ok(ProcessedDocument {
                id: row.get(0)?,
                filename: row.get(1)?,
                original_path: row.get(2)?,
                doc_type: row.get(3)?,
                classification_confidence: row.get(4)?,
                pages: row.get(5)?,
                word_count: row.get(6)?,
                size: row.get(7)?,
                full_text: None,
                text_preview: row.get(9)?,
                metadata,
                processed_at: row.get(11)?,
                images: vec![],
            })
        })?;

        Ok(docs.filter_map(|r| r.ok()).collect())
    }

    pub fn get_stats(&self) -> Result<Stats> {
        let total: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM documents",
            [],
            |row| row.get(0),
        )?;

        let processed: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM documents WHERE status = 'processed'",
            [],
            |row| row.get(0),
        )?;

        let failed: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM documents WHERE status = 'failed'",
            [],
            |row| row.get(0),
        )?;

        Ok(Stats { total, processed, failed })
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let result = stmt.query_row(params![key], |row| row.get(0));

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn delete_all_documents(&self) -> Result<usize> {
        // First delete all images
        self.conn.execute("DELETE FROM images", [])?;
        // Then delete all documents
        let deleted = self.conn.execute("DELETE FROM documents", [])?;
        Ok(deleted)
    }

    pub fn update_document_type(&self, id: &str, doc_type: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE documents SET doc_type = ?1, classification_confidence = 1.0 WHERE id = ?2",
            params![doc_type, id],
        )?;
        Ok(())
    }
}
