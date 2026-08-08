//! Test-only proof fixture for the request-scoped batching foundation.
//!
//! `ADR-0006` requires a real consumer to prove the abstraction, and
//! THOTH-GQL-BATCH-01 section 3.1 requires that consumer to be **test-only**:
//! the foundation adopts **no production field**, modifies no production child
//! resolver, and modifies none of the 88 `MutationRoot` resolvers.
//!
//! Everything here is `#[cfg(test)]`. It defines its own GraphQL root, its own
//! object types and its own mutations over the **existing** `publisher` /
//! `imprint` tables, so the whole mechanism — real look-ahead, real set-based
//! SQL, real partitioning — is exercised with no public schema change.
//!
//! # Shapes proven
//!
//! ```text
//! direct path:    testPublishers -> imprints            (loader-backed child)
//! indirect path:  testImprints -> publisher -> imprints (loader-backed descendant)
//! ```
//!
//! The terminal field `imprints(limit: Int = 3)` is **argument-bearing with a
//! schema default**, so load-shape normalization is proven against real juniper
//! argument handling without adding an argument to any production field.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex, OnceLock,
};

use diesel::{
    connection::{set_default_instrumentation, Instrumentation, InstrumentationEvent},
    r2d2::ConnectionManager,
    ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl,
};
use juniper::{
    graphql_object, DefaultScalarValue, EmptySubscription, Executor, FieldResult,
    LookAheadSelection, LookAheadValue, RootNode, ScalarValue,
};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use crate::db::PgPool;
use crate::model::imprint::Imprint;
use crate::model::publisher::Publisher;
use crate::model::Crud;
use crate::schema::imprint;

use super::batching::{
    BatchLoader, BatchLookup, DispatchResult, LoadShapeKey, LoaderIdentity, StoredParentKey,
};
use super::prefetch::{prefetch, PrefetchTarget};
use super::scope::top_level_response_key;
use super::Context;

// ---------------------------------------------------------------------------
// The test-only loader
// ---------------------------------------------------------------------------

/// The schema default for the terminal field's `limit` argument.
///
/// Look-ahead does **not** apply schema defaults
/// (`ADR-0006` section 4.4.3), so the prefetch site must apply this explicitly.
/// The child resolver receives the default already applied by juniper. Both
/// paths therefore go through [`TestImprintLoader::shape`], which is what makes
/// an omitted argument and an explicitly supplied default resolve against the
/// same entry.
pub(crate) const DEFAULT_IMPRINT_LIMIT: i32 = 3;

/// The terminal loader's typed load shape.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct TestImprintShape {
    pub(crate) limit: i32,
}

/// Test-only proof loader: imprints partitioned by `publisher_id`.
pub(crate) struct TestImprintLoader;

impl TestImprintLoader {
    /// The **single** loader-owned shape constructor.
    ///
    /// Both the prefetch site and the child lookup call this, so the two cannot
    /// drift. Equality is semantic: two shapes built from equivalent argument
    /// sets compare equal regardless of how they were written.
    pub(crate) fn shape(limit: i32) -> TestImprintShape {
        TestImprintShape { limit }
    }

    /// Normalize a **terminal** look-ahead selection into the shape, applying
    /// the schema default where the argument was omitted.
    pub(crate) fn shape_from_selection(
        selection: &LookAheadSelection<'_, DefaultScalarValue>,
    ) -> TestImprintShape {
        let limit = selection
            .arguments()
            .find(|argument| argument.name() == "limit")
            .and_then(|argument| match argument.value() {
                LookAheadValue::Scalar(scalar) => scalar.as_int(),
                _ => None,
            })
            // Look-ahead reads only literal AST arguments; the default is
            // applied here.
            .unwrap_or(DEFAULT_IMPRINT_LIMIT);
        Self::shape(limit)
    }
}

impl BatchLoader for TestImprintLoader {
    type Key = Uuid;
    type Value = Imprint;
    type Shape = TestImprintShape;

    const IDENTITY: LoaderIdentity = LoaderIdentity::TestImprints;

    fn shape_key(shape: &Self::Shape) -> LoadShapeKey {
        LoadShapeKey::TestImprints { limit: shape.limit }
    }

    fn stored_key(key: &Self::Key) -> StoredParentKey {
        StoredParentKey::Uuid(*key)
    }

