use crate::arguments;
use base64::{engine::general_purpose, Engine as _};
use clap::{ArgMatches, Command};
use lazy_static::lazy_static;
use thoth::errors::{ThothError, ThothResult};
use zitadel::api::{
    clients::ClientBuilder,
    zitadel::app::v1::{
        ApiAuthMethodType, OidcAppType, OidcAuthMethodType, OidcGrantType, OidcResponseType,
        OidcTokenType, OidcVersion,
    },
    zitadel::authn::v1::KeyType,
    zitadel::management::v1::{
        AddApiAppRequest, AddAppKeyRequest, AddOidcAppRequest, AddProjectRequest,
        AddProjectRoleRequest, AddUserGrantRequest,
    },
    zitadel::project::v1::PrivateLabelingSetting,
    zitadel::user::v2::{ListUsersRequest, UserFieldName},
};

lazy_static! {
    pub(crate) static ref COMMAND: Command = Command::new("zitadel")
        .about("Manage Zitadel workflows")
        .arg(arguments::zitadel_url())
        .arg(arguments::thoth_pat())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("setup").about("Intial setup of OIDC APPs in zitadel"));
}

pub fn setup(arguments: &ArgMatches) -> ThothResult<()> {
    let zitadel_url = arguments.get_one::<String>("zitadel-url").unwrap();
    let pat = arguments.get_one::<String>("thoth-pat").unwrap();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()?;

    runtime.block_on(async {
        let mut management_client = ClientBuilder::new(zitadel_url)
            .with_access_token(pat)
            .build_management_client()
            .await?;
        let mut user_client = ClientBuilder::new(zitadel_url)
            .with_access_token(pat)
            .build_user_client()
            .await?;

        // Create Zitadel project
        let project_name = "Thoth";
        let project = management_client
            .add_project(AddProjectRequest {
                name: project_name.to_string(),
                project_role_assertion: false,
                project_role_check: false,
                has_project_check: false,
                private_labeling_setting: PrivateLabelingSetting::EnforceProjectResourceOwnerPolicy
                    as i32,
            })
            .await?
            .into_inner();
        println!("\n✅ Created Zitadel project: {}", project_name);

        // Create project user roles
        let roles = [
            ("SUPERUSER", "Superuser", "Superusers"),
            ("PUBLISHER_ADMIN", "Publisher Admin", "Publisher admins"),
            ("PUBLISHER_USER", "Publisher User", "Publisher users"),
        ];
        for (role_key, display_name, group) in roles {
            management_client
                .add_project_role(AddProjectRoleRequest {
                    project_id: project.id.clone(),
                    role_key: role_key.to_string(),
                    display_name: display_name.to_string(),
                    group: group.to_string(),
                })
                .await?;
            println!("\n✅ Added project role: {}", role_key);
        }

        // Assign SUPERUSER role to default accounts
        let users = user_client
            .list_users(ListUsersRequest {
                query: None,
                sorting_column: UserFieldName::CreationDate as i32,
                queries: vec![],
            })
            .await?
            .into_inner()
            .result;
        for user in users {
            management_client
                .add_user_grant(AddUserGrantRequest {
                    user_id: user.user_id.clone(),
                    project_id: project.id.clone(),
                    project_grant_id: "".to_string(),
                    role_keys: vec!["SUPERUSER".to_string()],
                })
                .await?;
            println!("\n✅ Granted SUPERUSER role to user: {}", user.username);
        }

        // Create Zitadel APPs for GraphQL API and APP
        let graphql_api_name = "Thoth GraphQL API";
        let graphql_api = management_client
            .add_api_app(AddApiAppRequest {
                project_id: project.id.clone(),
                name: graphql_api_name.to_string(),
                auth_method_type: ApiAuthMethodType::PrivateKeyJwt as i32,
            })
            .await?
            .into_inner();
        println!("\n✅ Created API app: {}", graphql_api_name);

        let graphql_api_key = management_client
            .add_app_key(AddAppKeyRequest {
                project_id: project.id.clone(),
                app_id: graphql_api.app_id,
                r#type: KeyType::Json as i32,
                expiration_date: None,
            })
            .await?
            .into_inner();
        let encoded_key = general_purpose::STANDARD.encode(&graphql_api_key.key_details);
        println!("\n✅ {} application key generated.", graphql_api_name);
        println!("👉 Please copy the following and add it to the `.env` file as `PRIVATE_KEY`:\n");
        println!("PRIVATE_KEY={}\n", encoded_key);

        let app_name = "Thoth APP";
        management_client
            .add_oidc_app(AddOidcAppRequest {
                project_id: project.id.clone(),
                name: app_name.to_string(),
                redirect_uris: vec!["http://localhost:8080/callback".to_string()],
                response_types: vec![OidcResponseType::Code as i32],
                grant_types: vec![OidcGrantType::AuthorizationCode as i32],
                app_type: OidcAppType::UserAgent as i32,
                auth_method_type: OidcAuthMethodType::None as i32, // PKCE
                post_logout_redirect_uris: vec!["http://localhost:8080/logout".to_string()],
                version: OidcVersion::OidcVersion10 as i32,
                dev_mode: true,
                access_token_type: OidcTokenType::Bearer as i32,
                access_token_role_assertion: false,
                id_token_role_assertion: false,
                id_token_userinfo_assertion: false,
                clock_skew: None,
                additional_origins: vec!["http://localhost:8080".to_string()],
                skip_native_app_success_page: false,
                back_channel_logout_uri: "".to_string(),
                login_version: None,
            })
            .await?;
        println!("\n✅ Created OIDC app: {}", app_name);

        Ok::<(), ThothError>(())
    })
}
