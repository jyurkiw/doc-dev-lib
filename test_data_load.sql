-- test_data_load.sql
-- Loads the doc-lib-db design plan document as test data.
-- The software's own design documentation is used to exercise every table
-- and relationship in the schema.
--
-- Section hierarchy:
--   1. Context
--   2. Key Design Decisions
--       2.1  Current Version
--       2.2  `level` Column
--       2.3  Subsections References
--       2.4  layout_order Column
--   3. 4NF Design Issue: section_id FK Integrity
--   4. Tables
--       4.1  Authors
--       4.2  Documents
--       4.3  SectionIdentities
--       4.4  Sections
--       4.5  Subsections
--       4.6  Notes
--   5. File to Create
--   6. schema.sql Structure
--   7. Verification
--
-- Notes included:
--   - 1 resolved note  (on section 2.2 — the `level` column decision)
--   - 1 unresolved note (on section 7 — Verification, a follow-up task)

PRAGMA foreign_keys = ON;

BEGIN TRANSACTION;

-- ============================================================
-- Authors
-- ============================================================

INSERT INTO Authors (id, name, description) VALUES
    (1, 'Claude Sonnet 4.6', 'AI assistant; initial design author');

-- ============================================================
-- Documents
-- ============================================================

INSERT INTO Documents (id, name, description) VALUES
    (1, 'doc-lib-db Design Plan',
     'SQLite schema and MCP server library design plan. Stage 1: schema creation.');

-- ============================================================
-- SectionIdentities  (one row per logical section, UUID as PK)
-- ============================================================

-- Top-level sections
INSERT INTO SectionIdentities (section_id, document_id) VALUES
    ('00000000-0000-0000-0000-000000000001', 1),  -- 1.  Context
    ('00000000-0000-0000-0000-000000000002', 1),  -- 2.  Key Design Decisions
    ('00000000-0000-0000-0000-000000000003', 1),  -- 3.  4NF Design Issue
    ('00000000-0000-0000-0000-000000000004', 1),  -- 4.  Tables
    ('00000000-0000-0000-0000-000000000005', 1),  -- 5.  File to Create
    ('00000000-0000-0000-0000-000000000006', 1),  -- 6.  schema.sql Structure
    ('00000000-0000-0000-0000-000000000007', 1);  -- 7.  Verification

-- Subsections of Key Design Decisions (2.x)
INSERT INTO SectionIdentities (section_id, document_id) VALUES
    ('00000000-0000-0000-0000-000000000008', 1),  -- 2.1 Current Version
    ('00000000-0000-0000-0000-000000000009', 1),  -- 2.2 `level` Column
    ('00000000-0000-0000-0000-00000000000a', 1),  -- 2.3 Subsections References
    ('00000000-0000-0000-0000-00000000000b', 1);  -- 2.4 layout_order Column

-- Subsections of Tables (4.x)
INSERT INTO SectionIdentities (section_id, document_id) VALUES
    ('00000000-0000-0000-0000-00000000000c', 1),  -- 4.1 Authors table
    ('00000000-0000-0000-0000-00000000000d', 1),  -- 4.2 Documents table
    ('00000000-0000-0000-0000-00000000000e', 1),  -- 4.3 SectionIdentities table
    ('00000000-0000-0000-0000-00000000000f', 1),  -- 4.4 Sections table
    ('00000000-0000-0000-0000-000000000010', 1),  -- 4.5 Subsections table
    ('00000000-0000-0000-0000-000000000011', 1);  -- 4.6 Notes table

-- ============================================================
-- Sections  (versioned rows; id order matches section_id order)
-- ============================================================
-- revision_date is fixed so test data is deterministic.
-- id values are relied upon by the Notes inserts below.

-- 1. Context  (id=1)
INSERT INTO Sections (section_id, layout_order, name, revision_date, content) VALUES (
    '00000000-0000-0000-0000-000000000001',
    1.0,
    'Context',
    '2026-04-03 12:00:00',
    'The doc-lib-db project is a local document management library backed by SQLite, wrapped in an MCP server. The motivation is to replace Notion as a document drafting ROG (record of ground-truth), protecting against:

- Destructive overwrite from instructional error
- Hallucination drift
- Content duplication from mis-naming
- Platform unavailability

This is stage 1: creating the SQLite schema only.'
);

