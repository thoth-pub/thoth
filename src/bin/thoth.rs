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
        .subcommand(commands::zitadel::COMMAND.clone());
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
        _ => unreachable!(),
    }
}

#[test]
fn test_cli() {
    THOTH.clone().debug_assert();
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
    use std::sync::{Mutex, MutexGuard};
    use thoth::api::graphql::MutationGuardMode;

    const MODE_ENV: &str = "THOTH_GRAPHQL_MUTATION_GUARD_MODE";

    /// The environment is process-global. Serialise every test that mutates it.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Sets `THOTH_GRAPHQL_MUTATION_GUARD_MODE` for the life of the guard and
    /// restores the previous value — or its absence — on drop.
    struct ModeEnv {
        _lock: MutexGuard<'static, ()>,
        previous: Option<OsString>,
    }

    impl ModeEnv {
        fn set(value: Option<&str>) -> Self {
            let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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

    #[test]
    fn store_availability_is_derived_only_from_enforce() {
        assert!(!MutationGuardMode::Off.store_available());
        assert!(!MutationGuardMode::Observe.store_available());
        assert!(MutationGuardMode::Enforce.store_available());
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