    fn key_for_value(value: &Self::Value) -> Self::Key {
        value.publisher_id
    }

    /// Exactly **one** set-based statement, using `.eq_any(..)`
    /// (`WHERE publisher_id = ANY($1)`), returning raw canonical `Imprint`
    /// rows rather than GraphQL objects.
    ///
    /// The ordering is `(publisher_id, imprint_name)`: the owning field's
    /// declared order (`imprint_name`) extended with the partition key, so the
    /// order is total and stable across the whole result set. Per-key
    /// truncation to `limit` is applied after grouping, which keeps the result
    /// identical, element for element and in order, to the direct per-parent
    /// query's `ORDER BY imprint_name LIMIT n`.
    fn load(db: &PgPool, keys: &[Self::Key], shape: &Self::Shape) -> ThothResult<Vec<Self::Value>> {
        let mut connection = db.get().map_err(ThothError::from)?;
        let rows: Vec<Imprint> = imprint::table
            .filter(imprint::publisher_id.eq_any(keys))
            .order((imprint::publisher_id.asc(), imprint::imprint_name.asc()))
            .load(&mut connection)
            .map_err(ThothError::from)?;

        // Truncate each parent's bucket to `limit`, preserving order. This is
        // still one statement; only the in-memory grouping is per key.
        let mut out: Vec<Imprint> = Vec::with_capacity(rows.len());
        let mut current: Option<Uuid> = None;
        let mut taken = 0usize;
        for row in rows {
            if current != Some(row.publisher_id) {
                current = Some(row.publisher_id);
                taken = 0;
            }
            if (taken as i32) < shape.limit {
                taken += 1;
                out.push(row);
            }
        }
        Ok(out)
    }
}

/// The direct per-parent query the terminal field falls back to.
///
/// This is the always-correct fallback of `ADR-0006` section 4.7, and the
/// reference the prefetched result is compared against.
fn imprints_direct(db: &PgPool, publisher_id: Uuid, limit: i32) -> ThothResult<Vec<Imprint>> {
    let mut connection = db.get().map_err(ThothError::from)?;
    imprint::table
        .filter(imprint::publisher_id.eq(publisher_id))
        .order(imprint::imprint_name.asc())
        .limit(limit.into())
        .load(&mut connection)
        .map_err(ThothError::from)
}

// ---------------------------------------------------------------------------
// Resolver-call and write counters, for the guard's zero-execution proof
// ---------------------------------------------------------------------------

/// Counts test-only **mutation resolver** entries.
///
/// Tests are serialized by the existing exclusive database test lock, so a
/// process-global counter is safe and is reset at the start of each measured
/// case.
static MUTATION_RESOLVER_CALLS: AtomicUsize = AtomicUsize::new(0);

/// Counts terminal child-resolver **fallback** statements (direct queries), so
/// "no terminal fallback statement on the covered path" is measurable
/// independently of the SQL harness.
static TERMINAL_FALLBACK_CALLS: AtomicUsize = AtomicUsize::new(0);

/// Counts intermediate (`publisher`) resolver statements separately from
/// terminal-loader statements, per `ADR-0006` sections 4.19.5 and 8.2 item 9.
static INTERMEDIATE_RESOLVER_CALLS: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn reset_counters() {
    MUTATION_RESOLVER_CALLS.store(0, Ordering::SeqCst);
    TERMINAL_FALLBACK_CALLS.store(0, Ordering::SeqCst);
    INTERMEDIATE_RESOLVER_CALLS.store(0, Ordering::SeqCst);
}

pub(crate) fn mutation_resolver_calls() -> usize {
    MUTATION_RESOLVER_CALLS.load(Ordering::SeqCst)
}

pub(crate) fn terminal_fallback_calls() -> usize {
    TERMINAL_FALLBACK_CALLS.load(Ordering::SeqCst)
}

pub(crate) fn intermediate_resolver_calls() -> usize {
    INTERMEDIATE_RESOLVER_CALLS.load(Ordering::SeqCst)
}

// ---------------------------------------------------------------------------
// SQL statement-count harness (`ADR-0006` section 8.1.1)
// ---------------------------------------------------------------------------

/// Captured `StartQuery` statements, in order.
static CAPTURED_SQL: OnceLock<Arc<Mutex<Vec<String>>>> = OnceLock::new();

