---
--- Migrations: Bootstrap _migration table for tracking
---
DEFINE TABLE OVERWRITE _migration SCHEMAFULL
	PERMISSIONS NONE;

DEFINE FIELD OVERWRITE file ON _migration
	TYPE string
	COMMENT "Relative path to migration file";

DEFINE FIELD OVERWRITE applied_at ON _migration
	TYPE datetime
	DEFAULT time::now();

---
--- Indexes
---
DEFINE INDEX OVERWRITE by_file ON _migration
	FIELDS file
	COMMENT "Lookup by file name";

DEFINE TABLE OVERWRITE _surrealkit_sync SCHEMAFULL
	PERMISSIONS NONE;

DEFINE FIELD OVERWRITE path ON _surrealkit_sync
	TYPE string;

DEFINE FIELD OVERWRITE hash ON _surrealkit_sync
	TYPE string;

DEFINE FIELD OVERWRITE synced_at ON _surrealkit_sync
	TYPE datetime
	DEFAULT time::now();

DEFINE INDEX OVERWRITE by_path ON _surrealkit_sync
	FIELDS path
	UNIQUE;

DEFINE TABLE OVERWRITE _surrealkit_sync_meta SCHEMAFULL
	PERMISSIONS NONE;

DEFINE FIELD OVERWRITE key ON _surrealkit_sync_meta
	TYPE string;

DEFINE FIELD OVERWRITE value ON _surrealkit_sync_meta
	TYPE any;

DEFINE FIELD OVERWRITE updated_at ON _surrealkit_sync_meta
	TYPE datetime
	DEFAULT time::now();

DEFINE INDEX OVERWRITE by_key ON _surrealkit_sync_meta
	FIELDS key
	UNIQUE;
