use diesel::prelude::*;
use juniper::RootNode;
use uuid::Uuid;
use chrono::naive::NaiveDate;

use crate::db::PgPool;
use crate::schema::*;
use crate::models::publisher::*;
use crate::models::work::*;
use crate::models::language::*;
use crate::models::series::*;
use crate::models::contributor::*;
use crate::models::publication::*;
use crate::models::price::*;
use crate::models::subject::*;
use crate::models::funder::*;

#[derive(Clone)]
pub struct Context {
  pub db: PgPool,
}

impl juniper::Context for Context {}

pub struct QueryRoot;

#[juniper::object(Context = Context)]
impl QueryRoot {
  fn works(context: &Context) -> Vec<Work> {
    use crate::schema::work::dsl::*;
    let connection = context.db.get().unwrap();
    work
      .limit(100)
      .load::<Work>(&connection)
      .expect("Error loading works")
  }

  fn publications(context: &Context) -> Vec<Publication> {
    use crate::schema::publication::dsl::*;
    let connection = context.db.get().unwrap();
    publication
      .limit(100)
      .load::<Publication>(&connection)
      .expect("Error loading publications")
  }

  fn publishers(context: &Context) -> Vec<Publisher> {
    use crate::schema::publisher::dsl::*;
    let connection = context.db.get().unwrap();
    publisher
      .limit(100)
      .load::<Publisher>(&connection)
      .expect("Error loading publishers")
  }

  fn imprints(context: &Context) -> Vec<Imprint> {
    use crate::schema::imprint::dsl::*;
    let connection = context.db.get().unwrap();
    imprint
      .limit(100)
      .load::<Imprint>(&connection)
      .expect("Error loading imprints")
  }

  fn contributors(context: &Context) -> Vec<Contributor> {
    use crate::schema::contributor::dsl::*;
    let connection = context.db.get().unwrap();
    contributor
        .limit(100)
        .load::<Contributor>(&connection)
        .expect("Error loading contributors")
  }

  fn series(context: &Context) -> Vec<Series> {
    use crate::schema::series::dsl::*;
    let connection = context.db.get().unwrap();
    series
        .limit(100)
        .load::<Series>(&connection)
        .expect("Error loading series")
  }

  fn issues(context: &Context) -> Vec<Issue> {
    use crate::schema::issue::dsl::*;
    let connection = context.db.get().unwrap();
    issue
        .limit(100)
        .load::<Issue>(&connection)
        .expect("Error loading issues")
  }

  fn languages(context: &Context) -> Vec<Language> {
    use crate::schema::language::dsl::*;
    let connection = context.db.get().unwrap();
    language
        .limit(100)
        .load::<Language>(&connection)
        .expect("Error loading languages")
  }

  fn prices(context: &Context) -> Vec<Price> {
    use crate::schema::price::dsl::*;
    let connection = context.db.get().unwrap();
    price
        .limit(100)
        .load::<Price>(&connection)
        .expect("Error loading prices")
  }

  fn subjects(context: &Context) -> Vec<Subject> {
    use crate::schema::subject::dsl::*;
    let connection = context.db.get().unwrap();
    subject
        .limit(100)
        .load::<Subject>(&connection)
        .expect("Error loading subjects")
  }

  fn funders(context: &Context) -> Vec<Funder> {
    use crate::schema::funder::dsl::*;
    let connection = context.db.get().unwrap();
    funder
        .limit(100)
        .load::<Funder>(&connection)
        .expect("Error loading funders")
  }

  fn funders(context: &Context) -> Vec<Funding> {
    use crate::schema::funding::dsl::*;
    let connection = context.db.get().unwrap();
    funding
        .limit(100)
        .load::<Funding>(&connection)
        .expect("Error loading fundings")
  }
}

pub struct MutationRoot;

#[juniper::object(Context = Context)]
impl MutationRoot {
  fn create_work(context: &Context, data: NewWork) -> Work {
    let connection = context.db.get().unwrap();
    diesel::insert_into(work::table)
      .values(&data)
      .get_result(&connection)
      .expect("Error saving new work")
  }

  fn create_publisher(context: &Context, data: NewPublisher) -> Publisher {
    let connection = context.db.get().unwrap();
    diesel::insert_into(publisher::table)
      .values(&data)
      .get_result(&connection)
      .expect("Error saving new publisher")
  }

  fn create_imprint(context: &Context, data: NewImprint) -> Imprint {
    let connection = context.db.get().unwrap();
    diesel::insert_into(imprint::table)
      .values(&data)
      .get_result(&connection)
      .expect("Error saving new imprint")
  }