/// Whether capture is currently armed. The hook is installed globally and
/// permanently, but only records while armed, so setup, fixture and migration
/// statements are excluded from the measured window.
static CAPTURE_ARMED: AtomicUsize = AtomicUsize::new(0);

fn captured() -> Arc<Mutex<Vec<String>>> {
    CAPTURED_SQL
        .get_or_init(|| Arc::new(Mutex::new(Vec::new())))
        .clone()
}

fn counting_instrumentation() -> Option<Box<dyn Instrumentation>> {
    Some(Box::new(|event: InstrumentationEvent<'_>| {
        if CAPTURE_ARMED.load(Ordering::SeqCst) == 0 {
            return;
        }
        if let InstrumentationEvent::StartQuery { query, .. } = event {
            if let Ok(mut sink) = captured().lock() {
                sink.push(query.to_string());
            }
        }
    }))
}

/// A dedicated measured pool, constructed **after** the instrumentation hook is
/// installed.
///
/// This is essential and is why the repository's ordinary
/// `OnceLock<Arc<PgPool>>` test pool must **not** be used: the hook applies
/// only to connections established *after* installation, and that singleton may
/// already hold established connections. Holding the test lock serializes
/// tests; it does not recreate connections.
pub(crate) struct SqlProbe {
    pub(crate) pool: Arc<PgPool>,
    previous_installed: bool,
}

impl SqlProbe {
    /// Install the hook and build a fresh pool behind it.
    ///
    /// Call **after** the database has been reset and fixtures created through
    /// the ordinary pool, so only the measured operation's statements are seen.
    pub(crate) fn install(database_url: &str) -> Self {
        set_default_instrumentation(counting_instrumentation)
            .expect("failed to install diesel instrumentation");

        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = diesel::r2d2::Pool::builder()
            .max_size(4)
            .build(manager)
            .expect("failed to build measured pool");

        Self {
            pool: Arc::new(pool),
            previous_installed: true,
        }
    }

    /// Begin recording. Clears anything captured earlier.
    pub(crate) fn start(&self) {
        captured().lock().expect("sql capture lock").clear();
        CAPTURE_ARMED.store(1, Ordering::SeqCst);
    }

    /// Stop recording and return the statements observed.
    pub(crate) fn stop(&self) -> Vec<String> {
        CAPTURE_ARMED.store(0, Ordering::SeqCst);
        captured().lock().expect("sql capture lock").clone()
    }

    /// Statements observed that touch the `imprint` table — the terminal
    /// loader's statements plus any terminal fallback.
    pub(crate) fn imprint_statements(&self) -> Vec<String> {
        self.stop()
            .into_iter()
            .filter(|sql| sql.contains("\"imprint\""))
            .collect()
    }
}

impl Drop for SqlProbe {
    /// Restore global instrumentation after the test, so a measured test cannot
    /// affect any other.
    fn drop(&mut self) {
        if self.previous_installed {
            CAPTURE_ARMED.store(0, Ordering::SeqCst);
            let _ = set_default_instrumentation(no_instrumentation);
        }
    }
}

fn no_instrumentation() -> Option<Box<dyn Instrumentation>> {
    None
}

// ---------------------------------------------------------------------------
// Test-only GraphQL types
// ---------------------------------------------------------------------------

/// Test-only wrapper over the canonical `Publisher` row.
#[derive(Clone, Debug)]
pub(crate) struct TestPublisherNode(pub(crate) Publisher);

#[graphql_object(Context = Context, Scalar = DefaultScalarValue, name = "TestPublisher")]
impl TestPublisherNode {
    fn publisher_id(&self) -> Uuid {
        self.0.publisher_id
    }

    fn publisher_name(&self) -> &str {
        &self.0.publisher_name
    }

