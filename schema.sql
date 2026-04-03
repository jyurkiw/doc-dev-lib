PRAGMA foreign_keys = ON;

-- Top-level entities

CREATE TABLE Authors (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL,
    description TEXT
);

CREATE TABLE Documents (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL,
    description TEXT
);

-- Logical section registry: one row per unique section identity.
-- Anchors all version history and hierarchy relationships.
-- Needed so that section_id can be used as a proper FK target
-- despite being non-unique in the versioned Sections table.

CREATE TABLE SectionIdentities (
    section_id  TEXT    PRIMARY KEY,   -- UUID assigned at section creation
    document_id INTEGER NOT NULL REFERENCES Documents(id)
);

-- Versioned section rows. Each INSERT is a new revision.
-- Current version = MAX(revision_date) for a given section_id.

CREATE TABLE Sections (
    id            INTEGER  PRIMARY KEY AUTOINCREMENT,
    section_id    TEXT     NOT NULL REFERENCES SectionIdentities(section_id),
    "order"       REAL     NOT NULL,           -- float, relative to siblings
    name          TEXT     NOT NULL DEFAULT '',
    revision_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    content       TEXT     NOT NULL DEFAULT ''
);

-- Fast path: retrieve the current version of every section in a document.
CREATE INDEX idx_sections_current
    ON Sections (section_id, revision_date DESC);

-- Parent-child hierarchy between logical sections (version-agnostic).
-- A section with no row in this table as a child is a top-level section.

CREATE TABLE Subsections (
    parent_section_id TEXT NOT NULL REFERENCES SectionIdentities(section_id),
    child_section_id  TEXT NOT NULL REFERENCES SectionIdentities(section_id),
    PRIMARY KEY (parent_section_id, child_section_id)
);

-- Notes are attached to a logical section (section_id).
-- creation_id and resolution_id pin the note to specific version rows.
-- resolution_id IS NULL means the note is unresolved.

CREATE TABLE Notes (
    id            INTEGER  PRIMARY KEY AUTOINCREMENT,
    author_id     INTEGER  NOT NULL REFERENCES Authors(id),
    section_id    TEXT     NOT NULL REFERENCES SectionIdentities(section_id),
    creation_id   INTEGER  NOT NULL REFERENCES Sections(id),
    resolution_id INTEGER           REFERENCES Sections(id),
    note_date     DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    content       TEXT     NOT NULL
);

CREATE INDEX idx_notes_section
    ON Notes (section_id);

-- Efficient query for all unresolved notes (WHERE resolution_id IS NULL).
CREATE INDEX idx_notes_unresolved
    ON Notes (resolution_id)
    WHERE resolution_id IS NULL;