  fn create_contributor(
      context: &Context,
      data: NewContributor
  ) -> Contributor {
    let connection = context.db.get().unwrap();
    diesel::insert_into(contributor::table)
        .values(&data)
        .get_result(&connection)
        .expect("Error saving new contributor")
  }

  fn create_contribution(
      context: &Context,
      data: NewContribution
  ) -> Contribution {
    let connection = context.db.get().unwrap();
    diesel::insert_into(contribution::table)
        .values(&data)
        .get_result(&connection)
        .expect("Error saving new contribution")
  }

  fn create_publication(
      context: &Context,
      data: NewPublication
  ) -> Publication {
    let connection = context.db.get().unwrap();
    diesel::insert_into(publication::table)
      .values(&data)
      .get_result(&connection)
      .expect("Error saving new publication")
  }

  fn create_series(context: &Context, data: NewSeries) -> Series {
    let connection = context.db.get().unwrap();
    diesel::insert_into(series::table)
      .values(&data)
      .get_result(&connection)
      .expect("Error saving new series")
  }

  fn create_issue(context: &Context, data: NewIssue) -> Issue {
    let connection = context.db.get().unwrap();
    diesel::insert_into(issue::table)
      .values(&data)
      .get_result(&connection)
      .expect("Error saving new issue")
  }

  fn create_language(context: &Context, data: NewLanguage) -> Language {
    let connection = context.db.get().unwrap();
    diesel::insert_into(language::table)
      .values(&data)
      .get_result(&connection)
      .expect("Error saving new language")
  }

  fn create_funder(context: &Context, data: NewFunder) -> Funder {
    let connection = context.db.get().unwrap();
    diesel::insert_into(funder::table)
      .values(&data)
      .get_result(&connection)
      .expect("Error saving new funder")
  }

  fn create_funding(context: &Context, data: NewFunding) -> Funding {
    let connection = context.db.get().unwrap();
    diesel::insert_into(funding::table)
      .values(&data)
      .get_result(&connection)
      .expect("Error saving new funding")
  }

  fn create_funding(context: &Context, data: NewFunding) -> Funding {
    let connection = context.db.get().unwrap();
    diesel::insert_into(funding::table)
      .values(&data)
      .get_result(&connection)
      .expect("Error saving new funding")
  }

  fn create_price(context: &Context, data: NewPrice) -> Price {
    let connection = context.db.get().unwrap();
    diesel::insert_into(price::table)
      .values(&data)
      .get_result(&connection)
      .expect("Error saving new price")
  }

  fn create_subject(context: &Context, data: NewSubject) -> Subject {
    check_subject(&data.subject_type, &data.subject_code)
        .expect(&format!("{} is not a valid {} code",
                data.subject_code, data.subject_type.to_string()));

    let connection = context.db.get().unwrap();
    diesel::insert_into(subject::table)
      .values(&data)
      .get_result(&connection)
      .expect("Error saving new subject")
  }
}

#[juniper::object(Context = Context, description = "A written text that can be published")]
impl Work {
    pub fn work_id(&self) -> &Uuid {
        &self.work_id
    }

    pub fn work_type(&self) -> &WorkType {
        &self.work_type
    }

    pub fn work_status(&self) -> &WorkStatus {
        &self.work_status
    }

    #[graphql(description="Concatenation of title and subtitle with punctuation mark")]
    pub fn full_title(&self) -> &str {
        self.full_title.as_str()
    }

    #[graphql(description="Main title of the work (excluding subtitle)")]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    #[graphql(description="Secondary title of the work (excluding main title)")]
    pub fn subtitle(&self) -> Option<&String> {
        self.subtitle.as_ref()
    }

    #[graphql(description="Internal reference code")]
    pub fn reference(&self) -> Option<&String> {
        self.reference.as_ref()
    }

    pub fn edition(&self) -> &i32 {
        &self.edition
    }

    #[graphql(description="Digital Object Identifier of the work as full URL. It must use the HTTPS scheme and the doi.org domain (e.g. https://doi.org/10.11647/obp.0001)")]
    pub fn doi(&self) -> Option<&String> {
        self.doi.as_ref()
    }

    pub fn publication_date(&self) -> Option<NaiveDate> {
        self.publication_date
    }

    pub fn place(&self) -> Option<&String> {
        self.place.as_ref()
    }

    pub fn width(&self) -> Option<&i32> {
        self.width.as_ref()
    }

