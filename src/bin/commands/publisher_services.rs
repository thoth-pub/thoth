//! The `MIG-01` administrative audit/backfill command.
//!
//! This is a deliberately thin CLI wiring: it parses only filesystem paths and
//! simple operational arguments (database connection, manifest/plan/report paths,
//! the expected reviewed-plan hash, the production-preflight flag, the effective
//! automatic-job-creation setting and the optional per-publisher lock envelope)
//! and hands them to the `thoth-api` `MIG-01` administrative facade.
//!
//! All domain behaviour — reading and parsing the manifest, generating and
//! parsing the canonical plan, SHA-256 calculation, canonical serialization,
//! mapping, recovery classification and the apply through the canonical
//! coordinator — lives in `thoth-api`. This command never parses `MIG-01` JSON,
//! serializes a plan, computes a hash or reproduces migration-domain rules.

use std::path::PathBuf;

use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use lazy_static::lazy_static;
use thoth::{
    api::{
        db::init_pool,
        model::publisher_service_configuration::migration_backfill::{
            apply as run_apply, dry_run as run_dry_run, ApplyExecutionMode, ApplyRequest,
            DryRunRequest,
        },
    },
    errors::ThothResult,
};

use crate::arguments;
use crate::commands::start::distribution_job_creation;

lazy_static! {
    pub(crate) static ref COMMAND: Command = Command::new("publisher-services")
        .about("Publisher Services administrative tooling")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("migration-backfill")
                .about(
                    "MIG-01 audit and backfill of canonical publisher package and \
                     distribution-platform configuration"
                )
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("dry-run")
                        .about(
                            "Produce a deterministic dry-run plan and reconciliation report; \
                             write no database change"
                        )
                        .arg(arguments::database())
                        .arg(manifest())
                        .arg(plan_out())
                        .arg(report_out())
                        .arg(production())
                        .arg(arguments::distribution_job_creation()),
                )
                .subcommand(
                    Command::new("apply")
                        .about(
                            "Apply exactly a reviewed plan through the canonical coordinator \
                             with MIGRATION_BACKFILL provenance"
                        )
                        .arg(arguments::database())
                        .arg(manifest())
                        .arg(plan())
                        .arg(expected_plan_sha256())
                        .arg(report_out())
                        .arg(execution_mode())
                        .arg(max_works_per_publisher())
                        .arg(arguments::distribution_job_creation()),
                ),
        );
}

fn manifest() -> Arg {
    Arg::new("manifest")
        .long("manifest")
        .value_name("MANIFEST_PATH")
        .help("Path to the approved, immutable MIG-01 input manifest")
        .value_parser(value_parser!(PathBuf))
        .required(true)
        .num_args(1)
}

fn plan_out() -> Arg {
    Arg::new("plan-out")
        .long("plan-out")
        .value_name("PLAN_PATH")
        .help("Path to write the canonical dry-run plan")
        .value_parser(value_parser!(PathBuf))
        .required(true)
        .num_args(1)
}

fn plan() -> Arg {
    Arg::new("plan")
        .long("plan")
        .value_name("PLAN_PATH")
        .help("Path to the exact reviewed canonical plan to apply")
        .value_parser(value_parser!(PathBuf))
        .required(true)
        .num_args(1)
}

fn report_out() -> Arg {
    Arg::new("report-out")
        .long("report-out")
        .value_name("REPORT_PATH")
        .help("Path to write the bounded reconciliation report")
        .value_parser(value_parser!(PathBuf))
        .required(true)
        .num_args(1)
}

fn expected_plan_sha256() -> Arg {
    Arg::new("expected-plan-sha256")
        .long("expected-plan-sha256")
        .value_name("SHA256_HEX")
        .help("The expected reviewed-plan SHA-256 (lowercase hexadecimal)")
        .required(true)
        .num_args(1)
}

fn production() -> Arg {
    Arg::new("production")
        .long("production")
        .help("Require the strict production job-state preflight to pass first")
        .action(ArgAction::SetTrue)
}

/// The required, explicit apply execution scope. `production` additionally
/// requires `--max-works-per-publisher` (clap enforces this), so a production
/// apply can never be launched without an approved lock envelope.
fn execution_mode() -> Arg {
    Arg::new("execution-mode")
        .long("execution-mode")
        .value_name("MODE")
        .help("Apply execution scope: disposable or production")
        .value_parser(["disposable", "production"])
        .required(true)
        .num_args(1)
}

fn max_works_per_publisher() -> Arg {
    Arg::new("max-works-per-publisher")
        .long("max-works-per-publisher")
        .value_name("COUNT")
        .help(
            "The approved maximum per-publisher work-count lock envelope; a pending \
             publisher exceeding it stops the run before that publisher is written. \
             Required for --execution-mode production",
        )
        .value_parser(value_parser!(i64).range(0..))
        .required_if_eq("execution-mode", "production")
        .num_args(1)
}

