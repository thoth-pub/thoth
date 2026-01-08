use uuid::Uuid;
use zitadel::actix::introspection::IntrospectedUser;

use crate::model::{PublisherId, PublisherIds};
use crate::db::PgPool;
use thoth_errors::{ThothError, ThothResult};

pub(crate) trait UserAccess {
    fn is_superuser(&self) -> bool;
    fn can_edit(&self, publisher_id: &Uuid) -> ThothResult<()>;
}

impl UserAccess for IntrospectedUser {
    fn is_superuser(&self) -> bool {
        self.project_roles
            .as_ref()
            .is_some_and(|roles| roles.contains_key("SUPERUSER"))
    }

    /// Determines whether the user has edit permissions for the given `publisher_id`.
    ///
    /// A user is authorized to edit a publisher if:
    /// - They have the `SUPERUSER` role (see [`is_superuser`]) — or
    /// - Their `metadata` includes a `publishers` key containing a
    ///   comma-separated list of UUIDs they are associated with.
    ///
    /// ### Expected Metadata Format
    ///
    /// ```json
    /// {
    ///   "publishers": "85fd969a-a16c-480b-b641-cb9adf979c3b, 12345678-9abc-def0-1234-56789abcdef0"
    /// }
    /// ```
    ///
    /// The value **must** be a single string of UUIDs, separated by commas,
    /// with optional whitespace.
    ///
    /// If the `publishers` key is missing, or does not contain the provided `publisher_id`,
    /// the user is considered unauthorised.
    ///
    /// # Errors
    ///
    /// Returns [`ThothError::Unauthorised`] if the user is not a superuser and
    /// does not have access to the given publisher.
    fn can_edit(&self, publisher_id: &Uuid) -> ThothResult<()> {
        if self.is_superuser() {
            return Ok(());
        }

        self.metadata
            .as_ref()
            .and_then(|meta| meta.get("publishers"))
            .map(|val| val.as_str())
            .map(|raw| {
                raw.split(',')
                    .map(str::trim)
                    .filter_map(|s| Uuid::parse_str(s).ok())
                    .any(|id| id == *publisher_id)
            })
            .filter(|&matches| matches)
            .map(|_| ())
            .ok_or(ThothError::Unauthorised)
    }
}

pub(crate) trait PolicyContext {
    fn db(&self) -> &PgPool;
    fn user(&self) -> Option<&IntrospectedUser>;

    fn require_authentication(&self) -> ThothResult<&IntrospectedUser> {
        self.user().ok_or(ThothError::Unauthorised)
    }

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
        let user = self.require_authentication()?;
        let publisher_id = value.publisher_id(self.db())?;
        user.can_edit(&publisher_id)?;
        Ok(user)
    }

    /// Authorise the current user against all publishers derived from the given value.
    ///
    /// This is intended for entities that span more than one publisher scope, e.g. `WorkRelation`.
    fn require_publishers_for<T: PublisherIds>(&self, value: &T) -> ThothResult<&IntrospectedUser> {
        let user = self.require_authentication()?;
        for publisher_id in value.publisher_ids(self.db())? {
            user.can_edit(&publisher_id)?;
        }
        Ok(user)
    }
}