    pub fn height(&self) -> Option<&i32> {
        self.height.as_ref()
    }

    pub fn page_count(&self) -> Option<&i32> {
        self.page_count.as_ref()
    }

    pub fn page_breakdown(&self) -> Option<&String> {
        self.page_breakdown.as_ref()
    }

    pub fn image_count(&self) -> Option<&i32> {
        self.image_count.as_ref()
    }

    pub fn table_count(&self) -> Option<&i32> {
        self.table_count.as_ref()
    }

    pub fn audio_count(&self) -> Option<&i32> {
        self.audio_count.as_ref()
    }

    pub fn video_count(&self) -> Option<&i32> {
        self.video_count.as_ref()
    }

    pub fn license(&self) -> Option<&String> {
        self.license.as_ref()
    }

    pub fn copyright_holder(&self) -> &str {
        self.copyright_holder.as_str()
    }

    pub fn landing_page(&self) -> Option<&String> {
        self.landing_page.as_ref()
    }

    pub fn lccn(&self) -> Option<&i32> {
        self.lccn.as_ref()
    }

    pub fn oclc(&self) -> Option<&i32> {
        self.oclc.as_ref()
    }

    pub fn short_abstract(&self) -> Option<&String> {
        self.short_abstract.as_ref()
    }

    pub fn long_abstract(&self) -> Option<&String> {
        self.long_abstract.as_ref()
    }

    pub fn general_note(&self) -> Option<&String> {
        self.general_note.as_ref()
    }

    pub fn toc(&self) -> Option<&String> {
        self.toc.as_ref()
    }

    pub fn cover_url(&self) -> Option<&String> {
        self.cover_url.as_ref()
    }

    pub fn cover_caption(&self) -> Option<&String> {
        self.cover_caption.as_ref()
    }

    pub fn imprint(&self, context: &Context) -> Imprint {
        use crate::schema::imprint::dsl::*;
        let connection = context.db.get().unwrap();
        imprint
            .find(self.imprint_id)
            .first(&connection)
            .expect("Error loading imprint")
    }

    pub fn contributions(&self, context: &Context) -> Vec<Contribution> {
        use crate::schema::contribution::dsl::*;
        let connection = context.db.get().unwrap();
        contribution
            .filter(work_id.eq(self.work_id))
            .load::<Contribution>(&connection)
            .expect("Error loading contributions")
    }

    pub fn languages(&self, context: &Context) -> Vec<Language> {
        use crate::schema::language::dsl::*;
        let connection = context.db.get().unwrap();
        language
            .filter(work_id.eq(self.work_id))
            .load::<Language>(&connection)
            .expect("Error loading languages")
    }

    pub fn publications(&self, context: &Context) -> Vec<Publication> {
        use crate::schema::publication::dsl::*;
        let connection = context.db.get().unwrap();
        publication
            .filter(work_id.eq(self.work_id))
            .load::<Publication>(&connection)
            .expect("Error loading publications")
    }

    pub fn subjects(&self, context: &Context) -> Vec<Subject> {
        use crate::schema::subject::dsl::*;
        let connection = context.db.get().unwrap();
        subject
            .filter(work_id.eq(self.work_id))
            .load::<Subject>(&connection)
            .expect("Error loading subjects")
    }

    pub fn issues(&self, context: &Context) -> Vec<Issue> {
        use crate::schema::issue::dsl::*;
        let connection = context.db.get().unwrap();
        issue
            .filter(work_id.eq(self.work_id))
            .load::<Issue>(&connection)
            .expect("Error loading issues")
    }
}

#[juniper::object(Context = Context, description = "A manifestation of a written text")]
impl Publication {
    pub fn publication_id(&self) -> Uuid {
        self.publication_id
    }

    pub fn publication_type(&self) -> &PublicationType {
        &self.publication_type
    }

    pub fn isbn(&self) -> Option<&String> {
        self.isbn.as_ref()
    }

    pub fn publication_url(&self) -> Option<&String> {
        self.publication_url.as_ref()
    }

    pub fn prices(&self, context: &Context) -> Vec<Price> {
        use crate::schema::price::dsl::*;
        let connection = context.db.get().unwrap();
        price
            .filter(publication_id.eq(self.publication_id))
            .load::<Price>(&connection)
            .expect("Error loading price")
    }

    pub fn work(&self, context: &Context) -> Work {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work
            .find(self.work_id)
            .first(&connection)
            .expect("Error loading work")
    }
}

#[juniper::object(Context = Context, description = "An organisation that produces and distributes written texts.")]
impl Publisher {
    pub fn publisher_id(&self) -> Uuid {
        self.publisher_id
    }

