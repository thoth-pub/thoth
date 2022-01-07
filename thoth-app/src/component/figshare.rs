use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thoth_api::model::subject::SubjectType;
use thoth_api::model::work::WorkType;
use thoth_api::model::work::WorkWithRelations;
use yew::html;
use yew::prelude::*;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchRequest;
use yewtil::fetch::Json;
use yewtil::fetch::MethodBody;
use yewtil::future::LinkFuture;
use yewtil::NeqAssign;

// Test instance. Production instance is "https://api.figshare.com/v2".
const FIGSHARE_API_ROOT: &str = "https://api.figsh.com/v2";

// Authorization token associated with a Figshare user account.
// The token itself is security information and must not be published in open-source code.
// Instead, set it as an environment variable in the shell before starting the Thoth app
// (`export FIGSHARE_TOKEN=[value]`).
const FIGSHARE_TOKEN: Option<&str> = option_env!("FIGSHARE_TOKEN");

// Temporary hard-coding of single Figshare article ID for basic test purposes.
// If required, set it as an environment variable, as above for FIGSHARE_TOKEN.
const TEST_ARTICLE_ID: Option<&str> = option_env!("FIGSHARE_ARTICLE_ID");

// Child object of ArticleCreate representing an author.
// Note that this will be transformed in the created article into an Author object
// (with attributes id, full_name, is_active, url_name and orcid_id).
// url_name will default to "_" if no valid Figshare author ID is supplied.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]

pub struct FigArticleCreateAuthor {
    // This information (Figshare author ID) is not stored in Thoth.
    // pub id: String,
    // pub first_name: String,
    // pub last_name: String,
    pub name: String,
    // pub email: String,
    // This information is stored in Thoth but not currently accessible via the Work page.
    // pub orcid_id: String,
}

// This will be transformed on creation into a FundingInformation object
// (with attributes id, title, grant_code, funder_name, is_user_defined, url).
// Thoth stores information such as grant number and funder (institution) name
// but these cannot be submitted here.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigFundingCreate {
    // This appears to be a Figshare funding ID and is not stored in Thoth.
    // pub id: String,
    // Defined as "the funding name"; Thoth stores program, project name, etc.
    pub title: String,
}

// Note: once a timeline has been created, it does not seem to be possible
// to remove it (submitting empty attribute strings and empty
// TimelineUpdate objects both had no effect).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FigTimelineUpdate {
    // pub first_online: String,
    // Omit this attribute if no publication date exists (i.e. create empty object).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher_publication: Option<String>,
    // pub publisher_acceptance: String,
}

// Can also be used to represent ArticleUpdate, as the objects are identical.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigArticleCreate {
    // Required fields for article creation:
    pub title: String,
    // Required fields for article publication:
    pub description: String,
    pub authors: Vec<FigArticleCreateAuthor>,
    // Figshare IDs representing ANZSRC FoR categories - TBD how to map to Thoth categories
    // pub categories: Vec<i32>,
    pub defined_type: String,
    // Transformed into "tags" on creation - consider renaming
    pub keywords: Vec<String>,
    // Figshare ID - TODO retrieve options from private licences endpoint,
    // match option URL to licence URL stored in Thoth, submit corresponding ID.
    // pub license: i32,
    // (A subset of) optional fields:
    pub funding_list: Vec<FigFundingCreate>,
    pub timeline: FigTimelineUpdate,
    pub resource_doi: String,
}

#[derive(Debug, Clone, Default)]
pub struct FigArticleUpdateRequest {
    pub body: FigArticleCreate,
}

// Standard Figshare response to API request (article create/update)
// appears to consist of "location" (of article) and "warnings";
// however, error responses seem to contain "message" and "code" instead.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigshareResponseBody {
    pub location: String,
    pub warnings: Vec<String>,
}

// Implement Yewtil's example template for reducing HTTP request boilerplate
// (see documentation for FetchRequest)
pub trait SlimFetchRequest {
    type RequestBody: Serialize;
    type ResponseBody: DeserializeOwned;
    fn path(&self) -> String;
    fn method(&self) -> MethodBody<Self::RequestBody>;
}

#[derive(Default)]
pub struct FetchWrapper<T>(T);

impl<T: SlimFetchRequest> FetchRequest for FetchWrapper<T> {
    type RequestBody = T::RequestBody;
    type ResponseBody = T::ResponseBody;
    type Format = Json;

    fn url(&self) -> String {
        format!("{}{}", FIGSHARE_API_ROOT, self.0.path())
    }

    fn method(&self) -> MethodBody<Self::RequestBody> {
        self.0.method()
    }

    // Write requests require authentication information and a JSON body containing the data to be written.
    fn headers(&self) -> Vec<(String, String)> {
        let json = ("Content-Type".into(), "application/json".into());
        let auth = (
            "Authorization".into(),
            format!("token {}", FIGSHARE_TOKEN.unwrap()),
        );
        vec![json, auth]
    }