    /// The loader-backed **terminal** field.
    ///
    /// It is argument-bearing, and `limit` carries a **schema default**, so
    /// default normalization is exercised against real juniper argument
    /// handling.
    ///
    /// Behaviour is fully determined by the store state
    /// (`ADR-0006` section 4.7): only `NotLoaded` performs the direct query.
    fn imprints(
        &self,
        context: &Context,
        executor: &Executor<'_, '_, Context>,
        #[graphql(default = 3)] limit: i32,
    ) -> FieldResult<Vec<TestImprintNode>> {
        let shape = TestImprintLoader::shape(limit);

        // Derive the scope from the same helper the prefetch site uses, so the
        // two necessarily agree. Failing closed here means `NotLoaded`.
        let lookup = match top_level_response_key(executor) {
            Some(scope) => context.batch_store.lookup::<TestImprintLoader>(
                &scope,
                &shape,
                &self.0.publisher_id,
            )?,
            None => BatchLookup::NotLoaded,
        };

        match lookup {
            // Prefetched — including a genuinely empty bucket — so no query.
            BatchLookup::Loaded(rows) => Ok(rows.into_iter().map(TestImprintNode).collect()),
            // Retained failure: return the derived error. No retry, and never
            // an empty result.
            BatchLookup::LoadFailed(error) => Err(error),
            // The always-correct direct fallback.
            //
            // The error conversion is `IntoFieldError`, **not** `?`/`Into`:
            // juniper carries a blanket `impl<T: Display, S> From<T> for
            // FieldError<S>` which would silently drop the `extensions.type`
            // discriminant. Using the canonical conversion is what makes the
            // direct path's error classification equal the prefetched path's
            // (`ADR-0006` section 4.9.3).
            BatchLookup::NotLoaded => {
                TERMINAL_FALLBACK_CALLS.fetch_add(1, Ordering::SeqCst);
                imprints_direct(&context.db, self.0.publisher_id, limit)
                    .map(|rows| rows.into_iter().map(TestImprintNode).collect())
                    .map_err(juniper::IntoFieldError::into_field_error)
            }
        }
    }
}

/// A terminal field whose **direct per-parent query** always fails.
///
/// This exists so the direct-path failure contract can be observed at the
/// *child* field — with the parent list resolving normally — and compared
/// against the prefetched-path failure contract
/// (`ADR-0006` section 4.9.3). Pointing the whole context at a failing pool
/// would instead fail the parent list resolver, which is a different error and
/// a different path.
#[graphql_object(Context = Context, Scalar = DefaultScalarValue, name = "TestPublisherFailing")]
impl TestPublisherFailingNode {
    fn publisher_id(&self) -> Uuid {
        self.0.publisher_id
    }

    fn imprints(&self, #[graphql(default = 3)] limit: i32) -> FieldResult<Vec<TestImprintNode>> {
        imprints_direct(
            &crate::model::tests::db::failing_pool(),
            self.0.publisher_id,
            limit,
        )
        .map(|rows| rows.into_iter().map(TestImprintNode).collect())
        .map_err(juniper::IntoFieldError::into_field_error)
    }
}

/// Wrapper selecting the always-failing direct terminal field.
#[derive(Clone, Debug)]
pub(crate) struct TestPublisherFailingNode(pub(crate) Publisher);

/// Test-only wrapper over the canonical `Imprint` row.
#[derive(Clone, Debug)]
pub(crate) struct TestImprintNode(pub(crate) Imprint);

#[graphql_object(Context = Context, Scalar = DefaultScalarValue, name = "TestImprint")]
impl TestImprintNode {
    fn imprint_id(&self) -> Uuid {
        self.0.imprint_id
    }

    fn imprint_name(&self) -> &str {
        &self.0.imprint_name
    }

    /// The **intermediate** object field of the descendant path.
    ///
    /// It queries once per imprint, exactly like the pre-existing legacy
    /// `Imprint.publisher` resolver it models. Descendant prefetch bounds the
    /// **terminal** loader; it deliberately does not remediate this
    /// (`ADR-0006` section 4.19.5), and this counter keeps the two evidence
    /// scopes separate.
    fn publisher(&self, context: &Context) -> FieldResult<TestPublisherNode> {
        INTERMEDIATE_RESOLVER_CALLS.fetch_add(1, Ordering::SeqCst);
        let publisher = Publisher::from_id(&context.db, &self.0.publisher_id)?;
        Ok(TestPublisherNode(publisher))
    }
}

/// Payload returned by the test-only mutations, so a mutation's own selection
/// can reach the loader-backed field.
#[derive(Clone, Debug)]
pub(crate) struct TestMutationPayload {
    pub(crate) publishers: Vec<TestPublisherNode>,
}

