use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use dataloader::non_cached::Loader;
use dataloader::BatchFn;
use diesel::{
    connection::{set_default_instrumentation, Instrumentation, InstrumentationEvent},
    r2d2::ConnectionManager,
    ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl,
};
use juniper::{
    graphql_object, graphql_value, DefaultScalarValue, EmptySubscription, FieldError, FieldResult,
    IntoFieldError, RootNode,
};
use uuid::Uuid;

use super::{configured_loader, FieldErrorConvention, SharedBatchError};
use crate::db::PgPool;
use crate::graphql::Context;
use crate::model::tests::db as test_db;
use crate::schema::imprint;
use thoth_errors::ThothError;

#[derive(Default)]
pub(crate) struct BatchStats {
    dispatches: AtomicUsize,
    batches: Mutex<Vec<usize>>,
}

impl BatchStats {
    pub(crate) fn record<K>(&self, keys: &[K]) {
        self.dispatches.fetch_add(1, Ordering::SeqCst);
        self.batches
            .lock()
            .expect("batch stats lock")
            .push(keys.len());
    }

    pub(crate) fn dispatch_count(&self) -> usize {
        self.dispatches.load(Ordering::SeqCst)
    }

    pub(crate) fn batch_sizes(&self) -> Vec<usize> {
        self.batches.lock().expect("batch stats lock").clone()
    }
}

pub(crate) type MemSource = Arc<Mutex<HashMap<i32, Vec<String>>>>;
pub(crate) type MemValue = Result<Vec<String>, SharedBatchError>;

pub(crate) struct MemBatcher {
    pub(crate) source: MemSource,
    pub(crate) stats: Arc<BatchStats>,
    pub(crate) marker: &'static str,
    pub(crate) fail: bool,
    pub(crate) omit_all: bool,
}

impl BatchFn<i32, MemValue> for MemBatcher {
    async fn load(&mut self, keys: &[i32]) -> HashMap<i32, MemValue> {
        self.stats.record(keys);
        if self.omit_all {
            return HashMap::new();
        }
        if self.fail {
            let error = SharedBatchError::from_message("simulated backend failure");
            return keys.iter().map(|key| (*key, Err(error.clone()))).collect();
        }
        let source = self.source.lock().expect("memory source lock");
        keys.iter()
            .map(|key| {
                let values = source
                    .get(key)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|value| {
                        if self.marker.is_empty() {
                            value
                        } else {
                            format!("{}:{value}", self.marker)
                        }
                    })
                    .collect();
                (*key, Ok(values))
            })
            .collect()
    }
}

pub(crate) type MemLoader = Loader<i32, MemValue, MemBatcher>;
pub(crate) type DbValue = Result<Vec<String>, SharedBatchError>;

pub(crate) struct DbBatcher {
    pool: Arc<PgPool>,
    stats: Arc<BatchStats>,
    convention: FieldErrorConvention,
}

impl BatchFn<Uuid, DbValue> for DbBatcher {
    async fn load(&mut self, keys: &[Uuid]) -> HashMap<Uuid, DbValue> {
        self.stats.record(keys);
        let pool = Arc::clone(&self.pool);
        let key_vec = keys.to_vec();
        let result =
            tokio::task::spawn_blocking(move || -> Result<Vec<(Uuid, String)>, ThothError> {
                let mut connection = pool.get().map_err(ThothError::from)?;
                imprint::table
                    .filter(imprint::publisher_id.eq_any(&key_vec))
                    .order((imprint::publisher_id.asc(), imprint::imprint_name.asc()))
                    .select((imprint::publisher_id, imprint::imprint_name))
                    .load::<(Uuid, String)>(&mut connection)
                    .map_err(ThothError::from)
            })
            .await;

        let mut output: HashMap<Uuid, DbValue> =
            keys.iter().map(|key| (*key, Ok(Vec::new()))).collect();
        match result {
            Ok(Ok(rows)) => {
                for (publisher_id, imprint_name) in rows {
                    if let Some(Ok(values)) = output.get_mut(&publisher_id) {
                        values.push(imprint_name);
                    }
                }
            }
            Ok(Err(error)) => {
                let error = SharedBatchError::from_thoth(error, self.convention);
                for key in keys {
                    output.insert(*key, Err(error.clone()));
                }
            }
            Err(join_error) => {
                let error = SharedBatchError::from_thoth(
                    ThothError::InternalError(join_error.to_string()),
                    self.convention,
                );
                for key in keys {
                    output.insert(*key, Err(error.clone()));
                }
            }
        }
        output
    }
}

