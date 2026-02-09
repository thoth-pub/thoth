use super::*;
use crate::model::Crud;
use uuid::Uuid;

fn make_new_contact(
    publisher_id: Uuid,
    contact_type: ContactType,
    email: impl Into<String>,
) -> NewContact {
    NewContact {
        publisher_id,
        contact_type,
        email: email.into(),
    }
}

fn make_patch_contact(
    contact: &Contact,
    contact_type: ContactType,
    email: impl Into<String>,
) -> PatchContact {
    PatchContact {
        contact_id: contact.contact_id,
        publisher_id: contact.publisher_id,
        contact_type,
        email: email.into(),
    }
}

fn make_contact(pool: &crate::db::PgPool, publisher_id: Uuid, email: String) -> Contact {
    let new_contact = make_new_contact(publisher_id, ContactType::Accessibility, email);

    Contact::create(pool, &new_contact).expect("Failed to create contact")
}

mod defaults {
    use super::*;

    #[test]
    fn contactfield_default_is_email() {
        let contfield: ContactField = Default::default();
        assert_eq!(contfield, ContactField::Email);
    }
}

mod helpers {
    use super::*;
    use crate::model::{Crud, HistoryEntry};

    #[test]
    fn pk_returns_id() {
        let contact: Contact = Default::default();
        assert_eq!(contact.pk(), contact.contact_id);
    }

    #[test]
    fn history_entry_serializes_model() {
        let contact: Contact = Default::default();
        let user_id = "12345";
        let new_contact_history = contact.new_history_entry(user_id);
        assert_eq!(new_contact_history.contact_id, contact.contact_id);
        assert_eq!(new_contact_history.user_id, user_id);
        assert_eq!(
            new_contact_history.data,
            serde_json::Value::String(serde_json::to_string(&contact).unwrap())
        );
    }
}

#[cfg(feature = "backend")]
mod policy {
    use super::*;

    use crate::model::contact::policy::ContactPolicy;
    use crate::model::tests::db::{
        create_publisher, setup_test_db, test_context_with_user, test_user_with_role,
    };
    use crate::model::Crud;
    use crate::policy::{CreatePolicy, DeletePolicy, Role, UpdatePolicy};

    #[test]
    fn crud_policy_allows_publisher_user_for_write() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let org_id = publisher
            .zitadel_id
            .clone()
            .expect("publisher missing zitadel id");
        let user = test_user_with_role("contact-user", Role::PublisherUser, &org_id);
        let ctx = test_context_with_user(pool.clone(), user);

        let new_contact = make_new_contact(
            publisher.publisher_id,
            ContactType::Accessibility,
            "policy@example.com",
        );

        let contact = Contact::create(pool.as_ref(), &new_contact).expect("Failed to create");
        let patch = make_patch_contact(&contact, contact.contact_type, "policy-update@example.com");

        assert!(ContactPolicy::can_create(&ctx, &new_contact, ()).is_ok());
        assert!(ContactPolicy::can_update(&ctx, &contact, &patch, ()).is_ok());
        assert!(ContactPolicy::can_delete(&ctx, &contact).is_ok());
    }

    #[test]
    fn crud_policy_rejects_user_without_publisher_role() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let contact = make_contact(
            pool.as_ref(),
            publisher.publisher_id,
            format!("policy-{}@example.com", Uuid::new_v4()),
        );
        let patch = make_patch_contact(&contact, contact.contact_type, "blocked@example.com");

        let user = test_user_with_role("contact-user", Role::PublisherUser, "org-other");
        let ctx = test_context_with_user(pool.clone(), user);

        let new_contact = make_new_contact(
            publisher.publisher_id,
            ContactType::Accessibility,
            "policy@example.com",
        );

        assert!(ContactPolicy::can_create(&ctx, &new_contact, ()).is_err());
        assert!(ContactPolicy::can_update(&ctx, &contact, &patch, ()).is_err());
        assert!(ContactPolicy::can_delete(&ctx, &contact).is_err());
    }
}

#[cfg(feature = "backend")]
mod crud {
    use super::*;

    use crate::model::tests::db::{create_publisher, setup_test_db, test_context};
    use crate::model::Crud;