-- 2. Key Design Decisions  (id=2)
INSERT INTO Sections (section_id, layout_order, name, revision_date, content) VALUES (
    '00000000-0000-0000-0000-000000000002',
    2.0,
    'Key Design Decisions',
    '2026-04-03 12:00:00',
    'Design decisions made during schema planning, based on clarifying questions about ambiguous aspects of the original specification. See subsections for individual decisions.'
);

-- 3. 4NF Design Issue: section_id FK Integrity  (id=3)
INSERT INTO Sections (section_id, layout_order, name, revision_date, content) VALUES (
    '00000000-0000-0000-0000-000000000003',
    3.0,
    '4NF Design Issue: section_id FK Integrity',
    '2026-04-03 12:00:00',
    'Since `section_id` is non-unique in `Sections` (multiple version rows share the same value), SQLite cannot enforce FK constraints on it from other tables. Solution: introduce a lightweight `SectionIdentities` table as the canonical registry of logical sections.

- `SectionIdentities(section_id, document_id)` — one row per logical section
- `Sections`, `Subsections`, and `Notes` all reference `SectionIdentities.section_id` as FK
- This also correctly places `document_id` at the logical level (not duplicated per-version), satisfying 4NF'
);

-- 4. Tables  (id=4)
INSERT INTO Sections (section_id, layout_order, name, revision_date, content) VALUES (
    '00000000-0000-0000-0000-000000000004',
    4.0,
    'Tables',
    '2026-04-03 12:00:00',
    'The schema consists of six tables. See subsections for individual table definitions.'
);

-- 5. File to Create  (id=5)
INSERT INTO Sections (section_id, layout_order, name, revision_date, content) VALUES (
    '00000000-0000-0000-0000-000000000005',
    5.0,
    'File to Create',
    '2026-04-03 12:00:00',
    '- `E:\Repos\doc-lib-db\schema.sql`'
);

