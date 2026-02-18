use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::migration::sha256_hex;

pub const SCHEMA_DIR: &str = "database/schema";
pub const MIGRATIONS_DIR: &str = "database/migrations";
pub const STATE_DIR: &str = "database/.surrealkit";
pub const SCHEMA_SNAPSHOT_PATH: &str = "database/.surrealkit/schema_snapshot.json";
pub const CATALOG_SNAPSHOT_PATH: &str = "database/.surrealkit/catalog_snapshot.json";

#[derive(Debug, Clone)]
pub struct SchemaFile {
	pub path: String,
	pub sql: String,
	pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaSnapshot {
	pub version: u32,
	pub files: Vec<SchemaSnapshotEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SchemaSnapshotEntry {
	pub path: String,
	pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogSnapshot {
	pub version: u32,
	pub entities: Vec<EntityKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityKey {
	pub kind: String,
	pub scope: Option<String>,
	pub name: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FileDiff {
	pub added: Vec<String>,
	pub modified: Vec<String>,
	pub removed: Vec<String>,
}

pub fn ensure_local_state_dirs() -> Result<()> {
	fs::create_dir_all(SCHEMA_DIR).with_context(|| format!("creating {}", SCHEMA_DIR))?;
	fs::create_dir_all(MIGRATIONS_DIR).with_context(|| format!("creating {}", MIGRATIONS_DIR))?;
	fs::create_dir_all(STATE_DIR).with_context(|| format!("creating {}", STATE_DIR))?;
	Ok(())
}

pub fn collect_schema_files() -> Result<Vec<SchemaFile>> {
	let mut files: Vec<PathBuf> = WalkDir::new(SCHEMA_DIR)
		.follow_links(true)
		.into_iter()
		.filter_map(|e| e.ok())
		.filter(|e| e.file_type().is_file())
		.map(|e| e.into_path())
		.filter(|p| p.extension().and_then(|s| s.to_str()) == Some("surql"))
		.collect();

	files.sort();

	let mut out = Vec::with_capacity(files.len());
	for path in files {
		let sql = fs::read_to_string(&path).with_context(|| format!("reading {:?}", path))?;
		let hash = sha256_hex(sql.as_bytes());
		let path_str = normalize_path(&path)?;
		out.push(SchemaFile {
			path: path_str,
			sql,
			hash,
		});
	}

	Ok(out)
}

pub fn snapshot_from_files(files: &[SchemaFile]) -> SchemaSnapshot {
	let mut entries: Vec<SchemaSnapshotEntry> = files
		.iter()
		.map(|f| SchemaSnapshotEntry {
			path: f.path.clone(),
			hash: f.hash.clone(),
		})
		.collect();
	entries.sort();
	SchemaSnapshot {
		version: 1,
		files: entries,
	}
}

pub fn load_schema_snapshot() -> Result<SchemaSnapshot> {
	load_json_or_default(
		SCHEMA_SNAPSHOT_PATH,
		SchemaSnapshot {
			version: 1,
			files: Vec::new(),
		},
	)
}

pub fn save_schema_snapshot(snapshot: &SchemaSnapshot) -> Result<()> {
	save_json_pretty(SCHEMA_SNAPSHOT_PATH, snapshot)
}

pub fn load_catalog_snapshot() -> Result<CatalogSnapshot> {
	load_json_or_default(
		CATALOG_SNAPSHOT_PATH,
		CatalogSnapshot {
			version: 1,
			entities: Vec::new(),
		},
	)
}

pub fn save_catalog_snapshot(snapshot: &CatalogSnapshot) -> Result<()> {
	save_json_pretty(CATALOG_SNAPSHOT_PATH, snapshot)
}

pub fn diff_schema(old: &SchemaSnapshot, new: &SchemaSnapshot) -> FileDiff {
	let old_map: BTreeMap<&str, &str> = old
		.files
		.iter()
		.map(|f| (f.path.as_str(), f.hash.as_str()))
		.collect();
	let new_map: BTreeMap<&str, &str> = new
		.files
		.iter()
		.map(|f| (f.path.as_str(), f.hash.as_str()))
		.collect();

	let mut added = Vec::new();
	let mut modified = Vec::new();
	let mut removed = Vec::new();

	for (path, hash) in &new_map {
		match old_map.get(path) {
			None => added.push((*path).to_string()),
			Some(old_hash) if old_hash != hash => modified.push((*path).to_string()),
			_ => {}
		}
	}

	for path in old_map.keys() {
		if !new_map.contains_key(path) {
			removed.push((*path).to_string());
		}
	}

	FileDiff {
		added,
		modified,
		removed,
	}
}

pub fn build_catalog_snapshot(files: &[SchemaFile]) -> CatalogSnapshot {
	let mut entities = BTreeSet::new();
	for file in files {
		for stmt in split_statements(&strip_line_comments(&file.sql)) {
			if let Some(entity) = parse_define_entity(&stmt) {
				entities.insert(entity);
			}
		}
	}

	CatalogSnapshot {
		version: 1,
		entities: entities.into_iter().collect(),
	}
}

pub fn removed_entities(old: &CatalogSnapshot, new: &CatalogSnapshot) -> Vec<EntityKey> {
	let old_set: BTreeSet<_> = old.entities.iter().cloned().collect();
	let new_set: BTreeSet<_> = new.entities.iter().cloned().collect();
	old_set.difference(&new_set).cloned().collect()
}

pub fn render_remove_sql(entities: &[EntityKey], api_supported: bool) -> Result<Vec<String>> {
	let mut out = Vec::new();
	for entity in entities {
		let stmt = match entity.kind.as_str() {
			"table" => format!("REMOVE TABLE {};", entity.name),
			"field" => format!(
				"REMOVE FIELD {} ON {};",
				entity.name,
				scope_or_err(entity, "FIELD")?
			),
			"event" => format!(
				"REMOVE EVENT {} ON {};",
				entity.name,
				scope_or_err(entity, "EVENT")?
			),
			"index" => format!(
				"REMOVE INDEX {} ON {};",
				entity.name,
				scope_or_err(entity, "INDEX")?
			),
			"function" => format!("REMOVE FUNCTION {};", entity.name),
			"param" => format!("REMOVE PARAM {};", entity.name),
			"access" => match &entity.scope {
				Some(scope) => format!("REMOVE ACCESS {} ON {};", entity.name, scope),
				None => format!("REMOVE ACCESS {};", entity.name),
			},
			"analyzer" => format!("REMOVE ANALYZER {};", entity.name),
			"user" => match &entity.scope {
				Some(scope) => format!("REMOVE USER {} ON {};", entity.name, scope),
				None => format!("REMOVE USER {};", entity.name),
			},
			"api" => {
				if api_supported {
					format!("REMOVE API {};", entity.name)
				} else {
					bail!(
						"API removal requested for '{}' but this SurrealDB server does not support `REMOVE API`. \
Use a manual migration or upgrade server support.",
						entity.name
					);
				}
			}
			_ => continue,
		};
		out.push(stmt);
	}
	Ok(out)
}

fn scope_or_err(entity: &EntityKey, object: &str) -> Result<String> {
	entity.scope.clone().ok_or_else(|| {
		anyhow!(
			"cannot render REMOVE {} for '{}' because scope is missing",
			object,
			entity.name
		)
	})
}

fn normalize_path(path: &Path) -> Result<String> {
	let cwd = std::env::current_dir().context("resolving current directory")?;
	let rel = path
		.strip_prefix(&cwd)
		.or_else(|_| path.strip_prefix("."))
		.unwrap_or(path);
	Ok(rel.to_string_lossy().replace('\\', "/"))
}

fn load_json_or_default<T>(path: &str, default: T) -> Result<T>
where
	T: for<'de> Deserialize<'de>,
{
	let p = Path::new(path);
	if !p.exists() {
		return Ok(default);
	}

	let raw = fs::read_to_string(p).with_context(|| format!("reading {}", path))?;
	let parsed = serde_json::from_str(&raw).with_context(|| format!("parsing {}", path))?;
	Ok(parsed)
}

fn save_json_pretty<T>(path: &str, value: &T) -> Result<()>
where
	T: Serialize,
{
	ensure_local_state_dirs()?;
	let raw = serde_json::to_string_pretty(value).context("serializing json")?;
	fs::write(path, format!("{raw}\n")).with_context(|| format!("writing {}", path))?;
	Ok(())
}

fn strip_line_comments(sql: &str) -> String {
	sql.lines()
		.filter(|line| {
			let t = line.trim_start();
			!(t.starts_with("--") || t.starts_with("//"))
		})
		.collect::<Vec<_>>()
		.join("\n")
}

fn split_statements(sql: &str) -> Vec<String> {
	let mut out = Vec::new();
	let mut buf = String::new();
	let mut in_single = false;
	let mut in_double = false;
	let mut in_backtick = false;
	let mut prev_escape = false;

	for ch in sql.chars() {
		match ch {
			'\'' if !in_double && !in_backtick && !prev_escape => in_single = !in_single,
			'"' if !in_single && !in_backtick && !prev_escape => in_double = !in_double,
			'`' if !in_single && !in_double && !prev_escape => in_backtick = !in_backtick,
			';' if !in_single && !in_double && !in_backtick => {
				let stmt = buf.trim();
				if !stmt.is_empty() {
					out.push(stmt.to_string());
				}
				buf.clear();
				prev_escape = false;
				continue;
			}
			_ => {}
		}

		prev_escape = ch == '\\' && !prev_escape;
		buf.push(ch);
	}

	let tail = buf.trim();
	if !tail.is_empty() {
		out.push(tail.to_string());
	}

	out
}

fn parse_define_entity(stmt: &str) -> Option<EntityKey> {
	let tokens = tokenize(stmt);
	if tokens.len() < 3 || !eq(tokens[0], "DEFINE") {
		return None;
	}

	let kind = tokens[1].to_ascii_lowercase();
	let mut idx = 2;
	idx = skip_modifiers(&tokens, idx);
	if idx >= tokens.len() {
		return None;
	}

	match kind.as_str() {
		"table" => Some(EntityKey {
			kind,
			scope: None,
			name: clean_ident(tokens[idx]),
		}),
		"field" | "event" | "index" => {
			let name = clean_ident(tokens[idx]);
			let on_idx = find_token(&tokens, idx + 1, "ON")?;
			let mut scope_idx = on_idx + 1;
			if scope_idx < tokens.len() && eq(tokens[scope_idx], "TABLE") {
				scope_idx += 1;
			}
			if scope_idx >= tokens.len() {
				return None;
			}
			Some(EntityKey {
				kind,
				scope: Some(clean_ident(tokens[scope_idx])),
				name,
			})
		}
		"function" | "param" | "analyzer" | "api" => Some(EntityKey {
			kind,
			scope: None,
			name: clean_ident(tokens[idx]),
		}),
		"access" | "user" => {
			let name = clean_ident(tokens[idx]);
			let scope = find_token(&tokens, idx + 1, "ON").and_then(|on_idx| {
				let i = on_idx + 1;
				if i < tokens.len() {
					Some(clean_ident(tokens[i]))
				} else {
					None
				}
			});

			Some(EntityKey { kind, scope, name })
		}
		_ => None,
	}
}

fn tokenize(stmt: &str) -> Vec<&str> {
	stmt.split_whitespace().collect()
}

fn clean_ident(token: &str) -> String {
	let trimmed = token.trim_matches(|c: char| {
		c == ',' || c == ';' || c == '(' || c == ')' || c == '{' || c == '}'
	});
	let core = match trimmed.find('(') {
		Some(pos) => &trimmed[..pos],
		None => trimmed,
	};
	core.to_string()
}

fn skip_modifiers(tokens: &[&str], mut idx: usize) -> usize {
	while idx < tokens.len()
		&& (eq(tokens[idx], "OVERWRITE")
			|| eq(tokens[idx], "IF")
			|| eq(tokens[idx], "NOT")
			|| eq(tokens[idx], "EXISTS"))
	{
		idx += 1;
	}
	idx
}

fn find_token(tokens: &[&str], start: usize, target: &str) -> Option<usize> {
	(start..tokens.len()).find(|&i| eq(tokens[i], target))
}

fn eq(value: &str, expected: &str) -> bool {
	value.eq_ignore_ascii_case(expected)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn schema_diff_detects_added_modified_removed() {
		let old = SchemaSnapshot {
			version: 1,
			files: vec![
				SchemaSnapshotEntry {
					path: "database/schema/a.surql".to_string(),
					hash: "1".to_string(),
				},
				SchemaSnapshotEntry {
					path: "database/schema/b.surql".to_string(),
					hash: "2".to_string(),
				},
			],
		};
		let new = SchemaSnapshot {
			version: 1,
			files: vec![
				SchemaSnapshotEntry {
					path: "database/schema/b.surql".to_string(),
					hash: "3".to_string(),
				},
				SchemaSnapshotEntry {
					path: "database/schema/c.surql".to_string(),
					hash: "4".to_string(),
				},
			],
		};

		let diff = diff_schema(&old, &new);
		assert_eq!(diff.added, vec!["database/schema/c.surql"]);
		assert_eq!(diff.modified, vec!["database/schema/b.surql"]);
		assert_eq!(diff.removed, vec!["database/schema/a.surql"]);
	}

	#[test]
	fn catalog_extracts_supported_entities() {
		let files = vec![SchemaFile {
			path: "database/schema/root.surql".to_string(),
			hash: "x".to_string(),
			sql: r#"
				DEFINE TABLE OVERWRITE person SCHEMAFULL;
				DEFINE FIELD OVERWRITE name ON person TYPE string;
				DEFINE EVENT changed ON person WHEN true THEN ();
				DEFINE INDEX by_name ON TABLE person FIELDS name;
				DEFINE FUNCTION fn::greet($name: string) { RETURN $name; };
				DEFINE PARAM $env VALUE "dev";
				DEFINE ACCESS admin ON DATABASE TYPE RECORD;
				DEFINE ANALYZER english TOKENIZERS blank, class;
				DEFINE USER app ON DATABASE PASSHASH "x";
				DEFINE API v1;
			"#
			.to_string(),
		}];

		let catalog = build_catalog_snapshot(&files);
		assert!(catalog.entities.contains(&EntityKey {
			kind: "table".to_string(),
			scope: None,
			name: "person".to_string()
		}));
		assert!(catalog.entities.contains(&EntityKey {
			kind: "field".to_string(),
			scope: Some("person".to_string()),
			name: "name".to_string()
		}));
		assert!(catalog.entities.contains(&EntityKey {
			kind: "api".to_string(),
			scope: None,
			name: "v1".to_string()
		}));
	}

	#[test]
	fn render_remove_sql_respects_api_support() {
		let entities = vec![
			EntityKey {
				kind: "field".to_string(),
				scope: Some("person".to_string()),
				name: "nickname".to_string(),
			},
			EntityKey {
				kind: "api".to_string(),
				scope: None,
				name: "v1".to_string(),
			},
		];

		let supported = render_remove_sql(&entities, true).expect("api should be supported");
		assert!(supported.iter().any(|line| line == "REMOVE API v1;"));

		let unsupported = render_remove_sql(&entities, false);
		assert!(unsupported.is_err());
	}

	#[test]
	fn snapshot_from_files_is_sorted_for_determinism() {
		let files = vec![
			SchemaFile {
				path: "database/schema/z.surql".to_string(),
				sql: "".to_string(),
				hash: "z".to_string(),
			},
			SchemaFile {
				path: "database/schema/a.surql".to_string(),
				sql: "".to_string(),
				hash: "a".to_string(),
			},
		];

		let snap = snapshot_from_files(&files);
		assert_eq!(snap.files[0].path, "database/schema/a.surql");
		assert_eq!(snap.files[1].path, "database/schema/z.surql");
	}
}
