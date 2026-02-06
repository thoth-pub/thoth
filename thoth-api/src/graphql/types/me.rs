use crate::graphql::Context;
use crate::model::publisher::{Publisher, PublisherOrderBy};
use crate::model::Crud;
use crate::policy::{PublisherPermissions, UserAccess};
use juniper::FieldResult;
use zitadel::actix::introspection::IntrospectedUser;

pub struct Me {
    pub user_id: String,
    pub email: Option<String>,
    pub is_superuser: bool,
    pub publisher_contexts: Vec<PublisherContext>,
}

#[derive(Clone)]
pub struct PublisherContext {
    pub publisher: Publisher,
    pub permissions: PublisherPermissions,
}

pub trait ToMe {
    fn to_me(&self, context: &Context) -> FieldResult<Me>;
}

impl ToMe for IntrospectedUser {
    fn to_me(&self, context: &Context) -> FieldResult<Me> {
        let is_superuser = self.is_superuser();
        let mut publisher_contexts = publisher_contexts_for_user(context, self)?;
        publisher_contexts
            .sort_by(|a, b| a.publisher.publisher_name.cmp(&b.publisher.publisher_name));

        Ok(Me {
            user_id: self.user_id.clone(),
            email: self.email.clone(),
            is_superuser,
            publisher_contexts,
        })
    }
}

fn publisher_contexts_for_user(
    context: &Context,
    user: &IntrospectedUser,
) -> FieldResult<Vec<PublisherContext>> {
    if user.is_superuser() {
        let publishers = Publisher::all(
            &context.db,
            i32::MAX,
            0,
            None,
            PublisherOrderBy::default(),
            vec![],
            None,
            None,
            vec![],
            vec![],
            None,
            None,
        )?;

        return Ok(publishers
            .into_iter()
            .map(|publisher| {
                let permissions = publisher
                    .zitadel_id
                    .as_deref()
                    .map(|org_id| user.permissions_for_org(org_id))
                    .unwrap_or_else(PublisherPermissions::for_superuser);
                PublisherContext {
                    publisher,
                    permissions,
                }
            })
            .collect());
    }

    let org_ids = user.publisher_org_ids();
    if org_ids.is_empty() {
        return Ok(Vec::new());
    }

    let publishers = Publisher::by_zitadel_ids(&context.db, org_ids)?;
    Ok(publishers
        .into_iter()
        .filter_map(|publisher| {
            let org_id = publisher.zitadel_id.as_deref()?.to_string();
            Some(PublisherContext {
                publisher,
                permissions: user.permissions_for_org(&org_id),
            })
        })
        .collect())
}

#[juniper::graphql_object(Context = Context)]
impl Me {
    fn user_id(&self) -> &str {
        &self.user_id
    }

    fn email(&self) -> Option<&String> {
        self.email.as_ref()
    }

    fn is_superuser(&self) -> bool {
        self.is_superuser
    }

    fn publisher_contexts(&self) -> Vec<PublisherContext> {
        self.publisher_contexts.clone()
    }
}

#[juniper::graphql_object(Context = Context)]
impl PublisherContext {
    fn publisher(&self) -> &Publisher {
        &self.publisher
    }

    fn permissions(&self) -> PublisherPermissions {
        self.permissions
    }
}

#[juniper::graphql_object(Context = Context)]
impl PublisherPermissions {
    fn publisher_admin(&self) -> bool {
        self.publisher_admin
    }

    fn work_lifecycle(&self) -> bool {
        self.work_lifecycle
    }

    fn cdn_write(&self) -> bool {
        self.cdn_write
    }
}