pub(crate) type DbLoader = Loader<Uuid, DbValue, DbBatcher>;

pub(crate) struct FixtureLoaders {
    pub(crate) mem: MemLoader,
    pub(crate) meta: MemLoader,
    pub(crate) db: Option<DbLoader>,
    pub(crate) mem_stats: Arc<BatchStats>,
    pub(crate) meta_stats: Arc<BatchStats>,
    pub(crate) db_stats: Arc<BatchStats>,
    pub(crate) mem_source: MemSource,
    pub(crate) load_calls: Arc<AtomicUsize>,
    pub(crate) write_count: Arc<AtomicUsize>,
    pub(crate) direct_db: Option<Arc<PgPool>>,
}

impl FixtureLoaders {
    pub(crate) fn in_memory(source: MemSource, marker: &'static str) -> Self {
        Self::build(source, marker, false, false)
    }

    pub(crate) fn in_memory_omitting(source: MemSource) -> Self {
        Self::build(source, "", true, false)
    }

    pub(crate) fn in_memory_failing(source: MemSource) -> Self {
        Self::build(source, "", false, true)
    }

    fn build(source: MemSource, marker: &'static str, omit_all: bool, fail: bool) -> Self {
        let mem_stats = Arc::new(BatchStats::default());
        let meta_stats = Arc::new(BatchStats::default());
        let mem = configured_loader(MemBatcher {
            source: Arc::clone(&source),
            stats: Arc::clone(&mem_stats),
            marker,
            fail,
            omit_all,
        });
        let meta = configured_loader(MemBatcher {
            source: Arc::clone(&source),
            stats: Arc::clone(&meta_stats),
            marker: "meta",
            fail: false,
            omit_all: false,
        });
        Self {
            mem,
            meta,
            db: None,
            mem_stats,
            meta_stats,
            db_stats: Arc::new(BatchStats::default()),
            mem_source: source,
            load_calls: Arc::new(AtomicUsize::new(0)),
            write_count: Arc::new(AtomicUsize::new(0)),
            direct_db: None,
        }
    }

    pub(crate) fn with_db(mut self, pool: Arc<PgPool>, convention: FieldErrorConvention) -> Self {
        let stats = Arc::new(BatchStats::default());
        self.db = Some(configured_loader(DbBatcher {
            pool,
            stats: Arc::clone(&stats),
            convention,
        }));
        self.db_stats = stats;
        self
    }

    pub(crate) fn with_direct_db(mut self, pool: Arc<PgPool>) -> Self {
        self.direct_db = Some(pool);
        self
    }
}

pub(crate) fn empty_source() -> MemSource {
    Arc::new(Mutex::new(HashMap::new()))
}

pub(crate) fn source_from(entries: &[(i32, &[&str])]) -> MemSource {
    Arc::new(Mutex::new(
        entries
            .iter()
            .map(|(key, children)| {
                (
                    *key,
                    children.iter().map(|child| child.to_string()).collect(),
                )
            })
            .collect(),
    ))
}

pub(crate) fn fixture_context(pool: Arc<PgPool>, loaders: FixtureLoaders) -> Context {
    let mut context = test_db::test_context_anonymous(pool);
    context.loaders.fixture = Some(loaders);
    context
}

fn loaders(context: &Context) -> FieldResult<&FixtureLoaders> {
    context
        .loaders
        .fixture
        .as_ref()
        .ok_or_else(|| FieldError::new("test loaders not installed", graphql_value!(None)))
}

fn unpack(outcome: Result<MemValue, std::io::Error>) -> FieldResult<Vec<String>> {
    match outcome {
        Ok(Ok(values)) => Ok(values),
        Ok(Err(error)) => Err(error.to_field_error()),
        Err(missing) => Err(FieldError::new(
            format!("loader returned no entry: {missing}"),
            graphql_value!(None),
        )),
    }
}

