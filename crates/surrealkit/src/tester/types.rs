use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct TestOpts {
	pub suite: Option<String>,
	pub case: Option<String>,
	pub tags: Vec<String>,
	pub fail_fast: bool,
	pub parallel: usize,
	pub json_out: Option<PathBuf>,
	pub no_setup: bool,
	pub no_sync: bool,
	pub no_seed: bool,
	pub base_url: Option<String>,
	pub timeout_ms: Option<u64>,
	pub keep_db: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct GlobalTestConfig {
	#[serde(default)]
	pub defaults: GlobalDefaults,
	#[serde(default)]
	pub actors: BTreeMap<String, ActorSpec>,
	#[serde(default)]
	pub fixtures: Vec<FixtureSpec>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct GlobalDefaults {
	pub base_url: Option<String>,
	pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuiteSpec {
	pub name: Option<String>,
	#[serde(default)]
	pub tags: Vec<String>,
	#[serde(default)]
	pub actors: BTreeMap<String, ActorSpec>,
	#[serde(default)]
	pub fixtures: Vec<FixtureSpec>,
	#[serde(default)]
	pub cases: Vec<CaseSpec>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureSpec {
	pub name: Option<String>,
	pub actor: Option<String>,
	pub sql: Option<String>,
	pub file: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActorSpec {
	pub kind: ActorKind,
	pub username: Option<String>,
	pub username_env: Option<String>,
	pub password: Option<String>,
	pub password_env: Option<String>,
	pub namespace: Option<String>,
	pub namespace_env: Option<String>,
	pub database: Option<String>,
	pub database_env: Option<String>,
	pub access: Option<String>,
	pub access_env: Option<String>,
	pub params: Option<serde_json::Value>,
	pub token: Option<String>,
	pub token_env: Option<String>,
	#[serde(default)]
	pub headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
	Root,
	Namespace,
	Database,
	Record,
	Token,
	Headers,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseSpec {
	pub name: String,
	#[serde(default)]
	pub tags: Vec<String>,
	#[serde(flatten)]
	pub kind: CaseKind,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CaseKind {
	SqlExpect(SqlExpectCase),
	PermissionsMatrix(PermissionsMatrixCase),
	SchemaMetadata(SchemaMetadataCase),
	SchemaBehavior(SchemaBehaviorCase),
	ApiRequest(ApiRequestCase),
}

impl CaseKind {
	pub fn label(&self) -> &'static str {
		match self {
			Self::SqlExpect(_) => "sql_expect",
			Self::PermissionsMatrix(_) => "permissions_matrix",
			Self::SchemaMetadata(_) => "schema_metadata",
			Self::SchemaBehavior(_) => "schema_behavior",
			Self::ApiRequest(_) => "api_request",
		}
	}
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqlExpectCase {
	pub actor: Option<String>,
	pub sql: String,
	#[serde(default = "default_true")]
	pub allow: bool,
	pub error_contains: Option<String>,
	pub error_code: Option<String>,
	#[serde(default)]
	pub assertions: Vec<JsonAssertionSpec>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PermissionsMatrixCase {
	pub actor: Option<String>,
	pub table: String,
	pub record_id: Option<String>,
	#[serde(default)]
	pub rules: Vec<PermissionRuleSpec>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PermissionRuleSpec {
	pub action: PermissionAction,
	#[serde(default = "default_true")]
	pub allow: bool,
	pub sql: Option<String>,
	pub error_contains: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionAction {
	Create,
	Select,
	Update,
	Delete,
	Query,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaMetadataCase {
	pub actor: Option<String>,
	pub table: Option<String>,
	pub sql: Option<String>,
	#[serde(default)]
	pub contains: Vec<String>,
	#[serde(default)]
	pub assertions: Vec<JsonAssertionSpec>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaBehaviorCase {
	pub actor: Option<String>,
	#[serde(default)]
	pub setup_sql: Vec<String>,
	pub action_sql: String,
	#[serde(default = "default_true")]
	pub expect_success: bool,
	pub expect_error_contains: Option<String>,
	pub verify_sql: Option<String>,
	#[serde(default)]
	pub assertions: Vec<JsonAssertionSpec>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiRequestCase {
	pub actor: Option<String>,
	#[serde(default = "default_get")]
	pub method: String,
	pub path: String,
	pub expected_status: u16,
	#[serde(default)]
	pub headers: BTreeMap<String, String>,
	pub body: Option<serde_json::Value>,
	pub timeout_ms: Option<u64>,
	#[serde(default)]
	pub body_assertions: Vec<JsonAssertionSpec>,
	#[serde(default)]
	pub header_assertions: Vec<HeaderAssertionSpec>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonAssertionSpec {
	pub path: String,
	pub exists: Option<bool>,
	pub equals: Option<serde_json::Value>,
	pub contains: Option<String>,
	pub regex: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeaderAssertionSpec {
	pub name: String,
	pub exists: Option<bool>,
	pub equals: Option<String>,
	pub contains: Option<String>,
	pub regex: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoadedSpecs {
	pub global: GlobalTestConfig,
	pub suites: Vec<LoadedSuite>,
}

#[derive(Debug, Clone)]
pub struct LoadedSuite {
	pub path: PathBuf,
	pub spec: SuiteSpec,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunReport {
	pub started_at: String,
	pub finished_at: String,
	pub duration_ms: u128,
	pub suites_total: usize,
	pub suites_failed: usize,
	pub cases_total: usize,
	pub cases_passed: usize,
	pub cases_failed: usize,
	pub suites: Vec<SuiteReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SuiteReport {
	pub suite_file: String,
	pub suite_name: String,
	pub namespace: String,
	pub database: String,
	pub duration_ms: u128,
	pub cases_total: usize,
	pub cases_passed: usize,
	pub cases_failed: usize,
	pub cases: Vec<CaseReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CaseReport {
	pub name: String,
	pub kind: String,
	pub duration_ms: u128,
	pub passed: bool,
	pub message: Option<String>,
	pub assertions: Vec<AssertionReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssertionReport {
	pub name: String,
	pub passed: bool,
	pub message: String,
}

#[derive(Debug, Clone)]
pub struct FilterInput {
	pub suite_pattern: Option<String>,
	pub case_pattern: Option<String>,
	pub tags: Vec<String>,
}

pub fn default_true() -> bool {
	true
}

fn default_get() -> String {
	"GET".to_string()
}
