#![cfg(feature = "backend")]

use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::sync::{Arc, OnceLock};

use diesel::RunQueryDsl;
use fs2::FileExt;
use serde_json::Value;
use thoth_api::db::{init_pool, run_migrations, PgPool};
use thoth_api::graphql::{create_schema, Context, GraphQLRequest};
use zitadel::actix::introspection::IntrospectedUser;

static MIGRATIONS: OnceLock<Result<(), String>> = OnceLock::new();
static POOL: OnceLock<Arc<PgPool>> = OnceLock::new();

pub struct TestDbGuard {
    _file: std::fs::File,
}

pub fn test_lock() -> TestDbGuard {
    let mut path = env::temp_dir();
    path.push("thoth_test_db.lock");
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .unwrap_or_else(|err| panic!("Failed to open lock file {path:?}: {err}"));
    file.lock_exclusive()
        .unwrap_or_else(|err| panic!("Failed to lock test DB file {path:?}: {err}"));
    TestDbGuard { _file: file }
}

pub fn test_db_url() -> String {
    dotenv::dotenv().ok();
    env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set for backend tests")
}

pub fn db_pool() -> Arc<PgPool> {
    let url = test_db_url();
    let migrations = MIGRATIONS
        .get_or_init(|| run_migrations(&url).map_err(|err| err.to_string()))
        .clone();
    migrations.expect("Failed to run migrations for test DB");
    let pool = POOL.get_or_init(|| Arc::new(init_pool(&url)));
    pool.clone()
}

pub fn reset_db(pool: &PgPool) -> Result<(), diesel::result::Error> {
    let mut connection = pool.get().expect("Failed to get DB connection");
    let sql = r#"
DO $$
DECLARE
    tbls TEXT;
BEGIN
    SELECT string_agg(format('%I.%I', schemaname, tablename), ', ')
    INTO tbls
    FROM pg_tables
    WHERE schemaname = 'public'
      AND tablename != '__diesel_schema_migrations';

    IF tbls IS NOT NULL THEN
        EXECUTE 'TRUNCATE TABLE ' || tbls || ' RESTART IDENTITY CASCADE';
    END IF;
END $$;
"#;
    diesel::sql_query(sql).execute(&mut connection).map(|_| ())
}

pub async fn execute_graphql(
    pool: Arc<PgPool>,
    user: Option<IntrospectedUser>,
    query: &str,
    variables: Option<Value>,
) -> Value {
    let schema = create_schema();
    let ctx = Context::new(pool, user);

    let request_json = match variables {
        Some(vars) => serde_json::json!({ "query": query, "variables": vars }),
        None => serde_json::json!({ "query": query }),
    };

    let request: GraphQLRequest =
        serde_json::from_value(request_json).expect("Failed to build GraphQL request");
    let response = request.execute(&schema, &ctx).await;
    serde_json::to_value(response).expect("Failed to serialize GraphQL response")
}

fn build_user(
    user_id: &str,
    project_roles: Option<HashMap<String, HashMap<String, String>>>,
) -> IntrospectedUser {
    IntrospectedUser {
        user_id: user_id.to_string(),
        username: None,
        name: None,
        given_name: None,
        family_name: None,
        preferred_username: None,
        email: None,
        email_verified: None,
        locale: None,
        project_roles,
        metadata: None,
    }
}

pub fn user_with_roles(user_id: &str, role_scopes: &[(&str, &str)]) -> IntrospectedUser {
    let mut roles: HashMap<String, HashMap<String, String>> = HashMap::new();
    for (role, org_id) in role_scopes {
        roles
            .entry((*role).to_string())
            .or_default()
            .insert((*org_id).to_string(), "label".to_string());
    }

    build_user(user_id, if roles.is_empty() { None } else { Some(roles) })
}

pub fn superuser(user_id: &str) -> IntrospectedUser {
    let mut roles: HashMap<String, HashMap<String, String>> = HashMap::new();
    roles.insert("SUPERUSER".to_string(), HashMap::new());
    build_user(user_id, Some(roles))
}

pub fn assert_no_errors(response: &Value) {
    match response.get("errors") {
        None => {}
        Some(Value::Null) => {}
        Some(Value::Array(errors)) => {
            assert!(errors.is_empty(), "Expected no errors, got: {errors:?}");
        }
        Some(other) => panic!("Unexpected errors shape: {other:?}"),
    }
}

pub fn first_error_type(response: &Value) -> Option<&str> {
    response
        .get("errors")?
        .as_array()?
        .first()?
        .get("extensions")?
        .get("type")?
        .as_str()
}

pub fn first_error_message(response: &Value) -> Option<&str> {
    response
        .get("errors")?
        .as_array()?
        .first()?
        .get("message")?
        .as_str()
}

pub fn assert_no_access(response: &Value) {
    let Some(errors) = response.get("errors").and_then(|v| v.as_array()) else {
        panic!("Expected GraphQL errors, got: {response:?}");
    };
    if errors.is_empty() {
        panic!("Expected GraphQL errors, got: {response:?}");
    }

    let error_type = first_error_type(response);
    let message = first_error_message(response);

    if error_type == Some("NO_ACCESS")
        || message == Some("Unauthorized")
        || message == Some("Invalid credentials.")
    {
        return;
    }

    panic!(
        "Expected NO_ACCESS/Unauthorized error, got type={error_type:?} message={message:?} response={response:?}"
    );
}