    fn use_cors(&self) -> bool {
        false
    }
}

impl SlimFetchRequest for FigArticleUpdateRequest {
    type RequestBody = FigArticleCreate;
    type ResponseBody = FigshareResponseBody;
    fn path(&self) -> String {
        // Endpoint for updating existing article.
        format!("/account/articles/{}", TEST_ARTICLE_ID.unwrap())
    }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        // Updates use HTTP method PUT.
        MethodBody::Put(&self.body)
    }
}

pub type PushFigshareRequest = Fetch<FetchWrapper<FigArticleUpdateRequest>, FigshareResponseBody>;
pub type PushActionFigshareRequest = FetchAction<FigshareResponseBody>;

// Basic interface: triggers conversion of Thoth Work data into Figshare Article format
// and sends write request with formatted data to Figshare endpoint.

pub struct FigshareComponent {
    props: Props,
    link: ComponentLink<Self>,
    push_figshare: PushFigshareRequest,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub work: WorkWithRelations,
}

pub enum Msg {
    SetFigsharePushState(PushActionFigshareRequest),
    Submit,
}

impl Component for FigshareComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let push_figshare = Default::default();
        FigshareComponent {
            props,
            link,
            push_figshare,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props);
        // Appearance of component is currently static, so no need to re-render
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetFigsharePushState(fetch_state) => {
                self.push_figshare.apply(fetch_state);
                // TODO: process response received from Figshare
                false
            }
            Msg::Submit => {
                let mut authors = vec![];
                for contribution in self.props.work.contributions.clone().unwrap_or_default() {
                    let author = FigArticleCreateAuthor {
                        name: contribution.full_name,
                        // Stored in Thoth, but not currently requested when retrieving Work
                        // orcid_id: contribution.contributor.orcid.unwrap_or_default(),
                    };
                    authors.push(author);
                }
                // Options as listed in documentation are:
                // figure | online resource | preprint | book | conference contribution
                // media | dataset | poster | journal contribution | presentation | thesis | software
                // However, options from ArticleSearch item_type full list also seem to be accepted:
                // 1 - Figure, 2 - Media, 3 - Dataset, 5 - Poster, 6 - Journal contribution, 7 - Presentation,
                // 8 - Thesis, 9 - Software, 11 - Online resource, 12 - Preprint, 13 - Book, 14 - Conference contribution,
                // 15 - Chapter, 16 - Peer review, 17 - Educational resource, 18 - Report, 19 - Standard, 20 - Composition,
                // 21 - Funding, 22 - Physical object, 23 - Data management plan, 24 - Workflow, 25 - Monograph,
                // 26 - Performance, 27 - Event, 28 - Service, 29 - Model
                let defined_type = match self.props.work.work_type {
                    WorkType::BookChapter => "chapter".to_string(),
                    WorkType::Monograph => "monograph".to_string(),
                    WorkType::EditedBook => "book".to_string(),
                    WorkType::Textbook => "educational resource".to_string(),
                    WorkType::JournalIssue => "book".to_string(),
                    WorkType::BookSet => "book".to_string(),
                };
                let keywords = self
                    .props
                    .work
                    .subjects
                    .clone()
                    .unwrap_or_default()
                    .iter()
                    .filter(|s| s.subject_type.eq(&SubjectType::Keyword))
                    .map(|s| s.subject_code.clone())
                    .collect();
                let fundings: Vec<String> = self
                    .props
                    .work
                    .fundings
                    .clone()
                    .unwrap_or_default()
                    .iter()
                    // Unclear which attribute to use as "the funding name"; use grant number for now.
                    // (Will omit fundings with no grant number.)
                    .filter_map(|f| f.grant_number.clone())
                    .collect();
                let mut funding_list = vec![];
                for funding in fundings {
                    funding_list.push(FigFundingCreate { title: funding });
                }
                let body = FigArticleCreate {
                    title: self.props.work.full_title.clone(),
                    description: self.props.work.long_abstract.clone().unwrap_or_default(),
                    authors,
                    defined_type,
                    keywords,
                    funding_list,
                    timeline: FigTimelineUpdate {
                        publisher_publication: self.props.work.publication_date.clone(),
                    },
                    // Supplied without leading "https://doi.org/".
                    // If empty, will submit "" and clear any previous value.
                    resource_doi: self.props.work.doi.clone().unwrap_or_default().to_string(),
                };
                let request = FetchWrapper(FigArticleUpdateRequest { body });
                self.push_figshare = Fetch::new(request);
                self.link
                    .send_future(self.push_figshare.fetch(Msg::SetFigsharePushState));
                self.link
                    .send_message(Msg::SetFigsharePushState(FetchAction::Fetching));
                false
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <button onclick=self.link.callback(|_| Msg::Submit)>
                { "Submit to Figshare" }
            </button>
        }
    }
}
