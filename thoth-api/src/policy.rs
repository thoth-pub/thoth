use uuid::Uuid;
use zitadel::actix::introspection::IntrospectedUser;

use crate::db::PgPool;
use crate::model::{Crud, PublisherId, PublisherIds};
use thoth_errors::{ThothError, ThothResult};

use std::collections::HashSet;
use strum::AsRefStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
enum Role {
    Superuser,
    PublisherAdmin,
    PublisherUser,
    WorkLifecycle,
    CdnWrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct PublisherPermissions {
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
        let role = Role::Superuser.as_ref();
        self.project_roles
            .as_ref()
            .is_some_and(|roles| roles.contains_key(role))
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

        // Collect org ids from all scoped project roles (excluding SUPERUSER).
        // This is future-proof: adding a new publisher-scoped role automatically enables publisher selection.
        let superuser_key = Role::Superuser.as_ref();
        for (role_key, scoped) in project_roles {
            if role_key == superuser_key {
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
    fn require_cdn_write_for<T: PublisherId>(&self, value: &T) -> ThothResult<&IntrospectedUser> {
        self.require_role_for_publisher(value, Role::CdnWrite)
    }

    /// Authorise the current user against all publishers derived from the given value.
    ///
    /// This is intended for entities that span more than one publisher scope, e.g. `WorkRelation`.
    fn require_publishers_for<T: PublisherIds>(&self, value: &T) -> ThothResult<&IntrospectedUser> {
        self.require_role_for_publishers(value, Role::PublisherUser)
    }

    /// Authorise the current user to edit publisher and imprint data for ALL publishers derived from the given value.
    fn require_publisher_admin_for_publishers<T: PublisherIds>(
        &self,
        value: &T,
    ) -> ThothResult<&IntrospectedUser> {
        self.require_role_for_publishers(value, Role::PublisherAdmin)
    }

    /// Authorise the current user to change lifecycle-related fields for ALL publishers derived from the given value.
    fn require_work_lifecycle_for_publishers<T: PublisherIds>(
        &self,
        value: &T,
    ) -> ThothResult<&IntrospectedUser> {
        self.require_role_for_publishers(value, Role::WorkLifecycle)
    }

    /// Authorise the current user to upload or modify files for ALL publishers derived from the given value.
    fn require_cdn_write_for_publishers<T: PublisherIds>(
        &self,
        value: &T,
    ) -> ThothResult<&IntrospectedUser> {
        self.require_role_for_publishers(value, Role::CdnWrite)
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