#[graphql_object(Context = Context, Scalar = DefaultScalarValue, name = "TestMutationPayload")]
impl TestMutationPayload {
    /// A nested fan-out inside a mutation payload: a **direct** prefetch site
    /// on a mutation path, using exactly the same mechanism as the query path.
    fn publishers(
        &self,
        context: &Context,
        executor: &Executor<'_, '_, Context>,
    ) -> FieldResult<Vec<TestPublisherNode>> {
        run_direct_prefetch(context, executor, &self.publishers);
        Ok(self.publishers.clone())
    }
}

// ---------------------------------------------------------------------------
// Prefetch sites
// ---------------------------------------------------------------------------

/// Selection path for the **direct** site: the terminal field is a direct child
/// of the resolved list items.
const DIRECT_PATH: &[&str] = &["imprints"];

/// Selection path for the **descendant** site: the terminal field sits beneath
/// the intermediate `publisher` object field.
const DESCENDANT_PATH: &[&str] = &["publisher", "imprints"];

/// Install the direct prefetch site over a resolved list of publishers.
fn run_direct_prefetch(
    context: &Context,
    executor: &Executor<'_, '_, Context>,
    items: &[TestPublisherNode],
) -> Vec<(TestImprintShape, DispatchResult)> {
    let target: PrefetchTarget<'_, TestPublisherNode, TestImprintLoader, DefaultScalarValue> =
        PrefetchTarget {
            path: DIRECT_PATH,
            terminal_shape: TestImprintLoader::shape_from_selection,
            // Identity projector: a direct site's degenerate case.
            project_key: |item| Some(item.0.publisher_id),
        };
    prefetch(executor, &context.batch_store, &context.db, items, &target)
}

/// Install the descendant prefetch site over a resolved list of imprints.
///
/// The key projector reads `Imprint.publisher_id`, a foreign key already
/// present on the already-resolved, already-authorized row. It derives nothing
/// from user input and bypasses no intermediate authorization decision
/// (`ADR-0006` section 4.19.4).
fn run_descendant_prefetch(
    context: &Context,
    executor: &Executor<'_, '_, Context>,
    items: &[TestImprintNode],
) -> Vec<(TestImprintShape, DispatchResult)> {
    let target: PrefetchTarget<'_, TestImprintNode, TestImprintLoader, DefaultScalarValue> =
        PrefetchTarget {
            path: DESCENDANT_PATH,
            terminal_shape: TestImprintLoader::shape_from_selection,
            project_key: |item| Some(item.0.publisher_id),
        };
    prefetch(executor, &context.batch_store, &context.db, items, &target)
}

// ---------------------------------------------------------------------------
// Test-only roots
// ---------------------------------------------------------------------------

pub(crate) struct TestQueryRoot;

#[graphql_object(Context = Context, Scalar = DefaultScalarValue, name = "TestQuery")]
impl TestQueryRoot {
    /// Direct prefetch site: `testPublishers -> imprints`.
    fn test_publishers(
        context: &Context,
        executor: &Executor<'_, '_, Context>,
    ) -> FieldResult<Vec<TestPublisherNode>> {
        let publishers = all_publishers(&context.db)?;
        run_direct_prefetch(context, executor, &publishers);
        Ok(publishers)
    }

    /// A second, distinct prefetch site covering the **same** loader, so
    /// multi-site coverage is proven by the foundation
    /// (`ADR-0006` section 4.18.3).
    fn test_publishers_alt(
        context: &Context,
        executor: &Executor<'_, '_, Context>,
    ) -> FieldResult<Vec<TestPublisherNode>> {
        let publishers = all_publishers(&context.db)?;
        run_direct_prefetch(context, executor, &publishers);
        Ok(publishers)
    }

    /// Descendant prefetch site: `testImprints -> publisher -> imprints`.
    fn test_imprints(
        context: &Context,
        executor: &Executor<'_, '_, Context>,
    ) -> FieldResult<Vec<TestImprintNode>> {
        let imprints = all_imprints(&context.db)?;
        run_descendant_prefetch(context, executor, &imprints);
        Ok(imprints)
    }

    /// A list with **no** prefetch site, so the `NotLoaded` fallback path is
    /// reachable in an operation that otherwise batches.
    fn test_publishers_unprefetched(context: &Context) -> FieldResult<Vec<TestPublisherNode>> {
        Ok(all_publishers(&context.db)?)
    }