-- 6. schema.sql Structure  (id=6)
INSERT INTO Sections (section_id, layout_order, name, revision_date, content) VALUES (
    '00000000-0000-0000-0000-000000000006',
    6.0,
    'schema.sql Structure',
    '2026-04-03 12:00:00',
    '```sql
PRAGMA foreign_keys = ON;

CREATE TABLE Authors ( ... );
CREATE TABLE Documents ( ... );
CREATE TABLE SectionIdentities ( ... );
CREATE TABLE Sections ( ... );
CREATE TABLE Subsections ( ... );
CREATE TABLE Notes ( ... );
```

Includes useful indexes:
- `Sections(section_id, revision_date DESC)` — fast current-version lookups
- `Notes(section_id)` — notes by logical section
- `Notes(resolution_id) WHERE resolution_id IS NULL` — partial index for open notes'
);

-- 7. Verification  (id=7)
INSERT INTO Sections (section_id, layout_order, name, revision_date, content) VALUES (
    '00000000-0000-0000-0000-000000000007',
    7.0,
    'Verification',
    '2026-04-03 12:00:00',
    'After applying the schema:

```bash
sqlite3 db.sqlite < schema.sql
sqlite3 db.sqlite ".tables"
sqlite3 db.sqlite "PRAGMA foreign_key_list(''Notes'');"
```

Verify all six tables are present and all four FK relationships on Notes are reported correctly.'
);

-- 2.1 Current Version  (id=8)
INSERT INTO Sections (section_id, layout_order, name, revision_date, content) VALUES (
    '00000000-0000-0000-0000-000000000008',
    1.0,
    'Current Version',
    '2026-04-03 12:00:00',
    '**Decision:** Current version of a section is determined by `MAX(revision_date)` for a given `section_id`. Append-only design — no `is_current` flag is needed.

Every new revision is a new row in `Sections`. Queries for the current version use:

```sql
SELECT * FROM Sections WHERE section_id = ? ORDER BY revision_date DESC LIMIT 1;
```

**Why:** Simple and safe. An `is_current` flag requires a two-step write (clear old flag, set new flag), which creates a failure window.'
);

-- 2.2 `level` Column  (id=9)
INSERT INTO Sections (section_id, layout_order, name, revision_date, content) VALUES (
    '00000000-0000-0000-0000-000000000009',
    2.0,
    '`level` Column',
    '2026-04-03 12:00:00',
    '**Decision:** The `level` column was dropped from the `Sections` table.

`level` was in the original specification to indicate heading depth (h1/h2/h3). Once the `Subsections` join table was introduced to capture parent-child hierarchy, `level` became fully derivable from tree depth and was therefore redundant.

**Why dropped:** Redundant data creates update anomalies. The join table is the single source of truth for hierarchy.'
);

-- 2.3 Subsections References  (id=10)
INSERT INTO Sections (section_id, layout_order, name, revision_date, content) VALUES (
    '00000000-0000-0000-0000-00000000000a',
    3.0,
    'Subsections References',
    '2026-04-03 12:00:00',
    '**Decision:** The `Subsections` join table references logical `section_id` values (from `SectionIdentities`), not physical row `id` values from `Sections`.

Parent-child relationships are a property of the logical section, not of any specific version. Using `section_id` means the hierarchy does not need to be re-linked each time a section is revised.

**Why:** Version-agnostic hierarchy is simpler to maintain and query.'
);

-- 2.4 layout_order Column  (id=11)
INSERT INTO Sections (section_id, layout_order, name, revision_date, content) VALUES (
    '00000000-0000-0000-0000-00000000000b',
    4.0,
    'layout_order Column',
    '2026-04-03 12:00:00',
    '**Decision:** The `layout_order` column is a `REAL` (float) value, scoped relative to the parent section (or document root for top-level sections).

Using a float allows a new section to be inserted between two existing sections by choosing a midpoint value, without renumbering siblings. Top-level sections are ordered among other top-level sections; subsections are ordered among siblings under the same parent.

**Why float:** Avoids costly re-sequencing on insert. Standard technique (e.g., Jira-style lexorank simplified).'
);

-- 4.1 Authors table  (id=12)
INSERT INTO Sections (section_id, layout_order, name, revision_date, content) VALUES (
    '00000000-0000-0000-0000-00000000000c',
    1.0,
    'Authors Table',
    '2026-04-03 12:00:00',
    '| Column      | Type    | Constraints             |
|-------------|---------|-------------------------|
| id          | INTEGER | PRIMARY KEY AUTOINCREMENT |
| name        | TEXT    | NOT NULL                |
| description | TEXT    |                         |

Top-level entity. An author is any agent (human or AI) that can be attributed to a note.'
);

-- 4.2 Documents table  (id=13)
INSERT INTO Sections (section_id, layout_order, name, revision_date, content) VALUES (
    '00000000-0000-0000-0000-00000000000d',
    2.0,
    'Documents Table',
    '2026-04-03 12:00:00',
    '| Column      | Type    | Constraints             |
|-------------|---------|-------------------------|
| id          | INTEGER | PRIMARY KEY AUTOINCREMENT |
| name        | TEXT    | NOT NULL                |
| description | TEXT    |                         |

Top-level entity. A document is the top-level container for a set of sections.'
);

-- 4.3 SectionIdentities table  (id=14)
INSERT INTO Sections (section_id, layout_order, name, revision_date, content) VALUES (
    '00000000-0000-0000-0000-00000000000e',
    3.0,
    'SectionIdentities Table',
    '2026-04-03 12:00:00',
    '| Column      | Type    | Constraints                        |
|-------------|---------|-------------------------------------|
| section_id  | TEXT    | PRIMARY KEY (UUID)                  |
| document_id | INTEGER | NOT NULL, FK → Documents(id)        |

Logical section registry. One row per unique section identity. Introduced to give `section_id` a unique anchor so it can be referenced as a proper FK target from `Subsections` and `Notes`, despite being non-unique in the versioned `Sections` table.'
);

-- 4.4 Sections table  (id=15)
INSERT INTO Sections (section_id, layout_order, name, revision_date, content) VALUES (
    '00000000-0000-0000-0000-00000000000f',
    4.0,
    'Sections Table',
    '2026-04-03 12:00:00',
    '| Column        | Type     | Constraints                              |
|---------------|----------|-------------------------------------------|
| id            | INTEGER  | PRIMARY KEY AUTOINCREMENT                 |
| section_id    | TEXT     | NOT NULL, FK → SectionIdentities(section_id) |
| layout_order  | REAL     | NOT NULL                                  |
| name          | TEXT     | NOT NULL DEFAULT ''''                      |
| revision_date | DATETIME | NOT NULL DEFAULT CURRENT_TIMESTAMP        |
| content       | TEXT     | NOT NULL DEFAULT ''''                      |

Versioned rows. Each INSERT is a new revision. Current version = MAX(revision_date) for a given section_id.'
);

-- 4.5 Subsections table  (id=16)
INSERT INTO Sections (section_id, layout_order, name, revision_date, content) VALUES (
    '00000000-0000-0000-0000-000000000010',
    5.0,
    'Subsections Table',
    '2026-04-03 12:00:00',
    '| Column            | Type | Constraints                                  |
|-------------------|------|-----------------------------------------------|
| parent_section_id | TEXT | NOT NULL, FK → SectionIdentities(section_id)  |
| child_section_id  | TEXT | NOT NULL, FK → SectionIdentities(section_id)  |
| PRIMARY KEY       | (parent_section_id, child_section_id)         |

Parent-child join table operating on logical section_ids. A section with no row in this table as a child_section_id is a top-level section.'
);

-- 4.6 Notes table  (id=17)
INSERT INTO Sections (section_id, layout_order, name, revision_date, content) VALUES (
    '00000000-0000-0000-0000-000000000011',
    6.0,
    'Notes Table',
    '2026-04-03 12:00:00',
    '| Column        | Type     | Constraints                                  |
|---------------|----------|-----------------------------------------------|
| id            | INTEGER  | PRIMARY KEY AUTOINCREMENT                     |
| author_id     | INTEGER  | NOT NULL, FK → Authors(id)                    |
| section_id    | TEXT     | NOT NULL, FK → SectionIdentities(section_id)  |
| creation_id   | INTEGER  | NOT NULL, FK → Sections(id)                   |
| resolution_id | INTEGER  | FK → Sections(id); NULL if unresolved         |
| note_date     | DATETIME | NOT NULL DEFAULT CURRENT_TIMESTAMP            |
| content       | TEXT     | NOT NULL                                      |

Notes are attached to a logical section (section_id). creation_id and resolution_id pin the note to specific version rows. resolution_id IS NULL means the note is unresolved.'
);

-- ============================================================
-- Subsections  (parent-child pairs on logical section_ids)
-- ============================================================

-- Children of Key Design Decisions (sec 2)
INSERT INTO Subsections (parent_section_id, child_section_id) VALUES
    ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000008'),
    ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000009'),
    ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-00000000000a'),
    ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-00000000000b');

-- Children of Tables (sec 4)
INSERT INTO Subsections (parent_section_id, child_section_id) VALUES
    ('00000000-0000-0000-0000-000000000004', '00000000-0000-0000-0000-00000000000c'),
    ('00000000-0000-0000-0000-000000000004', '00000000-0000-0000-0000-00000000000d'),
    ('00000000-0000-0000-0000-000000000004', '00000000-0000-0000-0000-00000000000e'),
    ('00000000-0000-0000-0000-000000000004', '00000000-0000-0000-0000-00000000000f'),
    ('00000000-0000-0000-0000-000000000004', '00000000-0000-0000-0000-000000000010'),
    ('00000000-0000-0000-0000-000000000004', '00000000-0000-0000-0000-000000000011');

-- ============================================================
-- Notes
-- ============================================================
-- Note 1 (resolved): raised on the `level` column section (id=9)
--   when the question was whether to keep it; resolved in the same
--   version once the decision was made to drop it.
--
-- Note 2 (unresolved): follow-up task on the Verification section (id=7)
--   to add a test exercising the partial index on Notes.resolution_id.

INSERT INTO Notes
    (author_id, section_id, creation_id, resolution_id, note_date, content)
VALUES (
    1,
    '00000000-0000-0000-0000-000000000009',  -- `level` Column section
    9,                                        -- created at Sections.id=9
    9,                                        -- resolved at Sections.id=9 (same revision)
    '2026-04-03 12:05:00',
    'Should we keep `level` as a convenience column for flat rendering (e.g., indented lists without traversing the Subsections tree)?

**Resolved:** No. `level` is fully derivable from tree depth via the Subsections join table. Storing it separately would create an update anomaly whenever the hierarchy changes. Dropped.'
);

INSERT INTO Notes
    (author_id, section_id, creation_id, resolution_id, note_date, content)
VALUES (
    1,
    '00000000-0000-0000-0000-000000000007',  -- Verification section
    7,                                        -- created at Sections.id=7
    NULL,                                     -- unresolved
    '2026-04-03 12:10:00',
    'Add a verification step that inserts a Note with NULL resolution_id and confirms it appears in a query using the idx_notes_unresolved partial index:

```sql
SELECT COUNT(*) FROM Notes WHERE resolution_id IS NULL;
```

Expected result: 1 (this note itself).'
);

COMMIT;
