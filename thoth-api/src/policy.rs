use uuid::Uuid;
use zitadel::actix::introspection::IntrospectedUser;

use crate::db::PgPool;
use crate::model::{Crud, PublisherId, PublisherIds};
use thoth_errors::{ThothError, ThothResult};

use std::collections::HashSet;
use strum::AsRefStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Role {
    Superuser,
    PublisherAdmin,
    PublisherUser,
    WorkLifecycle,
    CdnWrite,
    /// The `BE-04`/`DIS-02` durable distribution worker
    /// ([ADR-0008](../../docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md)
    /// section 3.2).
    ///
    /// This is a **Publisher-Services-specific** machine role with exactly three
    /// permitted operations, not a generic service role. It is unscoped because
    /// a back-catalogue worker genuinely operates across every publisher, which
    /// is the only condition under which ADR-0008 section 3.1 permits an
    /// unscoped machine role: a per-organisation grant would have to be
    /// re-issued for every publisher and would silently skip the first
    /// unenrolled one, which is a fail-*open* failure mode for coverage.
    ///
    /// It confers **no** publisher scope, no `CDN_WRITE` capability and no
    /// Metrics permission, and `SUPERUSER` does not imply it.
    DisseminationWorker,
}

/// The unscoped-role check, expressed once.
///
/// This trait is **private to this module** deliberately. It is the shared
/// implementation of a key-presence check on `project_roles`, and nothing more:
/// it is not a general service-role API, there is no `ServiceRole` type, no role
/// registry and no machine-identity storage.
///
/// Sharing this implementation pattern between `SUPERUSER` and
/// `DISSEMINATION_WORKER` says nothing about what either role may do. ADR-0008
/// section 3.1 fixes that independently: `SUPERUSER` authority does not imply
/// machine-role authority, and holding a machine role confers no administrative
/// authority.
trait UnscopedRoleAccess {
    fn has_unscoped_role(&self, role: Role) -> bool;
}