    /// The parent list resolves normally; only the terminal field's **direct**
    /// query fails. Used for the error-contract comparison.
    fn test_publishers_failing_child(
        context: &Context,
    ) -> FieldResult<Vec<TestPublisherFailingNode>> {
        Ok(all_publishers(&context.db)?
            .into_iter()
            .map(|node| TestPublisherFailingNode(node.0))
            .collect())
    }
}

pub(crate) struct TestMutationRoot;

#[graphql_object(Context = Context, Scalar = DefaultScalarValue, name = "TestMutation")]
impl TestMutationRoot {
    /// Writes an imprint, then returns a payload whose selection can reach the
    /// loader-backed field — proving read-after-write **within one top-level
    /// mutation field**.
    fn add_imprint(
        context: &Context,
        publisher_id: Uuid,
        imprint_name: String,
    ) -> FieldResult<TestMutationPayload> {
        MUTATION_RESOLVER_CALLS.fetch_add(1, Ordering::SeqCst);
        insert_imprint(&context.db, publisher_id, &imprint_name)?;
        Ok(TestMutationPayload {
            publishers: all_publishers(&context.db)?,
        })
    }

    /// A **second distinct** top-level mutation field, so one operation can
    /// carry two top-level response keys without aliasing.
    fn add_imprint_alt(
        context: &Context,
        publisher_id: Uuid,
        imprint_name: String,
    ) -> FieldResult<TestMutationPayload> {
        MUTATION_RESOLVER_CALLS.fetch_add(1, Ordering::SeqCst);
        insert_imprint(&context.db, publisher_id, &imprint_name)?;
        Ok(TestMutationPayload {
            publishers: all_publishers(&context.db)?,
        })
    }

    /// A write-free mutation, for guard cases that must prove zero writes
    /// without needing a fixture publisher.
    fn touch(context: &Context) -> FieldResult<TestMutationPayload> {
        MUTATION_RESOLVER_CALLS.fetch_add(1, Ordering::SeqCst);
        Ok(TestMutationPayload {
            publishers: all_publishers(&context.db)?,
        })
    }
}

pub(crate) type TestSchema =
    RootNode<'static, TestQueryRoot, TestMutationRoot, EmptySubscription<Context>>;

pub(crate) fn test_schema() -> TestSchema {
    TestSchema::new(TestQueryRoot, TestMutationRoot, EmptySubscription::new())
}

// ---------------------------------------------------------------------------
// Plain data access used by the fixture
// ---------------------------------------------------------------------------

fn all_publishers(db: &PgPool) -> ThothResult<Vec<TestPublisherNode>> {
    use crate::schema::publisher;
    let mut connection = db.get().map_err(ThothError::from)?;
    let rows: Vec<Publisher> = publisher::table
        .order(publisher::publisher_name.asc())
        .load(&mut connection)
        .map_err(ThothError::from)?;
    Ok(rows.into_iter().map(TestPublisherNode).collect())
}

fn all_imprints(db: &PgPool) -> ThothResult<Vec<TestImprintNode>> {
    let mut connection = db.get().map_err(ThothError::from)?;
    let rows: Vec<Imprint> = imprint::table
        .order(imprint::imprint_name.asc())
        .load(&mut connection)
        .map_err(ThothError::from)?;
    Ok(rows.into_iter().map(TestImprintNode).collect())
}

fn insert_imprint(db: &PgPool, publisher_id: Uuid, imprint_name: &str) -> ThothResult<()> {
    use crate::model::imprint::NewImprint;
    let mut connection = db.get().map_err(ThothError::from)?;
    diesel::insert_into(imprint::table)
        .values(&NewImprint {
            publisher_id,
            imprint_name: imprint_name.to_string(),
            imprint_url: None,
            crossmark_doi: None,
            s3_bucket: None,
            cdn_domain: None,
            cloudfront_dist_id: None,
            default_currency: None,
            default_place: None,
            default_locale: None,
        })
        .execute(&mut connection)
        .map_err(ThothError::from)?;
    Ok(())
}

/// Convenience for tests that need the direct per-parent reference result.
pub(crate) fn direct_imprint_names(db: &PgPool, publisher_id: Uuid, limit: i32) -> Vec<String> {
    imprints_direct(db, publisher_id, limit)
        .expect("direct query failed")
        .into_iter()
        .map(|row| row.imprint_name)
        .collect()
}