    pub fn publisher_name(&self) -> &String {
        &self.publisher_name
    }

    pub fn publisher_shortname(&self) -> Option<&String> {
        self.publisher_shortname.as_ref()
    }

    pub fn publisher_url(&self) -> Option<&String> {
        self.publisher_url.as_ref()
    }

    pub fn imprints(&self, context: &Context) -> Vec<Imprint> {
        use crate::schema::imprint::dsl::*;
        let connection = context.db.get().unwrap();
        imprint
            .filter(publisher_id.eq(self.publisher_id))
            .load::<Imprint>(&connection)
            .expect("Error loading imprints")
    }
}

#[juniper::object(Context = Context, description = "The brand under which a publisher issues works.")]
impl Imprint {
    pub fn imprint_id(&self) -> Uuid {
        self.imprint_id
    }

    pub fn imprint_name(&self) -> &String {
        &self.imprint_name
    }

    pub fn imprint_url(&self) -> Option<&String> {
        self.imprint_url.as_ref()
    }

    pub fn publisher(&self, context: &Context) -> Publisher {
        use crate::schema::publisher::dsl::*;
        let connection = context.db.get().unwrap();
        publisher
            .find(self.publisher_id)
            .first(&connection)
            .expect("Error loading publisher")
    }

    pub fn works(&self, context: &Context) -> Vec<Work> {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work
            .filter(imprint_id.eq(self.imprint_id))
            .load::<Work>(&connection)
            .expect("Error loading works")
    }
}

#[juniper::object(Context = Context, description = "A person who has been involved in the production of a written text.")]
impl Contributor {
    pub fn contributor_id(&self) -> Uuid {
        self.contributor_id
    }

    pub fn first_name(&self) -> Option<&String> {
        self.first_name.as_ref()
    }

    pub fn last_name(&self) -> &String {
        &self.last_name
    }

    pub fn full_name(&self) -> &String {
        &self.full_name
    }

    pub fn orcid(&self) -> Option<&String> {
        self.orcid.as_ref()
    }

    pub fn website(&self) -> Option<&String> {
        self.website.as_ref()
    }

    pub fn contributions(&self, context: &Context) -> Vec<Contribution> {
        use crate::schema::contribution::dsl::*;
        let connection = context.db.get().unwrap();
        contribution
            .filter(contributor_id.eq(self.contributor_id))
            .load::<Contribution>(&connection)
            .expect("Error loading contributions")
    }
}

#[juniper::object(Context = Context, description = "A person's involvement in the production of a written text.")]
impl Contribution {
    pub fn contributor_id(&self) -> Uuid {
        self.contributor_id
    }

    pub fn work_id(&self) -> Uuid {
        self.work_id
    }

    pub fn contribution_type(&self) -> &ContributionType {
        &self.contribution_type
    }

    pub fn main_contribution(&self) -> bool{
        self.main_contribution
    }

    pub fn biography(&self) -> Option<&String> {
        self.biography.as_ref()
    }

    pub fn institution(&self) -> Option<&String> {
        self.institution.as_ref()
    }

    pub fn work(&self, context: &Context) -> Work {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work
            .find(self.work_id)
            .first(&connection)
            .expect("Error loading work")
    }

    pub fn contributor(&self, context: &Context) -> Contributor {
        use crate::schema::contributor::dsl::*;
        let connection = context.db.get().unwrap();
        contributor
            .find(self.contributor_id)
            .first(&connection)
            .expect("Error loading contributions")
    }
}

#[juniper::object(Context = Context, description = "A periodical of publications about a particular subject.")]
impl Series {
    pub fn series_id(&self) -> Uuid {
        self.series_id
    }

    pub fn series_type(&self) -> &SeriesType {
        &self.series_type
    }

    pub fn series_name(&self) -> &String {
        &self.series_name
    }

    pub fn issn_print(&self) -> &String {
        &self.issn_print
    }

    pub fn issn_digital(&self) -> &String {
        &self.issn_digital
    }

    pub fn series_url(&self) -> Option<&String> {
        self.series_url.as_ref()
    }

    pub fn imprint(&self, context: &Context) -> Imprint {
        use crate::schema::imprint::dsl::*;
        let connection = context.db.get().unwrap();
        imprint
            .find(self.imprint_id)
            .first(&connection)
            .expect("Error loading imprint")
    }