fn unpack_db(outcome: Result<DbValue, std::io::Error>) -> FieldResult<Vec<String>> {
    match outcome {
        Ok(Ok(values)) => Ok(values),
        Ok(Err(error)) => Err(error.to_field_error()),
        Err(missing) => Err(FieldError::new(
            format!("loader returned no entry: {missing}"),
            graphql_value!(None),
        )),
    }
}

fn direct_imprint_names_thoth(
    pool: &PgPool,
    publisher_id: Uuid,
) -> Result<Vec<String>, ThothError> {
    let mut connection = pool.get().map_err(ThothError::from)?;
    imprint::table
        .filter(imprint::publisher_id.eq(publisher_id))
        .order(imprint::imprint_name.asc())
        .select(imprint::imprint_name)
        .load::<String>(&mut connection)
        .map_err(ThothError::from)
}

pub(crate) struct TestQuery;
pub(crate) struct TestMutation;
pub(crate) struct TestParent {
    id: i32,
}
pub(crate) struct TestDbParent {
    publisher_id: Uuid,
}
pub(crate) struct TestPayload {
    id: i32,
    before: Vec<String>,
}

pub(crate) type TestSchema = RootNode<'static, TestQuery, TestMutation, EmptySubscription<Context>>;

pub(crate) fn schema() -> TestSchema {
    TestSchema::new(TestQuery, TestMutation, EmptySubscription::new())
}

#[graphql_object(Context = Context, Scalar = DefaultScalarValue, name = "DataLoaderTestQuery")]
impl TestQuery {
    fn parents(count: i32) -> Vec<TestParent> {
        (1..=count).map(|id| TestParent { id }).collect()
    }

    fn db_parents(context: &Context) -> FieldResult<Vec<TestDbParent>> {
        use crate::schema::publisher;
        let mut connection = context
            .db
            .get()
            .map_err(|error| FieldError::new(error.to_string(), graphql_value!(None)))?;
        let ids = publisher::table
            .order(publisher::publisher_name.asc())
            .select(publisher::publisher_id)
            .load::<Uuid>(&mut connection)
            .map_err(|error| FieldError::new(error.to_string(), graphql_value!(None)))?;
        Ok(ids
            .into_iter()
            .map(|publisher_id| TestDbParent { publisher_id })
            .collect())
    }
}

#[graphql_object(Context = Context, Scalar = DefaultScalarValue, name = "DataLoaderTestParent")]
impl TestParent {
    fn parent_id(&self) -> i32 {
        self.id
    }

    async fn async_probe(&self) -> String {
        tokio::task::yield_now().await;
        format!("async-{}", self.id)
    }

    async fn children(&self, context: &Context) -> FieldResult<Vec<String>> {
        let bundle = loaders(context)?;
        bundle.load_calls.fetch_add(1, Ordering::SeqCst);
        unpack(bundle.mem.try_load(self.id).await)
    }

    async fn children_after_yield(&self, context: &Context) -> FieldResult<Vec<String>> {
        let bundle = loaders(context)?;
        tokio::task::yield_now().await;
        bundle.load_calls.fetch_add(1, Ordering::SeqCst);
        unpack(bundle.mem.try_load(self.id).await)
    }

    /// A resolver that deliberately violates loader-first for half its
    /// cohort: odd-id resolvers perform unrelated awaited work until the
    /// loader's **first dispatch has already happened**, then register their
    /// keys. This makes dispatch fragmentation deterministic — the delayed
    /// cohort can only land in a later batch — where a wall-clock sleep
    /// proved scheduler/host dependent (a loaded CI runner coalesced a 1 ms
    /// delayed cohort into a single dispatch).
    async fn children_delayed(&self, context: &Context) -> FieldResult<Vec<String>> {
        let bundle = loaders(context)?;
        if self.id % 2 == 1 {
            while bundle.mem_stats.dispatch_count() == 0 {
                tokio::task::yield_now().await;
            }
        }
        bundle.load_calls.fetch_add(1, Ordering::SeqCst);
        unpack(bundle.mem.try_load(self.id).await)
    }

    async fn children_chained(&self, context: &Context) -> FieldResult<Vec<String>> {
        let bundle = loaders(context)?;
        let _ = unpack(bundle.meta.try_load(self.id).await)?;
        bundle.load_calls.fetch_add(1, Ordering::SeqCst);
        unpack(bundle.mem.try_load(self.id).await)
    }
}

