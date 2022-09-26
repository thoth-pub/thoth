use crate::ThothError;

/// An array of tuples containing a database constraint name and a corresponding error to output
/// when the constraint is violated.
///
/// To obtain a list of unique and check constraints:
/// ```sql
/// SELECT conname
/// FROM pg_catalog.pg_constraint con
/// INNER JOIN pg_catalog.pg_class rel ON rel.oid = con.conrelid
/// INNER JOIN pg_catalog.pg_namespace nsp ON nsp.oid = connamespace
/// WHERE nsp.nspname = 'public'
/// AND contype in ('u', 'c');
/// ```
pub(crate) const DATABASE_CONSTRAINT_ERRORS: [(&str, &str); 3] = [
    (
        "contribution_contribution_ordinal_work_id_uniq",
        "A contribution with this ordinal number already exists.",
    ),
    (
        "work_relation_ordinal_type_uniq",
        "A relation with this ordinal number already exists.",
    ),
    (
        "affiliation_uniq_ord_in_contribution_idx",
        "An affiliation with this ordinal number already exists.",
    ),
];

impl From<diesel::result::Error> for ThothError {
    fn from(error: diesel::result::Error) -> ThothError {
        use diesel::result::Error;
        match error {
            Error::DatabaseError(_kind, info) => {
                if let Some(constraint_name) = info.constraint_name() {
                    for (constranint, error) in DATABASE_CONSTRAINT_ERRORS {
                        if constraint_name == constranint {
                            return ThothError::DatabaseConstraintError(error);
                        }
                    }
                }
                ThothError::DatabaseError(info.message().to_string())
            }
            Error::NotFound => ThothError::EntityNotFound,
            _ => ThothError::InternalError("".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::result::{DatabaseErrorKind, Error};

    struct TestDatabaseError {
        message: &'static str,
        constraint: Option<&'static str>,
    }
    impl diesel::result::DatabaseErrorInformation for TestDatabaseError {
        fn message(&self) -> &str {
            self.message
        }
        fn details(&self) -> Option<&str> {
            None
        }
        fn hint(&self) -> Option<&str> {
            None
        }
        fn table_name(&self) -> Option<&str> {
            None
        }
        fn column_name(&self) -> Option<&str> {
            None
        }
        fn constraint_name(&self) -> Option<&str> {
            self.constraint
        }
    }

    fn error_information(
        message: &'static str,
        constraint: Option<&'static str>,
    ) -> Box<TestDatabaseError> {
        Box::new(TestDatabaseError {
            message,
            constraint,
        })
    }

    #[test]
    fn test_unique_contribution_error() {
        let error_information = error_information(
            "duplicate key value violates unique constraint \"contribution_contribution_ordinal_work_id_uniq\"",
            Some("contribution_contribution_ordinal_work_id_uniq")
        );
        assert_eq!(
            ThothError::from(Error::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                error_information
            )),
            ThothError::DatabaseConstraintError(
                "A contribution with this ordinal number already exists."
            )
        )
    }
    #[test]
    fn test_unique_contribution_error_display() {
        let error_information = error_information(
            "duplicate key value violates unique constraint \"contribution_contribution_ordinal_work_id_uniq\"",
            Some("contribution_contribution_ordinal_work_id_uniq")
        );
        let error = ThothError::from(Error::DatabaseError(
            DatabaseErrorKind::UniqueViolation,
            error_information
        ));
        assert_eq!(
            format!("{}", error),
            "A contribution with this ordinal number already exists.",
        )
    }

    #[test]
    fn test_non_constraint_error() {
        let error_information = error_information(
            "Some error happened",
            None
        );
        assert_eq!(
            ThothError::from(Error::DatabaseError(
                DatabaseErrorKind::__Unknown,
                error_information
            )),
            ThothError::DatabaseError("Some error happened".to_string())
        )
    }

    #[test]
    fn test_non_constraint_error_display() {
        let error_information = error_information(
            "Some error happened",
            None
        );
        let error = ThothError::from(Error::DatabaseError(
            DatabaseErrorKind::__Unknown,
            error_information
        ));
        assert_eq!(
            format!("{}", error),
            "Database error: Some error happened",
        )
    }

    #[test]
    fn test_not_found_error() {
        assert_eq!(
            ThothError::from(Error::NotFound),
            ThothError::EntityNotFound
        )
    }
}