    pub fn issues(&self, context: &Context) -> Vec<Issue> {
        use crate::schema::issue::dsl::*;
        let connection = context.db.get().unwrap();
        issue
            .filter(series_id.eq(self.series_id))
            .load::<Issue>(&connection)
            .expect("Error loading issues")
    }
}

#[juniper::object(Context = Context, description = "A work published as a number in a periodical.")]
impl Issue {
    pub fn issue_ordinal(&self) -> &i32 {
        &self.issue_ordinal
    }

    pub fn series(&self, context: &Context) -> Series {
        use crate::schema::series::dsl::*;
        let connection = context.db.get().unwrap();
        series
            .find(self.series_id)
            .first(&connection)
            .expect("Error loading series")
    }

    pub fn work(&self, context: &Context) -> Work {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work
            .find(self.work_id)
            .first(&connection)
            .expect("Error loading work")
    }
}

#[juniper::object(Context = Context, description = "Description of a work's language.")]
impl Language {
    pub fn language_id(&self) -> Uuid {
        self.language_id
    }

    pub fn language_code(&self) -> &LanguageCode {
        &self.language_code
    }

    pub fn language_relation(&self) -> &LanguageRelation {
        &self.language_relation
    }

    pub fn main_language(&self) -> bool{
        self.main_language
    }

    pub fn work(&self, context: &Context) -> Work {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work
            .find(self.work_id)
            .first(&connection)
            .expect("Error loading work")
    }
}

#[juniper::object(Context = Context, description = "The amount of money, in any currency, that a publication costs.")]
impl Price {
    pub fn price_id(&self) -> Uuid {
        self.price_id
    }

    pub fn currency_code(&self) -> &CurrencyCode {
        &self.currency_code
    }

    pub fn unit_price(&self) -> f64 {
        self.unit_price
    }

    pub fn publication(&self, context: &Context) -> Publication {
        use crate::schema::publication::dsl::*;
        let connection = context.db.get().unwrap();
        publication
            .find(self.publication_id)
            .first(&connection)
            .expect("Error loading publication")
    }
}

#[juniper::object(Context = Context, description = "A significant discipline or term related to a work.")]
impl Subject {
    pub fn subject_id(&self) -> &Uuid {
        &self.subject_id
    }

    pub fn subject_type(&self) -> &SubjectType {
        &self.subject_type
    }

    pub fn subject_code(&self) -> &String {
        &self.subject_code
    }

    pub fn subject_ordinal(&self) -> &i32 {
        &self.subject_ordinal
    }

    pub fn work(&self, context: &Context) -> Work {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work
            .find(self.work_id)
            .first(&connection)
            .expect("Error loading work")
    }
}

#[juniper::object(Context = Context, description = "An organisation that provides the money to pay for the publication of a work.")]
impl Funder {
    pub fn funder_id(&self) -> &Uuid {
        &self.funder_id
    }

    pub fn funder_name(&self) -> &String {
        &self.funder_name
    }

    pub fn funder_doi(&self) -> Option<&String> {
        self.funder_doi.as_ref()
    }

    pub fn fundings(&self, context: &Context) -> Vec<Funding> {
        use crate::schema::funding::dsl::*;
        let connection = context.db.get().unwrap();
        funding
            .filter(funder_id.eq(self.funder_id))
            .load::<Funding>(&connection)
            .expect("Error loading fundings")
    }
}

#[juniper::object(Context = Context, description = "A grant awarded to the publication of a work by a funder.")]
impl Funding {
    pub fn funding_id(&self) -> &Uuid {
        &self.funding_id
    }

    pub fn program(&self) -> Option<&String> {
        self.program.as_ref()
    }

    pub fn project_name(&self) -> Option<&String> {
        self.project_name.as_ref()
    }

    pub fn project_shortname(&self) -> Option<&String> {
        self.project_shortname.as_ref()
    }

    pub fn grant_number(&self) -> Option<&String> {
        self.grant_number.as_ref()
    }

    pub fn jurisdiction(&self) -> Option<&String> {
        self.jurisdiction.as_ref()
    }

    pub fn work(&self, context: &Context) -> Work {
        use crate::schema::work::dsl::*;
        let connection = context.db.get().unwrap();
        work
            .find(self.work_id)
            .first(&connection)
            .expect("Error loading work")
    }

    pub fn funder(&self, context: &Context) -> Funder {
        use crate::schema::funder::dsl::*;
        let connection = context.db.get().unwrap();
        funder
            .find(self.funder_id)
            .first(&connection)
            .expect("Error loading funder")
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
  Schema::new(QueryRoot {}, MutationRoot {})
}