#[graphql_object(Context = Context, Scalar = DefaultScalarValue, name = "DataLoaderTestDbParent")]
impl TestDbParent {
    fn publisher_id(&self) -> Uuid {
        self.publisher_id
    }

    async fn imprints(&self, context: &Context) -> FieldResult<Vec<String>> {
        let bundle = loaders(context)?;
        if let Some(direct_pool) = bundle.direct_db.as_ref() {
            return direct_imprint_names_thoth(direct_pool, self.publisher_id).map_err(Into::into);
        }
        let loader = bundle
            .db
            .as_ref()
            .ok_or_else(|| FieldError::new("db loader not installed", graphql_value!(None)))?;
        unpack_db(loader.try_load(self.publisher_id).await)
    }

    async fn imprints_explicit(&self, context: &Context) -> FieldResult<Vec<String>> {
        let bundle = loaders(context)?;
        if let Some(direct_pool) = bundle.direct_db.as_ref() {
            return direct_imprint_names_thoth(direct_pool, self.publisher_id)
                .map_err(IntoFieldError::into_field_error);
        }
        let loader = bundle
            .db
            .as_ref()
            .ok_or_else(|| FieldError::new("db loader not installed", graphql_value!(None)))?;
        unpack_db(loader.try_load(self.publisher_id).await)
    }
}

#[graphql_object(Context = Context, Scalar = DefaultScalarValue, name = "DataLoaderTestMutation")]
impl TestMutation {
    async fn rewrite(context: &Context, id: i32, new_child: String) -> FieldResult<TestPayload> {
        let bundle = loaders(context)?;
        let before = unpack(bundle.mem.try_load(id).await)?;
        bundle
            .mem_source
            .lock()
            .expect("memory source lock")
            .insert(id, vec![new_child]);
        bundle.write_count.fetch_add(1, Ordering::SeqCst);
        Ok(TestPayload { id, before })
    }
}

#[graphql_object(Context = Context, Scalar = DefaultScalarValue, name = "DataLoaderTestPayload")]
impl TestPayload {
    fn id(&self) -> i32 {
        self.id
    }

    fn before(&self) -> Vec<String> {
        self.before.clone()
    }

    async fn after(&self, context: &Context) -> FieldResult<Vec<String>> {
        unpack(loaders(context)?.mem.try_load(self.id).await)
    }
}

static CAPTURED_SQL: OnceLock<Arc<Mutex<Vec<String>>>> = OnceLock::new();
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

fn no_instrumentation() -> Option<Box<dyn Instrumentation>> {
    None
}

pub(crate) struct SqlProbe {
    pub(crate) pool: Arc<PgPool>,
    installed: bool,
}

impl SqlProbe {
    pub(crate) fn install(database_url: &str) -> Self {
        set_default_instrumentation(counting_instrumentation)
            .expect("failed to install Diesel instrumentation");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = diesel::r2d2::Pool::builder()
            .max_size(4)
            .build(manager)
            .expect("failed to build measured pool");
        Self {
            pool: Arc::new(pool),
            installed: true,
        }
    }

    pub(crate) fn start(&self) {
        captured().lock().expect("SQL capture lock").clear();
        CAPTURE_ARMED.store(1, Ordering::SeqCst);
    }

    fn stop(&self) -> Vec<String> {
        CAPTURE_ARMED.store(0, Ordering::SeqCst);
        captured().lock().expect("SQL capture lock").clone()
    }

    pub(crate) fn imprint_statements(&self) -> Vec<String> {
        self.stop()
            .into_iter()
            .filter(|sql| sql.contains("\"imprint\""))
            .collect()
    }

    /// Every statement captured since [`Self::start`], unfiltered.
    ///
    /// Adopting fields classify their own target statements: a field whose
    /// root query also touches the child table cannot use a bare table-name
    /// filter.
    pub(crate) fn captured_statements(&self) -> Vec<String> {
        self.stop()
    }
}

impl Drop for SqlProbe {
    fn drop(&mut self) {
        if self.installed {
            CAPTURE_ARMED.store(0, Ordering::SeqCst);
            let _ = set_default_instrumentation(no_instrumentation);
        }
    }
}