impl UnscopedRoleAccess for IntrospectedUser {
    fn has_unscoped_role(&self, role: Role) -> bool {
        let role = role.as_ref();
        self.project_roles
            .as_ref()
            .is_some_and(|roles| roles.contains_key(role))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PublisherPermissions {
    pub publisher_admin: bool,
    pub work_lifecycle: bool,
    pub cdn_write: bool,
}

impl PublisherPermissions {
    pub fn for_superuser() -> Self {
        Self {
            publisher_admin: true,
            work_lifecycle: true,
            cdn_write: true,
        }
    }
}

pub(crate) trait UserAccess {
    fn is_superuser(&self) -> bool;

    /// Whether the user holds the unscoped `DISSEMINATION_WORKER` project role.
    ///
    /// This is the one explicit guard predicate for that role. There is no role
    /// inheritance anywhere in this module, so this is true only when the role
    /// itself is present.
    fn is_dissemination_worker(&self) -> bool;

    /// Returns true if the user has the given role scoped to the given ZITADEL organisation id.
    fn has_role_for_org(&self, role: Role, org_id: &str) -> bool;

    /// Return all ZITADEL organisation ids the user has any publisher-scoped role for.
    ///
    /// This is intended for building publisher switcher lists in the frontend.
    fn publisher_org_ids(&self) -> Vec<String>;

    /// Compute the user's permissions for a specific publisher organisation.
    fn permissions_for_org(&self, org_id: &str) -> PublisherPermissions;
}

impl UserAccess for IntrospectedUser {
    fn is_superuser(&self) -> bool {
        self.has_unscoped_role(Role::Superuser)
    }

    fn is_dissemination_worker(&self) -> bool {
        self.has_unscoped_role(Role::DisseminationWorker)
    }

    fn has_role_for_org(&self, role: Role, org_id: &str) -> bool {
        let role = role.as_ref();
        self.project_roles
            .as_ref()
            .and_then(|roles| roles.get(role))
            .and_then(|scoped| scoped.get(org_id))
            .is_some()
    }

    fn publisher_org_ids(&self) -> Vec<String> {
        if self.is_superuser() {
            // Superusers can access everything; callers should decide how to list publishers.
            return vec![];
        }

        let Some(project_roles) = self.project_roles.as_ref() else {
            return vec![];
        };

        let mut org_ids: HashSet<String> = HashSet::new();

        // Collect org ids from all scoped project roles, excluding the unscoped
        // ones. This is future-proof: adding a new publisher-scoped role
        // automatically enables publisher selection.
        //
        // `DISSEMINATION_WORKER` is excluded for the same reason `SUPERUSER` is:
        // it is an unscoped project role, so any organisation key present under
        // it is not a publisher the account may act for. Without this a
        // worker-only account would appear to hold publisher organisations in
        // the frontend switcher list, which it must not.
        let unscoped_keys = [
            Role::Superuser.as_ref(),
            Role::DisseminationWorker.as_ref(),
        ];
        for (role_key, scoped) in project_roles {
            if unscoped_keys.contains(&role_key.as_str()) {
                continue;
            }

            for org_id in scoped.keys() {
                org_ids.insert(org_id.clone());
            }
        }

        let mut out: Vec<String> = org_ids.into_iter().collect();
        out.sort();
        out
    }

    fn permissions_for_org(&self, org_id: &str) -> PublisherPermissions {
        if self.is_superuser() {
            return PublisherPermissions::for_superuser();
        }

        PublisherPermissions {
            publisher_admin: self.has_role_for_org(Role::PublisherAdmin, org_id),
            work_lifecycle: self.has_role_for_org(Role::WorkLifecycle, org_id),
            cdn_write: self.has_role_for_org(Role::CdnWrite, org_id),
        }
    }
}

pub(crate) trait PolicyContext {
    /// Return a reference to the database connection pool for the current request context.
    fn db(&self) -> &PgPool;

    /// Return the authenticated user for the current request, if any.
    fn user(&self) -> Option<&IntrospectedUser>;

    /// Require that a user is authenticated and return the authenticated user.
    ///
    /// # Errors
    ///
    /// Returns [`ThothError::Unauthorised`] if no user is present in the context.
    fn require_authentication(&self) -> ThothResult<&IntrospectedUser> {
        self.user().ok_or(ThothError::Unauthorised)
    }

    /// Return the user id of the authenticated user.
    ///
    /// # Errors
    ///
    /// Returns [`ThothError::Unauthorised`] if no user is present in the context.
    fn user_id(&self) -> ThothResult<&str> {
        self.user()
            .map(|u| u.user_id.as_str())
            .ok_or(ThothError::Unauthorised)
    }

    /// Require that the authenticated user has the `SUPERUSER` role.
    ///
    /// # Errors
    ///
    /// Returns [`ThothError::Unauthorised`] if the user is not authenticated or does not have
    /// the superuser role.
    fn require_superuser(&self) -> ThothResult<&IntrospectedUser> {
        let user = self.require_authentication()?;
        if user.is_superuser() {
            Ok(user)
        } else {
            Err(ThothError::Unauthorised)
        }
    }

    /// Require that the authenticated user holds the `DISSEMINATION_WORKER`
    /// role.
    ///
    /// This is the one explicit guard for the three permitted worker
    /// operations. It mirrors [`Self::require_superuser`] in shape and in
    /// nothing else: `SUPERUSER` does **not** satisfy it, and this guard confers
    /// no publisher scope, no administrative authority and no capability of any
    /// other role. A principal that must genuinely act as both is granted both
    /// roles explicitly, and may then exercise exactly the operations each is
    /// independently allowed — which is a `BE-04` matrix decision and not a
    /// general role-composition rule.
    ///
    /// # Errors
    ///
    /// Returns [`ThothError::Unauthorised`] if the user is not authenticated or
    /// does not hold the role.
    fn require_dissemination_worker(&self) -> ThothResult<&IntrospectedUser> {
        let user = self.require_authentication()?;
        if user.is_dissemination_worker() {
            Ok(user)
        } else {
            Err(ThothError::Unauthorised)
        }
    }

    /// Authorise the current user against the publisher derived from the given value.
    fn require_publisher_for<T: PublisherId>(&self, value: &T) -> ThothResult<&IntrospectedUser> {
        self.require_role_for_publisher(value, Role::PublisherUser)
    }

    /// Authorise the current user to edit publisher and imprint data for the publisher derived from the given value.
    fn require_publisher_admin_for<T: PublisherId>(
        &self,
        value: &T,
    ) -> ThothResult<&IntrospectedUser> {
        self.require_role_for_publisher(value, Role::PublisherAdmin)
    }

    /// Authorise the current user to change lifecycle-related fields (status, publication date, superseding, etc.).
    fn require_work_lifecycle_for<T: PublisherId>(
        &self,
        value: &T,
    ) -> ThothResult<&IntrospectedUser> {
        self.require_role_for_publisher(value, Role::WorkLifecycle)
    }

    /// Authorise the current user to upload or modify files for the publisher derived from the given value.
    #[allow(dead_code)]
    fn require_cdn_write_for<T: PublisherId>(&self, value: &T) -> ThothResult<&IntrospectedUser> {
        self.require_role_for_publisher(value, Role::CdnWrite)
    }

    /// Authorise the current user against all publishers derived from the given value.
    ///
    /// This is intended for entities that span more than one publisher scope, e.g. `WorkRelation`.
    fn require_publishers_for<T: PublisherIds>(&self, value: &T) -> ThothResult<&IntrospectedUser> {
        self.require_role_for_publishers(value, Role::PublisherUser)
    }

    /// Authorise the current user against the publisher derived from the given value,
    /// requiring the specified ZITADEL project role for that publisher's organisation.
    fn require_role_for_publisher<T: PublisherId>(
        &self,
        value: &T,
        role: Role,
    ) -> ThothResult<&IntrospectedUser> {
        let user = self.require_authentication()?;
        if user.is_superuser() {
            return Ok(user);
        }

        let org_id = value.zitadel_id(self.db())?;

        if user.has_role_for_org(role, &org_id) {
            Ok(user)
        } else {
            Err(ThothError::Unauthorised)
        }
    }

    /// Authorise the current user against all publishers derived from the given value,
    /// requiring the specified ZITADEL project role for each publisher's organisation.
    ///
    /// This is intended for entities that span more than one publisher scope, e.g. `WorkRelation`.
    fn require_role_for_publishers<T: PublisherIds>(
        &self,
        value: &T,
        role: Role,
    ) -> ThothResult<&IntrospectedUser> {
        let user = self.require_authentication()?;
        if user.is_superuser() {
            return Ok(user);
        }

        for org_id in value.zitadel_ids(self.db())? {
            if !user.has_role_for_org(role, &org_id) {
                return Err(ThothError::Unauthorised);
            }
        }

        Ok(user)
    }

    /// Load an entity by primary key after requiring authentication.
    fn load_current<T: Crud>(&self, id: &Uuid) -> ThothResult<T> {
        self.require_authentication()?;
        T::from_id(self.db(), id)
    }
}

/// A policy for create actions.
///
/// Some create operations require additional parameters beyond the `New*` input (e.g. markup
/// format). Use the `Params` type parameter for those cases.
pub(crate) trait CreatePolicy<New, Params = ()> {
    fn can_create<C: PolicyContext>(ctx: &C, data: &New, params: Params) -> ThothResult<()>;
}

/// A policy for update actions.
///
/// Some update operations require additional parameters beyond the `Patch*` input.
pub(crate) trait UpdatePolicy<Model, Patch, Params = ()> {
    fn can_update<C: PolicyContext>(
        ctx: &C,
        current: &Model,
        patch: &Patch,
        params: Params,
    ) -> ThothResult<()>;
}

/// A policy for delete actions.
pub(crate) trait DeletePolicy<Model> {
    fn can_delete<C: PolicyContext>(ctx: &C, current: &Model) -> ThothResult<()>;
}

/// A policy for move / reorder actions.
pub(crate) trait MovePolicy<Model> {
    fn can_move<C: PolicyContext>(ctx: &C, current: &Model) -> ThothResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    fn mk_user(
        project_roles: Option<HashMap<String, HashMap<String, String>>>,
    ) -> IntrospectedUser {
        IntrospectedUser {
            user_id: "user-1".to_string(),
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

    fn scoped(org_id: &str) -> HashMap<String, String> {
        let mut m = HashMap::new();
        // ZITADEL stores a label/value (often domain) as the map value; it is irrelevant for our checks.
        m.insert(org_id.to_string(), "label".to_string());
        m
    }

    #[test]
    fn role_as_ref_is_screaming_snake_case() {
        assert_eq!(Role::Superuser.as_ref(), "SUPERUSER");
        assert_eq!(Role::PublisherAdmin.as_ref(), "PUBLISHER_ADMIN");
        assert_eq!(Role::PublisherUser.as_ref(), "PUBLISHER_USER");
        assert_eq!(Role::WorkLifecycle.as_ref(), "WORK_LIFECYCLE");
        assert_eq!(Role::CdnWrite.as_ref(), "CDN_WRITE");
        assert_eq!(Role::DisseminationWorker.as_ref(), "DISSEMINATION_WORKER");
    }

    #[test]
    fn is_dissemination_worker_checks_project_roles_key() {
        let mut roles: HashMap<String, HashMap<String, String>> = HashMap::new();
        roles.insert(
            Role::DisseminationWorker.as_ref().to_string(),
            HashMap::new(),
        );

        let user = mk_user(Some(roles));
        assert!(user.is_dissemination_worker());

        assert!(!mk_user(None).is_dissemination_worker());
    }

    #[test]
    fn neither_superuser_nor_the_worker_role_implies_the_other() {
        let mut superuser_roles: HashMap<String, HashMap<String, String>> = HashMap::new();
        superuser_roles.insert(Role::Superuser.as_ref().to_string(), HashMap::new());
        let superuser = mk_user(Some(superuser_roles));
        assert!(superuser.is_superuser());
        assert!(
            !superuser.is_dissemination_worker(),
            "SUPERUSER must not confer machine-role authority"
        );

        let mut worker_roles: HashMap<String, HashMap<String, String>> = HashMap::new();
        worker_roles.insert(
            Role::DisseminationWorker.as_ref().to_string(),
            HashMap::new(),
        );
        let worker = mk_user(Some(worker_roles));
        assert!(worker.is_dissemination_worker());
        assert!(
            !worker.is_superuser(),
            "a machine role must not confer administrative authority"
        );
    }

    #[test]
    fn a_worker_only_account_holds_no_publisher_scope_or_permission() {
        let mut roles: HashMap<String, HashMap<String, String>> = HashMap::new();
        // ZITADEL may still carry an organisation key under an unscoped role;
        // it is not a publisher this account may act for.
        roles.insert(
            Role::DisseminationWorker.as_ref().to_string(),
            scoped("org-1"),
        );
        let user = mk_user(Some(roles));

        assert!(
            user.publisher_org_ids().is_empty(),
            "a worker account must not appear to hold publisher organisations"
        );
        assert_eq!(
            user.permissions_for_org("org-1"),
            PublisherPermissions::default(),
            "the worker role confers no publisher_admin, work_lifecycle or cdn_write capability"
        );
        assert_eq!(
            user.permissions_for_org("org-2"),
            PublisherPermissions::default()
        );
        assert!(!user.has_role_for_org(Role::PublisherUser, "org-1"));
        assert!(!user.has_role_for_org(Role::CdnWrite, "org-1"));
    }

    #[test]
    fn publisher_org_ids_still_collects_scoped_roles_alongside_the_worker_role() {
        let mut roles: HashMap<String, HashMap<String, String>> = HashMap::new();
        roles.insert(Role::PublisherUser.as_ref().to_string(), scoped("org-1"));
        roles.insert(
            Role::DisseminationWorker.as_ref().to_string(),
            scoped("org-9"),
        );

        let user = mk_user(Some(roles));
        assert_eq!(user.publisher_org_ids(), vec!["org-1".to_string()]);
    }

    #[test]
    fn is_superuser_checks_project_roles_key() {
        let mut roles: HashMap<String, HashMap<String, String>> = HashMap::new();
        roles.insert(Role::Superuser.as_ref().to_string(), HashMap::new());

        let user = mk_user(Some(roles));
        assert!(user.is_superuser());

        let user = mk_user(None);
        assert!(!user.is_superuser());
    }

    #[test]
    fn has_role_for_org_requires_scope_match() {
        let mut roles: HashMap<String, HashMap<String, String>> = HashMap::new();
        roles.insert(Role::PublisherUser.as_ref().to_string(), scoped("org-1"));

        let user = mk_user(Some(roles));
        assert!(user.has_role_for_org(Role::PublisherUser, "org-1"));
        assert!(!user.has_role_for_org(Role::PublisherUser, "org-2"));
        assert!(!user.has_role_for_org(Role::PublisherAdmin, "org-1"));
    }

    #[test]
    fn publisher_org_ids_collects_all_scoped_orgs_except_superuser() {
        let mut roles: HashMap<String, HashMap<String, String>> = HashMap::new();
        roles.insert(Role::PublisherUser.as_ref().to_string(), scoped("org-1"));

        // add another role with overlapping and new orgs
        let mut admin_scoped = scoped("org-2");
        admin_scoped.insert("org-1".to_string(), "label".to_string());
        roles.insert(Role::PublisherAdmin.as_ref().to_string(), admin_scoped);

        let user = mk_user(Some(roles));
        let orgs = user.publisher_org_ids();

        assert_eq!(orgs, vec!["org-1".to_string(), "org-2".to_string()]);
    }

    #[test]
    fn publisher_org_ids_is_empty_for_superuser() {
        let mut roles: HashMap<String, HashMap<String, String>> = HashMap::new();
        roles.insert(Role::Superuser.as_ref().to_string(), HashMap::new());

        let user = mk_user(Some(roles));
        assert!(user.publisher_org_ids().is_empty());
    }

    #[test]
    fn permissions_for_org_sets_booleans_from_roles() {
        let mut roles: HashMap<String, HashMap<String, String>> = HashMap::new();
        roles.insert(Role::PublisherAdmin.as_ref().to_string(), scoped("org-1"));
        roles.insert(Role::WorkLifecycle.as_ref().to_string(), scoped("org-1"));

        let user = mk_user(Some(roles));
        let p = user.permissions_for_org("org-1");

        assert!(p.publisher_admin);
        assert!(p.work_lifecycle);
        assert!(!p.cdn_write);

        // different org should yield no permissions
        let p = user.permissions_for_org("org-2");
        assert_eq!(p, PublisherPermissions::default());
    }

    #[test]
    fn permissions_for_org_all_true_for_superuser() {
        let mut roles: HashMap<String, HashMap<String, String>> = HashMap::new();
        roles.insert(Role::Superuser.as_ref().to_string(), HashMap::new());

        let user = mk_user(Some(roles));
        let p = user.permissions_for_org("any");

        assert_eq!(p, PublisherPermissions::for_superuser());
    }
}