    #[test]
    fn crud_roundtrip_create_fetch_update_delete() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let new_contact = make_new_contact(
            publisher.publisher_id,
            ContactType::Accessibility,
            "test@example.com",
        );

        let contact = Contact::create(pool.as_ref(), &new_contact).expect("Failed to create");
        let fetched =
            Contact::from_id(pool.as_ref(), &contact.contact_id).expect("Failed to fetch");
        assert_eq!(contact.contact_id, fetched.contact_id);

        let patch = make_patch_contact(&contact, contact.contact_type, "updated@example.com");

        let ctx = test_context(pool.clone(), "test-user");
        let updated = contact.update(&ctx, &patch).expect("Failed to update");
        assert_eq!(updated.email, patch.email);

        let deleted = updated.delete(pool.as_ref()).expect("Failed to delete");
        assert!(Contact::from_id(pool.as_ref(), &deleted.contact_id).is_err());
    }

    #[test]
    fn crud_all_respects_limit_and_offset() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let other_publisher = create_publisher(pool.as_ref());
        make_contact(
            pool.as_ref(),
            publisher.publisher_id,
            format!("first-{}@example.com", Uuid::new_v4()),
        );
        make_contact(
            pool.as_ref(),
            other_publisher.publisher_id,
            format!("second-{}@example.com", Uuid::new_v4()),
        );

        let order = ContactOrderBy {
            field: ContactField::ContactId,
            direction: Direction::Asc,
        };

        let first = Contact::all(
            pool.as_ref(),
            1,
            0,
            None,
            order.clone(),
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to fetch contacts");
        let second = Contact::all(
            pool.as_ref(),
            1,
            1,
            None,
            order,
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to fetch contacts");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].contact_id, second[0].contact_id);
    }

    #[test]
    fn crud_count_returns_total() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let other_publisher = create_publisher(pool.as_ref());
        make_contact(
            pool.as_ref(),
            publisher.publisher_id,
            format!("first-{}@example.com", Uuid::new_v4()),
        );
        make_contact(
            pool.as_ref(),
            other_publisher.publisher_id,
            format!("second-{}@example.com", Uuid::new_v4()),
        );

        let count = Contact::count(pool.as_ref(), None, vec![], vec![], vec![], None, None)
            .expect("Failed to count contacts");
        assert_eq!(count, 2);
    }

    #[test]
    fn crud_filter_parent_publisher_id_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let other_publisher = create_publisher(pool.as_ref());
        let matches = make_contact(
            pool.as_ref(),
            publisher.publisher_id,
            format!("match-{}@example.com", Uuid::new_v4()),
        );
        make_contact(
            pool.as_ref(),
            other_publisher.publisher_id,
            format!("other-{}@example.com", Uuid::new_v4()),
        );

        let filtered = Contact::all(
            pool.as_ref(),
            10,
            0,
            None,
            ContactOrderBy {
                field: ContactField::ContactId,
                direction: Direction::Asc,
            },
            vec![],
            Some(publisher.publisher_id),
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter contacts by publisher");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].contact_id, matches.contact_id);
    }

    #[test]
    fn crud_ordering_by_id_respects_direction() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let other_publisher = create_publisher(pool.as_ref());
        let first = make_contact(
            pool.as_ref(),
            publisher.publisher_id,
            format!("first-{}@example.com", Uuid::new_v4()),
        );
        let second = make_contact(
            pool.as_ref(),
            other_publisher.publisher_id,
            format!("second-{}@example.com", Uuid::new_v4()),
        );
        let mut ids = [first.contact_id, second.contact_id];
        ids.sort();

        let asc = Contact::all(
            pool.as_ref(),
            2,
            0,
            None,
            ContactOrderBy {
                field: ContactField::ContactId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to order contacts (asc)");

        let desc = Contact::all(
            pool.as_ref(),
            2,
            0,
            None,
            ContactOrderBy {
                field: ContactField::ContactId,
                direction: Direction::Desc,
            },
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to order contacts (desc)");

        assert_eq!(asc[0].contact_id, ids[0]);
        assert_eq!(desc[0].contact_id, ids[1]);
    }

    #[test]
    fn crud_filter_publishers_limits_results() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let other_publisher = create_publisher(pool.as_ref());
        let matches = make_contact(
            pool.as_ref(),
            publisher.publisher_id,
            format!("match-{}@example.com", Uuid::new_v4()),
        );
        make_contact(
            pool.as_ref(),
            other_publisher.publisher_id,
            format!("other-{}@example.com", Uuid::new_v4()),
        );

        let filtered = Contact::all(
            pool.as_ref(),
            10,
            0,
            None,
            ContactOrderBy {
                field: ContactField::ContactId,
                direction: Direction::Asc,
            },
            vec![publisher.publisher_id],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter contacts by publishers");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].contact_id, matches.contact_id);
    }

    #[test]
    fn crud_filter_param_limits_contact_types() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let other_publisher = create_publisher(pool.as_ref());
        let matches = Contact::create(
            pool.as_ref(),
            &make_new_contact(
                publisher.publisher_id,
                ContactType::Accessibility,
                format!("access-{}@example.com", Uuid::new_v4()),
            ),
        )
        .expect("Failed to create contact");
        Contact::create(
            pool.as_ref(),
            &make_new_contact(
                other_publisher.publisher_id,
                ContactType::Accessibility,
                format!("request-{}@example.com", Uuid::new_v4()),
            ),
        )
        .expect("Failed to create contact");

        let filtered = Contact::all(
            pool.as_ref(),
            10,
            0,
            None,
            ContactOrderBy {
                field: ContactField::ContactId,
                direction: Direction::Asc,
            },
            vec![],
            None,
            None,
            vec![ContactType::Accessibility],
            vec![],
            None,
            None,
        )
        .expect("Failed to filter contacts by type");

        assert_eq!(filtered.len(), 2);
        assert!(filtered
            .iter()
            .any(|contact| contact.contact_id == matches.contact_id));
    }

    #[test]
    fn crud_count_with_filter_matches_contact_type() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let other_publisher = create_publisher(pool.as_ref());
        Contact::create(
            pool.as_ref(),
            &make_new_contact(
                publisher.publisher_id,
                ContactType::Accessibility,
                format!("access-{}@example.com", Uuid::new_v4()),
            ),
        )
        .expect("Failed to create contact");
        Contact::create(
            pool.as_ref(),
            &make_new_contact(
                other_publisher.publisher_id,
                ContactType::Accessibility,
                format!("request-{}@example.com", Uuid::new_v4()),
            ),
        )
        .expect("Failed to create contact");

        let count = Contact::count(
            pool.as_ref(),
            None,
            vec![],
            vec![ContactType::Accessibility],
            vec![],
            None,
            None,
        )
        .expect("Failed to count contacts by type");

        assert_eq!(count, 2);
    }

    #[test]
    fn crud_ordering_by_fields_is_supported() {
        let (_guard, pool) = setup_test_db();

        let publisher = create_publisher(pool.as_ref());
        let other_publisher = create_publisher(pool.as_ref());
        Contact::create(
            pool.as_ref(),
            &make_new_contact(
                publisher.publisher_id,
                ContactType::Accessibility,
                "a@example.com",
            ),
        )
        .expect("Failed to create contact");
        Contact::create(
            pool.as_ref(),
            &make_new_contact(
                other_publisher.publisher_id,
                ContactType::Accessibility,
                "b@example.com",
            ),
        )
        .expect("Failed to create contact");

        let fields: Vec<fn() -> ContactField> = vec![
            || ContactField::ContactId,
            || ContactField::PublisherId,
            || ContactField::ContactType,
            || ContactField::Email,
            || ContactField::CreatedAt,
            || ContactField::UpdatedAt,
        ];

        for field in fields {
            for direction in [Direction::Asc, Direction::Desc] {
                let results = Contact::all(
                    pool.as_ref(),
                    10,
                    0,
                    None,
                    ContactOrderBy {
                        field: field(),
                        direction,
                    },
                    vec![],
                    None,
                    None,
                    vec![],
                    vec![],
                    None,
                    None,
                )
                .expect("Failed to order contacts");

                assert_eq!(results.len(), 2);
            }
        }
    }
}