pub fn dry_run(arguments: &ArgMatches) -> ThothResult<()> {
    let database_url = arguments.get_one::<String>("db").unwrap();
    let pool = init_pool(database_url);
    let request = DryRunRequest {
        manifest_path: arguments.get_one::<PathBuf>("manifest").unwrap(),
        plan_out_path: arguments.get_one::<PathBuf>("plan-out").unwrap(),
        report_out_path: arguments.get_one::<PathBuf>("report-out").unwrap(),
        run_production_preflight: arguments.get_flag("production"),
        job_creation: distribution_job_creation(arguments)?,
    };
    let outcome = run_dry_run(&pool, &request)?;
    println!("MIG-01 dry run complete.");
    println!("  manifest SHA-256: {}", outcome.manifest_sha256);
    println!("  plan SHA-256:     {}", outcome.plan_sha256);
    println!(
        "  publishers considered: {}, changing: {}, no-op: {}",
        outcome.report.publishers_considered,
        outcome.report.affected_publishers,
        outcome.report.publishers_considered - outcome.report.affected_publishers,
    );
    println!(
        "  expected distribution jobs/targets/attempts: {}/{}/{}",
        outcome.report.expected_distribution_jobs,
        outcome.report.expected_distribution_job_targets,
        outcome.report.expected_distribution_job_attempts,
    );
    Ok(())
}

pub fn apply(arguments: &ArgMatches) -> ThothResult<()> {
    let database_url = arguments.get_one::<String>("db").unwrap();
    let pool = init_pool(database_url);
    let max_works = arguments.get_one::<i64>("max-works-per-publisher").copied();
    // `--execution-mode` is required and its value is restricted by clap; the
    // production envelope is enforced as required by `required_if_eq`, so the
    // `Production` variant here always carries an approved envelope.
    let mode = match arguments
        .get_one::<String>("execution-mode")
        .unwrap()
        .as_str()
    {
        "production" => ApplyExecutionMode::Production {
            max_works_per_publisher: max_works
                .expect("clap requires --max-works-per-publisher for production"),
        },
        _ => ApplyExecutionMode::Disposable {
            max_works_per_publisher: max_works,
        },
    };
    let request = ApplyRequest {
        manifest_path: arguments.get_one::<PathBuf>("manifest").unwrap(),
        plan_path: arguments.get_one::<PathBuf>("plan").unwrap(),
        expected_plan_sha256: arguments.get_one::<String>("expected-plan-sha256").unwrap(),
        report_out_path: arguments.get_one::<PathBuf>("report-out").unwrap(),
        mode,
        job_creation: distribution_job_creation(arguments)?,
    };
    let outcome = run_apply(&pool, &request)?;
    println!("MIG-01 apply complete.");
    println!("  plan SHA-256: {}", outcome.plan_sha256);
    println!(
        "  written: {}, already applied: {}, reviewed no-ops: {}",
        outcome.written, outcome.already_applied, outcome.reviewed_noops,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse a `publisher-services migration-backfill apply` argument vector
    /// through the real command tree.
    fn parse_apply(extra: &[&str]) -> Result<clap::ArgMatches, clap::Error> {
        let mut argv = vec![
            "publisher-services",
            "migration-backfill",
            "apply",
            "--database-url",
            "postgres://localhost/x",
            "--manifest",
            "m.json",
            "--plan",
            "p.json",
            "--expected-plan-sha256",
            "abc",
            "--report-out",
            "r.json",
        ];
        argv.extend_from_slice(extra);
        COMMAND.clone().try_get_matches_from(argv)
    }

    #[test]
    fn production_apply_requires_a_lock_envelope() {
        // Production without an envelope is rejected at parse time: the unsafe
        // production combination is never even constructed.
        let error = parse_apply(&["--execution-mode", "production"])
            .expect_err("production without an envelope must fail to parse");
        assert_eq!(
            error.kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );

        // Production WITH an envelope parses.
        assert!(parse_apply(&[
            "--execution-mode",
            "production",
            "--max-works-per-publisher",
            "100"
        ])
        .is_ok());
    }

    #[test]
    fn apply_requires_an_explicit_execution_mode() {
        let error = parse_apply(&[]).expect_err("execution-mode is required");
        assert_eq!(
            error.kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn disposable_apply_does_not_require_an_envelope() {
        assert!(parse_apply(&["--execution-mode", "disposable"]).is_ok());
        assert!(parse_apply(&[
            "--execution-mode",
            "disposable",
            "--max-works-per-publisher",
            "5"
        ])
        .is_ok());
    }

    #[test]
    fn an_unknown_execution_mode_is_rejected() {
        let error = parse_apply(&["--execution-mode", "prod"])
            .expect_err("only disposable/production are accepted");
        assert_eq!(error.kind(), clap::error::ErrorKind::InvalidValue);
    }
}
