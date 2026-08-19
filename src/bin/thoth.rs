mod arguments;
mod commands;

lazy_static::lazy_static! {
    static ref THOTH: clap::Command = clap::Command::new(env!("CARGO_PKG_NAME"))
        .version(clap::crate_version!())
        .author(clap::crate_authors!("\n"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(commands::MIGRATE.clone())
        .subcommand(commands::start::COMMAND.clone())
        .subcommand(commands::INIT.clone())
        .subcommand(commands::cache::COMMAND.clone())
        .subcommand(commands::zitadel::COMMAND.clone())
        .subcommand(commands::publisher_services::COMMAND.clone());
}

fn main() -> thoth::errors::ThothResult<()> {
    // load environment variables from `.env`
    dotenv::dotenv().ok();

    match THOTH.clone().get_matches().subcommand() {
        Some(("start", start_arguments)) => match start_arguments.subcommand() {
            Some(("graphql-api", arguments)) => commands::start::graphql_api(arguments),
            Some(("export-api", arguments)) => commands::start::export_api(arguments),
            _ => unreachable!(),
        },
        Some(("migrate", arguments)) => commands::migrate(arguments),
        Some(("init", arguments)) => commands::run_init(
            || commands::run_migrations(arguments),
            || commands::start::graphql_api(arguments),
        ),
        Some(("cache", arguments)) => match arguments.subcommand() {
            Some(("delete", _)) => commands::cache::delete(arguments),
            _ => unreachable!(),
        },
        Some(("zitadel", arguments)) => match arguments.subcommand() {
            Some(("setup", _)) => commands::zitadel::setup(arguments),
            _ => unreachable!(),
        },
        Some(("publisher-services", arguments)) => match arguments.subcommand() {
            Some(("migration-backfill", arguments)) => match arguments.subcommand() {
                Some(("dry-run", arguments)) => commands::publisher_services::dry_run(arguments),
                Some(("apply", arguments)) => commands::publisher_services::apply(arguments),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

#[test]
fn test_cli() {
    THOTH.clone().debug_assert();
}

/// One process-wide lock for every test that mutates the environment.
///
/// Two control-argument test modules drive environment variables that the
/// **whole** `THOTH` command tree observes when it is parsed, so a per-module
/// lock is not enough: one module's temporary invalid value would otherwise make
/// the other module's parse of the same tree fail. Sharing one lock is what
/// keeps both deterministic.
#[cfg(test)]
mod test_env_lock {
    use std::sync::{Mutex, MutexGuard};

    pub(super) static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Hold the shared lock without changing anything.
    ///
    /// A test that only *parses* the command tree still has to hold it: another
    /// test's temporary invalid value for any environment-bound argument would
    /// otherwise make this parse fail on an argument it never mentioned.
    pub(super) fn hold() -> MutexGuard<'static, ()> {
        let guard = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        initialise_command_tree();
        guard
    }

    /// Force the shared `lazy_static` command tree to build while the
    /// environment is still ambient.
    ///
    /// `clap`'s `Arg::env` captures a variable's value when the `Arg` is
    /// **constructed**, and `THOTH` is constructed once per process. A command
    /// tree first built while some test held a deliberately invalid value would
    /// keep that value for the life of the process, and every later parse would
    /// then fail on an argument it never mentioned. Building it here, under the
    /// lock and before any test mutates the environment, removes the ordering
    /// dependency entirely.
    pub(super) fn initialise_command_tree() {
        let _ = super::THOTH.get_name();
    }
}

/// `THOTH-GQL-OPS-02` — the mutation-guard mode-control path.
///
/// Every row of the specification's compatibility matrix is pinned by its own
/// test, on the **production-applicable** command path (`init`, the image's
/// default command). The whole module is compiled and run in both profiles:
/// `cargo test` exercises the debug profile and `cargo test --release` the
/// release profile. That matters, because the defect this task fixes was
/// profile-dependent — in pinned `clap_builder` 4.6.0 `ArgMatches::verify_arg`
/// rejects an unregistered argument only under `cfg(debug_assertions)`, so
/// reading an unregistered `mutation-guard-mode` panicked in a debug build and
/// silently resolved the fallback `OFF` in a release build.
///
/// Two access surfaces are exercised, because `clap`'s `Arg::env()` captures the
/// environment value when the `Arg` is **constructed**, and the commands are
/// `lazy_static` — so a running process reads the variable exactly once, which
/// is correct for the binary but means in-process mutation cannot vary it
/// across tests:
///
/// ```text
/// the real INIT command   driven by argv, and asserted to register the
///                         argument and to declare the OFF default
/// a freshly built command driven by THOTH_GRAPHQL_MUTATION_GUARD_MODE,
///                         using the same arguments::mutation_guard_mode()
/// ```
///
/// The environment path is additionally verified end-to-end against real
/// release and debug binaries in the manual verification, which is how
/// production consumes it.
#[cfg(test)]
mod mutation_guard_mode_on_init {
    use super::*;
    use clap::{ArgMatches, Command};
    use std::cell::{Cell, RefCell};
    use std::env::{remove_var, set_var, var_os};
    use std::ffi::OsString;
    use std::sync::MutexGuard;
    use thoth::api::graphql::MutationGuardMode;

    const MODE_ENV: &str = "THOTH_GRAPHQL_MUTATION_GUARD_MODE";

    /// The environment is process-global, and more than one control-argument
    /// module drives it. Serialise every test that mutates it, through the one
    /// shared lock.
    use super::test_env_lock::ENV_LOCK;

    /// Sets `THOTH_GRAPHQL_MUTATION_GUARD_MODE` for the life of the guard and
    /// restores the previous value — or its absence — on drop.
    struct ModeEnv {
        _lock: MutexGuard<'static, ()>,
        previous: Option<OsString>,
    }

    impl ModeEnv {
        fn set(value: Option<&str>) -> Self {
            let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            super::test_env_lock::initialise_command_tree();
            let previous = var_os(MODE_ENV);
            match value {
                Some(value) => set_var(MODE_ENV, value),
                None => remove_var(MODE_ENV),
            }
            Self {
                _lock: lock,
                previous,
            }
        }
    }

    impl Drop for ModeEnv {
        fn drop(&mut self) {
            match self.previous.take() {
                Some(previous) => set_var(MODE_ENV, previous),
                None => remove_var(MODE_ENV),
            }
        }
    }

    // --- surface 1: the real `init` command, driven by argv -----------------

    /// Parses `thoth init` through the real command tree and returns the `init`
    /// sub-matches.
    fn parse_init(extra: &[&str]) -> Result<ArgMatches, clap::Error> {
        let mut argv = vec!["thoth", "init"];
        argv.extend_from_slice(extra);
        let matches = THOTH.clone().try_get_matches_from(argv)?;
        Ok(matches
            .subcommand_matches("init")
            .expect("`init` sub-matches")
            .clone())
    }

    /// The effective mode on the production-applicable path, resolved through
    /// the **same** accessor the running server uses. In a debug build this
    /// panics if `init` does not register the argument, which is precisely the
    /// regression being guarded against.
    fn mode_on_init(extra: &[&str]) -> MutationGuardMode {
        commands::start::mutation_guard_mode(&parse_init(extra).expect("`init` should parse"))
            .expect("mode should resolve")
    }

    // --- surface 2: a fresh command, driven by the environment variable -----

    /// Builds a command carrying the very same argument definition `init`
    /// registers, constructed *now* so it observes the current environment.
    fn parse_fresh() -> Result<ArgMatches, clap::Error> {
        Command::new("init")
            .no_binary_name(true)
            .arg(arguments::mutation_guard_mode())
            .try_get_matches_from::<_, &str>([])
    }

    fn mode_from_env() -> MutationGuardMode {
        commands::start::mutation_guard_mode(&parse_fresh().expect("should parse"))
            .expect("mode should resolve")
    }

    // --- compatibility matrix: one test per row, both surfaces --------------

    #[test]
    fn unset_yields_off() {
        let _env = ModeEnv::set(None);
        assert_eq!(mode_from_env(), MutationGuardMode::Off);
    }

    #[test]
    fn off_yields_off() {
        let _env = ModeEnv::set(Some("OFF"));
        assert_eq!(mode_from_env(), MutationGuardMode::Off);
        assert_eq!(
            mode_on_init(&["--mutation-guard-mode", "OFF"]),
            MutationGuardMode::Off
        );
    }

    #[test]
    fn observe_yields_observe_and_never_off() {
        let _env = ModeEnv::set(Some("OBSERVE"));
        for mode in [
            mode_from_env(),
            mode_on_init(&["--mutation-guard-mode", "OBSERVE"]),
        ] {
            assert_eq!(mode, MutationGuardMode::Observe);
            assert_ne!(
                mode,
                MutationGuardMode::Off,
                "OBSERVE must never be silently ignored on the init path"
            );
        }
    }

    #[test]
    fn enforce_yields_enforce_and_never_off() {
        let _env = ModeEnv::set(Some("ENFORCE"));
        for mode in [
            mode_from_env(),
            mode_on_init(&["--mutation-guard-mode", "ENFORCE"]),
        ] {
            assert_eq!(mode, MutationGuardMode::Enforce);
            assert_ne!(
                mode,
                MutationGuardMode::Off,
                "ENFORCE must never be silently ignored on the init path"
            );
        }
    }

    #[test]
    fn invalid_value_fails_startup_and_is_never_coerced_to_off() {
        let _env = ModeEnv::set(Some("SOMETHING_ELSE"));

        let from_env = parse_fresh().expect_err("an invalid environment value must fail parsing");
        assert_eq!(from_env.kind(), clap::error::ErrorKind::InvalidValue);

        let from_argv = parse_init(&["--mutation-guard-mode", "SOMETHING_ELSE"])
            .expect_err("an invalid command-line value must fail parsing");
        assert_eq!(from_argv.kind(), clap::error::ErrorKind::InvalidValue);

        // Parsing fails, so no mode is produced at all — least of all a
        // silently coerced `Off`.
        assert!(parse_fresh().is_err());
        assert!(parse_init(&["--mutation-guard-mode", "SOMETHING_ELSE"]).is_err());
    }

    // --- direct regression tests for the exact defect -----------------------

    #[test]
    fn init_registers_the_mutation_guard_mode_argument() {
        assert!(
            commands::INIT
                .get_arguments()
                .any(|arg| arg.get_id() == "mutation-guard-mode"),
            "init must register mutation-guard-mode: dispatching into the \
             graphql-api handler without it silently ignored the configured mode"
        );
    }

    #[test]
    fn init_binds_the_documented_environment_variable_and_off_default() {
        let arg = commands::INIT
            .get_arguments()
            .find(|arg| arg.get_id() == "mutation-guard-mode")
            .expect("mutation-guard-mode on init");
        assert_eq!(arg.get_env().and_then(|env| env.to_str()), Some(MODE_ENV));
        assert_eq!(
            arg.get_default_values(),
            ["OFF"],
            "the declared default must remain OFF, so an absent value yields Off"
        );
    }

    #[test]
    fn resolving_the_mode_on_init_does_not_panic_in_this_profile() {
        let _lock = super::test_env_lock::hold();
        // In a debug build an unregistered argument makes `get_one` panic; in a
        // release build it silently yields the fallback. Running this in both
        // profiles pins both halves of the old divergence.
        assert_eq!(
            mode_on_init(&["--mutation-guard-mode", "OBSERVE"]),
            MutationGuardMode::Observe
        );
    }

    // --- existing behaviour that must not move ------------------------------

    #[test]
    fn start_graphql_api_behaviour_is_unchanged() {
        let _lock = super::test_env_lock::hold();
        let matches = THOTH
            .clone()
            .try_get_matches_from([
                "thoth",
                "start",
                "graphql-api",
                "--mutation-guard-mode",
                "OBSERVE",
            ])
            .expect("`start graphql-api` should parse");
        let arguments = matches
            .subcommand_matches("start")
            .and_then(|start| start.subcommand_matches("graphql-api"))
            .expect("graphql-api sub-matches");
        assert_eq!(
            commands::start::mutation_guard_mode(arguments).expect("mode should resolve"),
            MutationGuardMode::Observe
        );
    }

    #[test]
    fn every_other_init_argument_keeps_its_binding() {
        // Names, environment bindings and defaults of the pre-existing `init`
        // arguments, pinned so this task cannot have disturbed them.
        let expected: [(&str, Option<&str>, &[&str]); 11] = [
            ("db", Some("DATABASE_URL"), &[]),
            ("host", Some("GRAPHQL_API_HOST"), &["0.0.0.0"]),
            ("port", Some("GRAPHQL_API_PORT"), &["8000"]),
            ("threads", Some("GRAPHQL_API_THREADS"), &["5"]),
            ("keep-alive", Some("GRAPHQL_API_KEEP_ALIVE"), &["5"]),
            (
                "gql-url",
                Some("THOTH_GRAPHQL_API"),
                &["http://localhost:8000"],
            ),
            ("key", Some("PRIVATE_KEY"), &[]),
            (
                "zitadel-url",
                Some("ZITADEL_URL"),
                &["http://localhost:8282"],
            ),
            ("aws-access-key-id", Some("AWS_ACCESS_KEY_ID"), &[]),
            ("aws-secret-access-key", Some("AWS_SECRET_ACCESS_KEY"), &[]),
            ("aws-region", Some("AWS_REGION"), &[]),
        ];
        for (id, env, defaults) in expected {
            let arg = commands::INIT
                .get_arguments()
                .find(|arg| arg.get_id() == id)
                .unwrap_or_else(|| panic!("init must still declare `{id}`"));
            assert_eq!(
                arg.get_env().and_then(|value| value.to_str()),
                env,
                "environment binding of `{id}`"
            );
            assert_eq!(
                arg.get_default_values(),
                defaults,
                "default value of `{id}`"
            );
        }
    }

    // --- init ordering and failure behaviour --------------------------------

    #[test]
    fn init_runs_migrations_before_starting_the_api() {
        let order = RefCell::new(Vec::new());
        commands::run_init(
            || {
                order.borrow_mut().push("migrations");
                Ok(())
            },
            || {
                order.borrow_mut().push("api");
                Ok(())
            },
        )
        .expect("init should succeed when both steps succeed");
        assert_eq!(order.into_inner(), ["migrations", "api"]);
    }

    #[test]
    fn a_migration_failure_aborts_startup_and_the_api_never_starts() {
        let api_started = Cell::new(false);
        let result = commands::run_init(
            || Err(thoth::errors::ThothError::InternalError("boom".to_string())),
            || {
                api_started.set(true);
                Ok(())
            },
        );
        assert!(result.is_err(), "a migration failure must abort startup");
        assert!(
            !api_started.get(),
            "the API must not start after a failed migration"
        );
    }

    // --- security -----------------------------------------------------------

    #[test]
    fn an_invalid_mode_error_leaks_no_secret_bearing_value() {
        // The guard mode is not itself sensitive, but the failure is raised from
        // a command whose other arguments bind secret-bearing variables. No
        // *value* of any of them may appear in the rendered error.
        let secrets = [
            ("DATABASE_URL", "postgres://sentinel-db-value"),
            ("PRIVATE_KEY", "sentinel-private-key-value"),
            ("AWS_SECRET_ACCESS_KEY", "sentinel-aws-secret-value"),
        ];
        let _env = ModeEnv::set(Some("SOMETHING_ELSE"));
        let restore: Vec<_> = secrets
            .iter()
            .map(|(name, value)| {
                let previous = var_os(name);
                set_var(name, value);
                (*name, previous)
            })
            .collect();

        let rendered = parse_init(&["--mutation-guard-mode", "SOMETHING_ELSE"])
            .expect_err("an invalid mode must fail parsing")
            .to_string();

        for (name, previous) in restore {
            match previous {
                Some(previous) => set_var(name, previous),
                None => remove_var(name),
            }
        }

        for (name, value) in secrets {
            assert!(
                !rendered.contains(value),
                "the invalid-mode error leaked the value bound to `{name}`"
            );
        }
        assert!(
            rendered.contains("SOMETHING_ELSE"),
            "the error should name the rejected mode"
        );
    }
}

/// `BE-04` — the automatic distribution-job creation control path.
///
/// This is the same matrix `THOTH-GQL-OPS-02` established for the guard mode,
/// applied to `THOTH_DISTRIBUTION_JOB_CREATION`, and for the same reason: the
/// argument is read by the `graphql-api` handler, and **both** production
/// command paths dispatch into that handler. Registering it on only one is the
/// exact defect that task had to fix, and in pinned `clap_builder` the symptom
/// is profile-dependent — a debug build panics on an unregistered argument while
/// a release build silently resolves the fallback. The whole module therefore
/// runs in both profiles: `cargo test` exercises debug and
/// `cargo test --release` exercises release.
///
/// `OFF` is the merged default and is asserted as a declared property of the
/// argument, not only as a resolved value.
#[cfg(test)]
mod distribution_job_creation_control {
    use super::*;
    use clap::{ArgMatches, Command};
    use std::env::{remove_var, set_var, var_os};
    use std::ffi::OsString;
    use std::sync::MutexGuard;
    use thoth::api::model::distribution_job::DistributionJobCreation;

    const CREATION_ENV: &str = "THOTH_DISTRIBUTION_JOB_CREATION";

    /// Shared with the guard-mode module: both parse the same command tree, so
    /// one module's temporary invalid value must never be visible to the other.
    use super::test_env_lock::ENV_LOCK;

    struct CreationEnv {
        _lock: MutexGuard<'static, ()>,
        previous: Option<OsString>,
    }

    impl CreationEnv {
        fn set(value: Option<&str>) -> Self {
            let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            super::test_env_lock::initialise_command_tree();
            let previous = var_os(CREATION_ENV);
            match value {
                Some(value) => set_var(CREATION_ENV, value),
                None => remove_var(CREATION_ENV),
            }
            Self {
                _lock: lock,
                previous,
            }
        }
    }

    impl Drop for CreationEnv {
        fn drop(&mut self) {
            match self.previous.take() {
                Some(previous) => set_var(CREATION_ENV, previous),
                None => remove_var(CREATION_ENV),
            }
        }
    }

    // --- surface 1: the real `init` command, driven by argv -----------------

    fn parse_init(extra: &[&str]) -> Result<ArgMatches, clap::Error> {
        let mut argv = vec!["thoth", "init"];
        argv.extend_from_slice(extra);
        let matches = THOTH.clone().try_get_matches_from(argv)?;
        Ok(matches
            .subcommand_matches("init")
            .expect("`init` sub-matches")
            .clone())
    }

    fn creation_on_init(extra: &[&str]) -> DistributionJobCreation {
        commands::start::distribution_job_creation(&parse_init(extra).expect("`init` should parse"))
            .expect("setting should resolve")
    }

    // --- surface 2: the real `start graphql-api` command --------------------

    fn parse_start(extra: &[&str]) -> Result<ArgMatches, clap::Error> {
        let mut argv = vec!["thoth", "start", "graphql-api"];
        argv.extend_from_slice(extra);
        let matches = THOTH.clone().try_get_matches_from(argv)?;
        Ok(matches
            .subcommand_matches("start")
            .and_then(|start| start.subcommand_matches("graphql-api"))
            .expect("graphql-api sub-matches")
            .clone())
    }

    fn creation_on_start(extra: &[&str]) -> DistributionJobCreation {
        commands::start::distribution_job_creation(
            &parse_start(extra).expect("`start graphql-api` should parse"),
        )
        .expect("setting should resolve")
    }

    // --- surface 3: a fresh command, driven by the environment variable -----

    fn parse_fresh() -> Result<ArgMatches, clap::Error> {
        Command::new("init")
            .no_binary_name(true)
            .arg(arguments::distribution_job_creation())
            .try_get_matches_from::<_, &str>([])
    }

    fn creation_from_env() -> DistributionJobCreation {
        commands::start::distribution_job_creation(&parse_fresh().expect("should parse"))
            .expect("setting should resolve")
    }

    // --- compatibility matrix ----------------------------------------------

    #[test]
    fn unset_yields_off() {
        let _env = CreationEnv::set(None);
        assert_eq!(creation_from_env(), DistributionJobCreation::Off);
    }

    #[test]
    fn off_yields_off_on_both_production_paths() {
        let _env = CreationEnv::set(Some("OFF"));
        assert_eq!(creation_from_env(), DistributionJobCreation::Off);
        assert_eq!(
            creation_on_init(&["--distribution-job-creation", "OFF"]),
            DistributionJobCreation::Off
        );
        assert_eq!(
            creation_on_start(&["--distribution-job-creation", "OFF"]),
            DistributionJobCreation::Off
        );
    }

    #[test]
    fn on_yields_on_and_is_never_silently_off() {
        let _env = CreationEnv::set(Some("ON"));
        for resolved in [
            creation_from_env(),
            creation_on_init(&["--distribution-job-creation", "ON"]),
            creation_on_start(&["--distribution-job-creation", "ON"]),
        ] {
            assert_eq!(resolved, DistributionJobCreation::On);
            assert_ne!(
                resolved,
                DistributionJobCreation::Off,
                "ON must never be silently ignored on a production command path"
            );
        }
    }

    #[test]
    fn an_invalid_value_fails_startup_and_is_never_coerced_to_off() {
        let _env = CreationEnv::set(Some("MAYBE"));

        let from_env = parse_fresh().expect_err("an invalid environment value must fail parsing");
        assert_eq!(from_env.kind(), clap::error::ErrorKind::InvalidValue);

        for command_line in [
            parse_init(&["--distribution-job-creation", "MAYBE"]),
            parse_start(&["--distribution-job-creation", "MAYBE"]),
        ] {
            let error = command_line.expect_err("an invalid command-line value must fail parsing");
            assert_eq!(error.kind(), clap::error::ErrorKind::InvalidValue);
        }

        // Parsing fails, so no value is produced at all — least of all a
        // silently coerced `Off` that would look like a deliberate decision.
        assert!(parse_fresh().is_err());
    }

    #[test]
    fn the_typed_parser_accepts_exactly_off_and_on() {
        assert_eq!(
            "OFF".parse::<DistributionJobCreation>(),
            Ok(DistributionJobCreation::Off)
        );
        assert_eq!(
            "ON".parse::<DistributionJobCreation>(),
            Ok(DistributionJobCreation::On)
        );
        for invalid in ["", "off", "on", "true", "1", "ENABLED", "OFF "] {
            assert!(
                invalid.parse::<DistributionJobCreation>().is_err(),
                "`{invalid}` must not parse"
            );
        }
    }

    // --- direct regression tests for the registration defect ----------------

    #[test]
    fn both_production_command_paths_register_the_argument() {
        assert!(
            commands::INIT
                .get_arguments()
                .any(|arg| arg.get_id() == "distribution-job-creation"),
            "init must register distribution-job-creation: it dispatches into the \
             graphql-api handler, which reads it"
        );

        let graphql_api = commands::start::COMMAND
            .get_subcommands()
            .find(|command| command.get_name() == "graphql-api")
            .expect("start must declare graphql-api");
        assert!(
            graphql_api
                .get_arguments()
                .any(|arg| arg.get_id() == "distribution-job-creation"),
            "start graphql-api must register distribution-job-creation"
        );
    }

    #[test]
    fn both_paths_bind_the_documented_variable_and_the_off_default() {
        let graphql_api = commands::start::COMMAND
            .get_subcommands()
            .find(|command| command.get_name() == "graphql-api")
            .expect("start must declare graphql-api");
        let arguments = [
            commands::INIT
                .get_arguments()
                .find(|arg| arg.get_id() == "distribution-job-creation")
                .expect("distribution-job-creation on init"),
            graphql_api
                .get_arguments()
                .find(|arg| arg.get_id() == "distribution-job-creation")
                .expect("distribution-job-creation on start graphql-api"),
        ];
        for argument in arguments {
            assert_eq!(
                argument.get_env().and_then(|env| env.to_str()),
                Some(CREATION_ENV)
            );
            assert_eq!(
                argument.get_default_values(),
                ["OFF"],
                "the declared default must remain OFF, so the merged state is inactive"
            );
        }
    }

    #[test]
    fn resolving_the_setting_does_not_panic_in_this_profile() {
        let _lock = super::test_env_lock::hold();
        // In a debug build an unregistered argument makes `get_one` panic; in a
        // release build it silently yields the fallback. Running this in both
        // profiles pins both halves.
        assert_eq!(
            creation_on_init(&["--distribution-job-creation", "ON"]),
            DistributionJobCreation::On
        );
        assert_eq!(
            creation_on_start(&["--distribution-job-creation", "ON"]),
            DistributionJobCreation::On
        );
    }

    #[test]
    fn an_invalid_value_error_leaks_no_secret_bearing_value() {
        let secrets = [
            ("DATABASE_URL", "postgres://sentinel-db-value"),
            ("PRIVATE_KEY", "sentinel-private-key-value"),
            ("AWS_SECRET_ACCESS_KEY", "sentinel-aws-secret-value"),
        ];
        let _env = CreationEnv::set(Some("MAYBE"));
        let restore: Vec<_> = secrets
            .iter()
            .map(|(name, value)| {
                let previous = var_os(name);
                set_var(name, value);
                (*name, previous)
            })
            .collect();

        let rendered = parse_init(&["--distribution-job-creation", "MAYBE"])
            .expect_err("an invalid value must fail parsing")
            .to_string();

        for (name, previous) in restore {
            match previous {
                Some(previous) => set_var(name, previous),
                None => remove_var(name),
            }
        }

        for (name, value) in secrets {
            assert!(
                !rendered.contains(value),
                "the invalid-value error leaked the value bound to `{name}`"
            );
        }
    }

    #[test]
    fn the_mutation_guard_mode_control_is_untouched() {
        // `BE-04` adds a second, independent control on the same commands. It
        // must not disturb the first one, and it deliberately shares no
        // machinery with it.
        let arg = commands::INIT
            .get_arguments()
            .find(|arg| arg.get_id() == "mutation-guard-mode")
            .expect("mutation-guard-mode on init");
        assert_eq!(
            arg.get_env().and_then(|env| env.to_str()),
            Some("THOTH_GRAPHQL_MUTATION_GUARD_MODE")
        );
        assert_eq!(arg.get_default_values(), ["OFF"]);
    }
}
