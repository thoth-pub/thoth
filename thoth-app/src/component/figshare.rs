use md5::{Digest, Md5};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thoth_api::model::publication::Publication;
use thoth_api::model::subject::SubjectType;
use thoth_api::model::work::WorkType;
use thoth_api::model::work::WorkWithRelations;
use uuid::Uuid;
use yew::html;
use yew::prelude::*;
use yewtil::fetch::{Fetch, FetchAction, FetchError, FetchRequest, FetchState, Json, MethodBody};
use yewtil::future::LinkFuture;
use yewtil::NeqAssign;

// Test instance. Production instance is "https://api.figshare.com/v2".
const FIGSHARE_API_ROOT: &str = "https://api.figsh.com/v2";

// Upload API is separate from main API. Unclear whether this value
// may change - if so, should be obtained from main API responses.
const FIGSHARE_UPLOAD_API_ROOT: &str = "https://fup1010100.figsh.com/upload/";

// Authorization token associated with a Figshare user account.
// The token itself is security information and must not be published in open-source code.
// Instead, set it as an environment variable in the shell before starting the Thoth app
// (`export FIGSHARE_TOKEN=[value]`).
const FIGSHARE_TOKEN: Option<&str> = option_env!("FIGSHARE_TOKEN");

// Structures are named to match Figshare API objects where appropriate throughout.

// Child object of ArticleCreate representing an author.
// AuthorsCreator object is composed of an array of these.
// Note that this will be transformed in the created article into an Author object
// (with attributes id, full_name, is_active, url_name and orcid_id).
// url_name will default to "_" if no valid Figshare author ID is supplied.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]

pub struct FigAuthorsCreatorItem {
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigCustomFields {
    // Temporarily use existing custom field.
    #[serde(rename = "Administrator link")]
    pub thoth_publication_id: String,
}

// Can also be used to represent ArticleUpdate, as the objects are identical,
// as well as ArticleProjectCreate (which lacks group_id, but we don't use it here).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigArticleCreate {
    // Required fields for article creation:
    pub title: String,
    // Required fields for article publication:
    pub description: String,
    pub authors: Vec<FigAuthorsCreatorItem>,
    // Figshare IDs representing ANZSRC FoR categories - TBD how to map to Thoth categories
    // pub categories: Vec<i32>,
    pub defined_type: String,
    // Transformed into "tags" on creation - consider renaming
    pub keywords: Vec<String>,
    // Figshare ID - detailed list found at licences endpoint
    pub license: i32,
    // (A subset of) optional fields:
    // (note we may want to submit these even if empty, to overwrite previous values
    // - otherwise we could use skip_serializing_if to omit them)
    pub funding_list: Vec<FigFundingCreate>,
    pub timeline: FigTimelineUpdate,
    pub resource_doi: String,
    pub custom_fields: FigCustomFields,
}

#[derive(Debug, Clone, Default)]
pub struct FigArticleUpdateRequest {
    pub body: FigArticleCreate,
    pub article_id: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigLocationWarningsUpdate {
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
    // Default to main API - can be overridden
    fn root(&self) -> String {
        FIGSHARE_API_ROOT.to_string()
    }
    // Default to creating URL from root + path - can be overridden
    fn full_url(&self) -> String {
        format!("{}{}", self.root(), self.path())
    }
}

#[derive(Default)]
pub struct FetchWrapper<T>(T);

impl<T: SlimFetchRequest> FetchRequest for FetchWrapper<T> {
    type RequestBody = T::RequestBody;
    type ResponseBody = T::ResponseBody;
    // Appears to govern format of both request and response bodies -
    // can't handle e.g. JSON in request and plain text in response.
    // Binary formats are also not supported by this framework.
    type Format = Json;

    fn url(&self) -> String {
        self.0.full_url()
    }

    fn method(&self) -> MethodBody<Self::RequestBody> {
        self.0.method()
    }

    // Most requests to the main API require authentication information and a JSON body
    // containing the data to be processed. This format is re-used for requests to the
    // upload API, however, the latter does not require authentication information
    // (so the Authorization header is ignored).
    fn headers(&self) -> Vec<(String, String)> {
        let json = ("Content-Type".into(), "application/json".into());
        let auth = (
            "Authorization".into(),
            format!("token {}", FIGSHARE_TOKEN.unwrap()),
        );
        vec![json, auth]
    }

    fn use_cors(&self) -> bool {
        true
    }
}

impl SlimFetchRequest for FigArticleUpdateRequest {
    type RequestBody = FigArticleCreate;
    // Expected body structure on success - may be ErrorMessage
    // (with fields "message" and "code") on failure.
    type ResponseBody = FigLocationWarningsUpdate;
    fn path(&self) -> String {
        // Endpoint for updating existing article.
        format!("/account/articles/{}", self.article_id)
    }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        // Updates use HTTP method PUT.
        MethodBody::Put(&self.body)
    }
}

#[derive(Debug, Clone, Default)]
pub struct FigProjectArticleCreateRequest {
    pub body: FigArticleCreate,
    pub project_id: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigLocationWarnings {
    // Successful response contains article ID and full article URL
    pub entity_id: i32,
    pub location: String,
    pub warnings: Vec<String>,
}

impl SlimFetchRequest for FigProjectArticleCreateRequest {
    type RequestBody = FigArticleCreate;
    type ResponseBody = FigLocationWarnings;
    fn path(&self) -> String {
        // Endpoint for creating new article under project.
        format!("/account/projects/{}/articles", self.project_id)
    }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        // Creates use HTTP method POST.
        MethodBody::Post(&self.body)
    }
}

// Can also be used to represent ProjectUpdate, as the objects are identical
// (except ProjectCreate has a group_id parameter option, unused here)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigProjectCreate {
    pub title: String,
    pub description: String,
    pub funding_list: Vec<FigFundingCreate>,
    // pub custom_fields: FigCustomFields,
}

#[derive(Debug, Clone, Default)]
pub struct FigProjectUpdateRequest {
    pub body: FigProjectCreate,
    pub project_id: i32,
}

impl SlimFetchRequest for FigProjectUpdateRequest {
    type RequestBody = FigProjectCreate;
    // Body is empty on success, but may contain JSON ErrorMessage on failure.
    type ResponseBody = ();
    fn path(&self) -> String {
        // Endpoint for updating existing project.
        format!("/account/projects/{}", self.project_id)
    }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        // Updates use HTTP method PUT.
        MethodBody::Put(&self.body)
    }
}

#[derive(Debug, Clone, Default)]
pub struct FigProjectCreateRequest {
    pub body: FigProjectCreate,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigCreateProjectResponse {
    // Successful response contains project ID and full project URL
    pub entity_id: i32,
    pub location: String,
}

impl SlimFetchRequest for FigProjectCreateRequest {
    type RequestBody = FigProjectCreate;
    type ResponseBody = FigCreateProjectResponse;
    fn path(&self) -> String {
        // Endpoint for creating new project.
        "/account/projects".to_string()
    }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        // Creates use HTTP method POST.
        MethodBody::Post(&self.body)
    }
}

// Figshare object CommonSearch is shared by ArticleSearch,
// ProjectsSearch and CollectionSearch.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigCommonSearch {
    pub search_for: String,
}

#[derive(Debug, Clone, Default)]
pub struct FigArticleSearchRequest {
    pub body: FigCommonSearch,
}

// We are currently only using searches to find ID and
// Custom Fields - other parameters can be safely omitted.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigArticleSearchResponse {
    pub id: i32,
    pub custom_fields: FigCustomFields,
}

impl SlimFetchRequest for FigArticleSearchRequest {
    type RequestBody = FigCommonSearch;
    type ResponseBody = Vec<FigArticleSearchResponse>;
    fn path(&self) -> String {
        "/account/articles/search".to_string()
    }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        MethodBody::Post(&self.body)
    }
}

#[derive(Debug, Clone, Default)]
pub struct FigProjectSearchRequest {
    pub body: FigCommonSearch,
}

// We are currently only using searches to find ID -
// other parameters can be safely omitted.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigProjectSearchResponse {
    pub id: i32,
}

impl SlimFetchRequest for FigProjectSearchRequest {
    type RequestBody = FigCommonSearch;
    type ResponseBody = Vec<FigProjectSearchResponse>;
    fn path(&self) -> String {
        "/account/projects/search".to_string()
    }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        MethodBody::Post(&self.body)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigLicense {
    pub value: i32,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Default)]
pub struct FigLicenseListRequest {}

impl SlimFetchRequest for FigLicenseListRequest {
    type RequestBody = ();
    type ResponseBody = Vec<FigLicense>;
    fn path(&self) -> String {
        "/account/licenses".to_string()
    }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        MethodBody::Get
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigFileCreator {
    pub md5: String,
    pub name: String,
    pub size: i32,
    // Should never be filled out - stores external link without saving its content
    // pub link: String,
}

#[derive(Debug, Clone, Default)]
pub struct FigUploadInitiateRequest {
    pub body: FigFileCreator,
    pub article_id: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigLocation {
    pub location: String,
}

impl SlimFetchRequest for FigUploadInitiateRequest {
    type RequestBody = FigFileCreator;
    type ResponseBody = FigLocation;
    fn path(&self) -> String {
        format!("/account/articles/{}/files", self.article_id)
    }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        MethodBody::Post(&self.body)
    }
}

#[derive(Debug, Clone, Default)]
pub struct FigUploadGetUrlRequest {
    // Previous response contains full URL. Plain file ID not easily extracted.
    // pub file_id: String,
    pub location: String,
}

// Defined by upload API, not main API - Figshare object name not specified
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigUploadGetUrlResponse {
    pub upload_token: String,
    pub upload_url: String,
    pub status: String,
    pub preview_state: String,
    pub viewer_type: String,
    pub id: i32,
    pub name: String,
    pub size: i32,
    pub is_link_only: bool,
    pub download_url: String,
    pub supplied_md5: String,
    pub computed_md5: String,
}

impl SlimFetchRequest for FigUploadGetUrlRequest {
    type RequestBody = ();
    type ResponseBody = FigUploadGetUrlResponse;
    // Override default root + path URL with full URL from previous response.
    // `path()` will not be used but must be implemented.
    // Alternatively, extract plain file ID and omit `full_url()`,
    // using commented-out version of `path()` below.
    fn full_url(&self) -> String {
        self.location.clone()
    }
    fn path(&self) -> String {
        "unimplemented".to_string()
    }
    // fn path(&self) -> String {
    //     format!("/account/articles/{}/files/{}",
    //     self.article_id.to_string(),
    //     &self.file_id)
    // }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        MethodBody::Get
    }
}

#[derive(Debug, Clone, Default)]
pub struct FigUploadGetPartsRequest {
    pub upload_token: String,
    // pub upload_url: String,
}

// As above - Figshare object name not defined
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigUploadGetPartsResponse {
    pub token: String,
    pub name: String,
    pub size: i32,
    pub md5: String,
    pub status: String,
    pub parts: Vec<FigUploadPartData>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FigUploadPartData {
    pub part_no: i32,
    pub start_offset: i32,
    pub end_offset: i32,
    pub status: String,
    pub locked: bool,
}

impl SlimFetchRequest for FigUploadGetPartsRequest {
    type RequestBody = ();
    type ResponseBody = FigUploadGetPartsResponse;
    fn root(&self) -> String {
        FIGSHARE_UPLOAD_API_ROOT.to_string()
    }
    fn path(&self) -> String {
        self.upload_token.to_string()
    }
    // Previous response contains both upload_url (root + upload_token)
    // and plain upload_token. Alternative implementation uses full URL:
    // fn full_url(&self) -> String { &self.upload_url }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        MethodBody::Get
    }
}

#[derive(Debug, Clone, Default)]
pub struct FigUploadSendPartRequest {
    pub upload_token: String,
    pub part_no: String,
    pub body: Vec<u8>,
}

impl SlimFetchRequest for FigUploadSendPartRequest {
    // Issue: due to framework limitations, this is submitted as JSON data
    // rather than plain binary. Uploaded file therefore does not match
    // original data and MD5 submitted/calculated values do not correspond.
    type RequestBody = Vec<u8>;
    // Body is not actually empty but contains plain text "OK" (if success -
    // may be a JSON-formatted error message otherwise).
    // Fetch framework expects JSON body so we cannot easily set appropriate type.
    type ResponseBody = ();
    fn root(&self) -> String {
        FIGSHARE_UPLOAD_API_ROOT.to_string()
    }
    fn path(&self) -> String {
        format!("{}/{}", self.upload_token, self.part_no)
    }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        MethodBody::Put(&self.body)
    }
}

// Note: structure identical to FigUploadGetUrlRequest
// (but this cannot be reused as the SlimFetchRequest impl needs to be different).
#[derive(Debug, Clone, Default)]
pub struct FigUploadCompleteRequest {
    // Previous response contains full URL. Plain file ID not easily extracted.
    // pub file_id: String,
    pub location: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FigUploadCompleteRequestBody {}

impl SlimFetchRequest for FigUploadCompleteRequest {
    // API requires a POST with empty body.
    // Unclear how to do this within Fetch framework.
    // Send dummy struct - this is successful as API ignores body.
    type RequestBody = FigUploadCompleteRequestBody;
    // Body is not actually empty but contains HTML "Accepted" message (if success -
    // may be a JSON-formatted error message otherwise).
    // Fetch framework expects JSON body so we cannot easily set appropriate type.
    type ResponseBody = ();
    // Override default root + path URL with full URL from previous response.
    // `path()` will not be used but must be implemented.
    // Alternatively, extract plain file ID and omit `full_url()`,
    // using commented-out version of `path()` below.
    // (See also FigUploadGetUrlRequest.)
    fn full_url(&self) -> String {
        self.location.clone()
    }
    fn path(&self) -> String {
        "unimplemented".to_string()
    }
    // fn path(&self) -> String {
    //     format!("/account/articles/{}/files/{}",
    //     self.article_id.to_string(),
    //     &self.file_id)
    // }
    fn method(&self) -> MethodBody<Self::RequestBody> {
        MethodBody::Post(&FigUploadCompleteRequestBody {})
    }
}

pub type PushCreateArticle =
    Fetch<FetchWrapper<FigProjectArticleCreateRequest>, FigLocationWarnings>;
pub type PushActionCreateArticle = FetchAction<FigLocationWarnings>;
pub type PushUpdateArticle =
    Fetch<FetchWrapper<FigArticleUpdateRequest>, FigLocationWarningsUpdate>;
pub type PushActionUpdateArticle = FetchAction<FigLocationWarningsUpdate>;
pub type FetchArticleDetails =
    Fetch<FetchWrapper<FigArticleSearchRequest>, Vec<FigArticleSearchResponse>>;
pub type FetchActionArticleDetails = FetchAction<Vec<FigArticleSearchResponse>>;
pub type PushCreateProject = Fetch<FetchWrapper<FigProjectCreateRequest>, FigCreateProjectResponse>;
pub type PushActionCreateProject = FetchAction<FigCreateProjectResponse>;
pub type PushUpdateProject = Fetch<FetchWrapper<FigProjectUpdateRequest>, ()>;
pub type PushActionUpdateProject = FetchAction<()>;
pub type FetchProjectDetails =
    Fetch<FetchWrapper<FigProjectSearchRequest>, Vec<FigProjectSearchResponse>>;
pub type FetchActionProjectDetails = FetchAction<Vec<FigProjectSearchResponse>>;
pub type FetchLicenseList = Fetch<FetchWrapper<FigLicenseListRequest>, Vec<FigLicense>>;
pub type FetchActionLicenseList = FetchAction<Vec<FigLicense>>;
pub type PushInitiateUpload = Fetch<FetchWrapper<FigUploadInitiateRequest>, FigLocation>;
pub type PushActionInitiateUpload = FetchAction<FigLocation>;
pub type FetchUploadUrl = Fetch<FetchWrapper<FigUploadGetUrlRequest>, FigUploadGetUrlResponse>;
pub type FetchActionUploadUrl = FetchAction<FigUploadGetUrlResponse>;
pub type FetchUploadParts =
    Fetch<FetchWrapper<FigUploadGetPartsRequest>, FigUploadGetPartsResponse>;
pub type FetchActionUploadParts = FetchAction<FigUploadGetPartsResponse>;
pub type PushCreateUploadPart = Fetch<FetchWrapper<FigUploadSendPartRequest>, ()>;
pub type PushActionCreateUploadPart = FetchAction<()>;
pub type PushCompleteUpload = Fetch<FetchWrapper<FigUploadCompleteRequest>, ()>;
pub type PushActionCompleteUpload = FetchAction<()>;

// Basic interface: triggers conversion of Thoth Work data into Figshare Article format
// and sends write request with formatted data to Figshare endpoint.

pub struct FigshareComponent {
    props: Props,
    link: ComponentLink<Self>,
    create_article: PushCreateArticle,
    update_article: PushUpdateArticle,
    get_article_id: FetchArticleDetails,
    create_project: PushCreateProject,
    update_project: PushUpdateProject,
    get_project_id: FetchProjectDetails,
    get_license_list: FetchLicenseList,
    upload_get_id: PushInitiateUpload,
    upload_get_url: FetchUploadUrl,
    upload_get_parts: FetchUploadParts,
    upload_send_part: PushCreateUploadPart,
    upload_get_result: PushCompleteUpload,
    file_location: String,
    project_id: i32,
    article_publication_mapping: Vec<(i32, Uuid)>,
    license_list: Vec<FigLicense>,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub work: WorkWithRelations,
}

pub enum Msg {
    SetArticleCreateState(PushActionCreateArticle),
    SetArticleUpdateState(PushActionUpdateArticle),
    SubmitArticleToProject(i32, Publication),
    SetFigshareArticleIdFetchState(FetchActionArticleDetails),
    GetFigshareArticleId(Uuid),
    SetProjectCreateState(PushActionCreateProject),
    SetProjectUpdateState(PushActionUpdateProject),
    SubmitAsProject,
    SetFigshareProjectIdFetchState(FetchActionProjectDetails),
    GetFigshareProjectId,
    SetFigshareLicenseListFetchState(FetchActionLicenseList),
    GetFigshareLicenseList,
    InitiateFigshareUpload,
    GetFigshareFileId(PushActionInitiateUpload),
    GetFigshareUploadUrl(FetchActionUploadUrl),
    GetFigshareUploadParts(FetchActionUploadParts),
    ConcludeFigshareUpload(PushActionCreateUploadPart),
    GetFigshareUploadResult(PushActionCompleteUpload),
}

impl Component for FigshareComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let create_article = Default::default();
        let update_article = Default::default();
        let get_article_id = Default::default();
        let create_project = Default::default();
        let update_project = Default::default();
        let get_project_id = Default::default();
        let get_license_list = Default::default();
        let upload_get_id = Default::default();
        let upload_get_url = Default::default();
        let upload_get_parts = Default::default();
        let upload_send_part = Default::default();
        let upload_get_result = Default::default();
        let file_location = Default::default();
        let project_id = Default::default();
        let article_publication_mapping = Default::default();
        let license_list = Default::default();

        // Check whether a Figshare project representing this work already exists.
        // Ideally we would also store the Figshare project ID within the Thoth work,
        // and double-check that both IDs matched, to avoid mis-associating data.
        // Alternative implementation: re-run this check immediately before any attempt
        // to submit data (as Figshare state may change after Thoth page is opened).
        link.send_message(Msg::GetFigshareProjectId);
        // For each publication under the work, check whether a Figshare article
        // representing it already exists (same considerations as above).
        for publication in props.work.publications.clone().unwrap_or_default() {
            link.send_message(Msg::GetFigshareArticleId(publication.publication_id));
        }

        // Obtain the current set of available licences from the Figshare API
        link.send_message(Msg::GetFigshareLicenseList);

        FigshareComponent {
            props,
            link,
            create_article,
            update_article,
            get_article_id,
            create_project,
            update_project,
            get_project_id,
            get_license_list,
            upload_get_id,
            upload_get_url,
            upload_get_parts,
            upload_send_part,
            upload_get_result,
            file_location,
            project_id,
            article_publication_mapping,
            license_list,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let updated_work = props.work.work_id != self.props.work.work_id;
        let updated_publications = props.work.publications != self.props.work.publications;
        self.props.neq_assign(props);
        if updated_work {
            // Retrieve and store Figshare project ID associated with new Work
            self.link.send_message(Msg::GetFigshareProjectId);
        }
        if updated_publications {
            // Recreate list of associations between publications and Figshare articles
            self.article_publication_mapping.clear();
            for publication in self.props.work.publications.clone().unwrap_or_default() {
                self.link
                    .send_message(Msg::GetFigshareArticleId(publication.publication_id));
            }
        }
        // Appearance of component is currently static, so no need to re-render
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetArticleCreateState(fetch_state) => {
                self.create_article.apply(fetch_state);
                match self.create_article.as_ref().state() {
                    FetchState::Fetched(_body) => {
                        // TODO we need to add the new article to the publication mapping list,
                        // but we don't have the publication ID any more.
                    }
                    // TODO handle other responses
                    FetchState::Fetching(_) => (),
                    FetchState::NotFetching(_) => (),
                    FetchState::Failed(_, _) => (),
                }
                false
            }
            Msg::SetArticleUpdateState(fetch_state) => {
                self.update_article.apply(fetch_state);
                // TODO: process response received from Figshare
                // Issue: we expect a FigLocationWarningsUpdate JSON body on success,
                // and Content-Type/Content-Length headers suggest one is sent,
                // but browser appears to interpret body as empty.
                false
            }
            Msg::SubmitArticleToProject(article_id, publication) => {
                // Extract metadata from Thoth record and convert to Figshare format.
                // Note that the metadata is taken from the display version of the
                // Thoth record, including any user changes not yet saved to database.
                let mut authors = vec![];
                for contribution in self.props.work.contributions.clone().unwrap_or_default() {
                    let author = FigAuthorsCreatorItem {
                        name: contribution.full_name,
                        // Stored in Thoth, but not currently requested when retrieving Work.
                        // Will cause error if duplicated by an existing Figshare author record.
                        // orcid_id: contribution.contributor.orcid.unwrap_or_default(),
                        // Workaround to store Thoth contributor ID in Figshare
                        // (will cause error if not formatted as an email address)
                        // - does not seem to be displayed in Figshare record.
                        // Will cause error if duplicated by an existing Figshare author record.
                        // email: format!("{}@thoth.pub", contribution.contributor_id),
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
                // Find the Figshare licence object corresponding to the Thoth licence URL.
                // Note URLs must match exactly, e.g. "http://[...]" will not match to "https://[...]".
                // If multiple Figshare licence objects have the same URL, the lowest numbered will be used.
                // If Thoth licence URL field is empty, use the special licence value "unknown".
                // TODO: Create "unknown" private licence (cannot be done via Figshare API).
                // This would need to be done individually for every institutional Figshare account.
                let mut figshare_license = self.license_list.iter().find(|l| {
                    l.url.eq(&self
                        .props
                        .work
                        .license
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string()))
                });
                if figshare_license.is_none() {
                    // No appropriate Figshare licence object was found. This is probably because
                    // the Thoth licence URL field was filled out but the value did not match any
                    // existing Figshare licence. Use the special licence value "unknown".
                    figshare_license = self
                        .license_list
                        .iter()
                        .find(|l| l.url.eq(&"unknown".to_string()));
                }
                // If we still haven't found an appropriate Figshare licence object,
                // we must submit a default value. Use 1 as this matches the Figshare
                // default behaviour. Not ideal as it corresponds to CC-BY[-4.0].
                let license = figshare_license.map_or(1, |l| l.value);
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
                    title: format!(
                        "{} - {}",
                        self.props.work.full_title.clone(),
                        publication.publication_type
                    ),
                    description: self.props.work.long_abstract.clone().unwrap_or_default(),
                    authors,
                    defined_type,
                    keywords,
                    license,
                    funding_list,
                    timeline: FigTimelineUpdate {
                        publisher_publication: self.props.work.publication_date.clone(),
                    },
                    // Supplied without leading "https://doi.org/", as required by Figshare.
                    // If empty, will submit "" and clear any previous value.
                    resource_doi: self.props.work.doi.clone().unwrap_or_default().to_string(),
                    // TODO first check that the custom field where we aim to store this value exists
                    custom_fields: FigCustomFields {
                        // Note that Thoth Publication IDs are only guaranteed unique per Thoth instance.
                        // If other users spin up independent versions of Thoth,
                        // multiple Figshare records might be created with the same Thoth Publication ID.
                        thoth_publication_id: format!(
                            "thoth-publication-id:{}",
                            publication.publication_id
                        ),
                    },
                };
                match article_id {
                    0 => {
                        // Create new article under current project
                        // POST to /account/projects/{project_id}/articles
                        // JSON body: article structure
                        let request = FetchWrapper(FigProjectArticleCreateRequest {
                            body,
                            project_id: self.project_id,
                        });
                        self.create_article = Fetch::new(request);
                        self.link
                            .send_future(self.create_article.fetch(Msg::SetArticleCreateState));
                        self.link
                            .send_message(Msg::SetArticleCreateState(FetchAction::Fetching));
                    }
                    _ => {
                        // Update existing article (note cannot be done via
                        // /projects/{project_id}/articles/{article_id} endpoint -
                        // but article_id value is the same for both)
                        // PUT to /account/articles/{article_id}
                        // JSON body: article structure (same as for create)
                        let request = FetchWrapper(FigArticleUpdateRequest { body, article_id });
                        self.update_article = Fetch::new(request);
                        self.link
                            .send_future(self.update_article.fetch(Msg::SetArticleUpdateState));
                        self.link
                            .send_message(Msg::SetArticleUpdateState(FetchAction::Fetching));
                    }
                }
                false
            }
            Msg::SetFigshareArticleIdFetchState(fetch_state) => {
                self.get_article_id.apply(fetch_state);
                match self.get_article_id.as_ref().state() {
                    FetchState::Fetched(body) => {
                        match body.len() {
                            // No matching articles found - we need to create one
                            0 => (),
                            // Article already exists for this Thoth Publication - we can update it
                            1 => {
                                let publication_id = Uuid::parse_str(
                                    body[0]
                                        .custom_fields
                                        .thoth_publication_id
                                        .strip_prefix("thoth-publication-id:")
                                        // TODO this should not fail - need to handle the error if
                                        // expected prefix is somehow missing
                                        .unwrap(),
                                )
                                // TODO again, should not fail - handle error if it does
                                .unwrap();
                                self.article_publication_mapping
                                    .push((body[0].id, publication_id));
                            }
                            // TODO raise an error - multiple matching articles found
                            // (Figshare representations of Thoth Publications should be unique)
                            // This could indicate that Publications from independent Thoth instances
                            // have coincidentally been assigned the same ID.
                            _ => (),
                        }
                    }
                    // TODO handle other responses
                    FetchState::Fetching(_) => (),
                    FetchState::NotFetching(_) => (),
                    FetchState::Failed(_, _) => (),
                }
                false
            }
            Msg::GetFigshareArticleId(publication_id) => {
                // POST to /account/articles/search
                // JSON body: term to be searched (formatted Thoth Publication ID)
                // TODO first check that the custom field where we expect to find this value exists
                let body = FigCommonSearch {
                    search_for: format!("thoth-publication-id:{}", publication_id),
                };
                let request = FetchWrapper(FigArticleSearchRequest { body });
                self.get_article_id = Fetch::new(request);
                self.link.send_future(
                    self.get_article_id
                        .fetch(Msg::SetFigshareArticleIdFetchState),
                );
                self.link
                    .send_message(Msg::SetFigshareArticleIdFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetProjectCreateState(fetch_state) => {
                // Duplicated from SetArticleCreateState.
                self.create_project.apply(fetch_state);
                match self.create_project.as_ref().state() {
                    FetchState::Fetched(body) => {
                        // On success, save off returned project ID
                        self.project_id = body.entity_id;
                        // Create articles under the project for each
                        // publication associated with the work.
                        for publication in self.props.work.publications.clone().unwrap_or_default()
                        {
                            self.link
                                .send_message(Msg::SubmitArticleToProject(0, publication));
                        }
                    }
                    // TODO handle other responses
                    FetchState::Fetching(_) => (),
                    FetchState::NotFetching(_) => (),
                    FetchState::Failed(_, _) => (),
                }
                false
            }
            Msg::SetProjectUpdateState(fetch_state) => {
                // Duplicated/extended from SetArticleUpdateState.
                self.update_project.apply(fetch_state);
                match self.update_project.as_ref().state() {
                    // TODO due to framework poorly handling empty response bodies,
                    // successful responses will show as "Failed" - need workaround
                    FetchState::Fetched(_body) => {
                        // For each publication associated with the work, either
                        // create a new article under the project, or update
                        // existing article if found.
                        for publication in self.props.work.publications.clone().unwrap_or_default()
                        {
                            let mapping = self
                                .article_publication_mapping
                                .iter()
                                .find(|m| m.1 == publication.publication_id);
                            if let Some((article_id, _publication_id)) = mapping {
                                self.link.send_message(Msg::SubmitArticleToProject(
                                    *article_id,
                                    publication,
                                ));
                            } else {
                                self.link
                                    .send_message(Msg::SubmitArticleToProject(0, publication));
                            }
                        }
                    }
                    // TODO handle other responses
                    FetchState::Fetching(_) => (),
                    FetchState::NotFetching(_) => (),
                    FetchState::Failed(_, _) => (),
                }
                false
            }
            Msg::SubmitAsProject => {
                // Duplicated from SubmitAsArticle but omitting non-Project fields
                // (authors, defined_type, keywords, license, timeline, resource_doi).
                // Extract metadata from Thoth record and convert to Figshare format.
                // Note that the metadata is taken from the display version of the
                // Thoth record, including any user changes not yet saved to database.
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
                let body = FigProjectCreate {
                    title: self.props.work.full_title.clone(),
                    // Ideally we would include the Thoth Work ID in the Custom Fields
                    // as for Articles, but test instance doesn't currently have
                    // Custom Fields set up for Projects.
                    description: format!("thoth-work-id:{}", self.props.work.work_id),
                    // description: self.props.work.long_abstract.clone().unwrap_or_default(),
                    funding_list,
                    // custom_fields: FigCustomFields {
                    //     thoth_work_id: format!("thoth-work-id:{}", self.props.work.work_id),
                    // },
                };
                match self.project_id {
                    0 => {
                        // Create new project
                        // POST to /account/projects
                        // JSON body: project structure
                        let request = FetchWrapper(FigProjectCreateRequest { body });
                        self.create_project = Fetch::new(request);
                        self.link
                            .send_future(self.create_project.fetch(Msg::SetProjectCreateState));
                        self.link
                            .send_message(Msg::SetProjectCreateState(FetchAction::Fetching));
                    }
                    _ => {
                        // Update existing project
                        // PUT to /account/projects/{project_id}
                        // JSON body: project structure (same as for create)
                        let request = FetchWrapper(FigProjectUpdateRequest {
                            body,
                            project_id: self.project_id,
                        });
                        self.update_project = Fetch::new(request);
                        self.link
                            .send_future(self.update_project.fetch(Msg::SetProjectUpdateState));
                        self.link
                            .send_message(Msg::SetProjectUpdateState(FetchAction::Fetching));
                    }
                }
                false
            }
            Msg::SetFigshareProjectIdFetchState(fetch_state) => {
                // Duplicated from SetFigshareArticleIdFetchState.
                self.get_project_id.apply(fetch_state);
                match self.get_project_id.as_ref().state() {
                    FetchState::Fetched(body) => {
                        match body.len() {
                            // No matching projects found - we need to create one
                            // (clear any existing project ID in case the search was
                            // triggered by loading a different Work)
                            0 => self.project_id = Default::default(),
                            // Project already exists for this Thoth Work - we can update it
                            1 => self.project_id = body[0].id,
                            // TODO raise an error - multiple matching projects found
                            // (Figshare representations of Thoth Works should be unique)
                            // This could indicate that Works from independent Thoth instances
                            // have coincidentally been assigned the same ID.
                            _ => (),
                        }
                    }
                    // TODO handle other responses
                    FetchState::Fetching(_) => (),
                    FetchState::NotFetching(_) => (),
                    FetchState::Failed(_, _) => (),
                }
                false
            }
            Msg::GetFigshareProjectId => {
                // Duplicated from GetFigshareArticleId - search logic is the same.
                // POST to /account/projects/search
                // JSON body: term to be searched (formatted Thoth Work ID)
                // TODO first check that the custom field where we expect to find this value exists
                // (note comment in SubmitAsProject - custom fields not currently set up for projects)
                let body = FigCommonSearch {
                    search_for: format!("thoth-work-id:{}", self.props.work.work_id),
                };
                let request = FetchWrapper(FigProjectSearchRequest { body });
                self.get_project_id = Fetch::new(request);
                self.link.send_future(
                    self.get_project_id
                        .fetch(Msg::SetFigshareProjectIdFetchState),
                );
                self.link
                    .send_message(Msg::SetFigshareProjectIdFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetFigshareLicenseListFetchState(fetch_state) => {
                self.get_license_list.apply(fetch_state);
                match self.get_license_list.as_ref().state() {
                    FetchState::Fetched(body) => {
                        // Store retrieved list locally for reference
                        self.license_list = body.to_vec();
                    }
                    // TODO handle other responses
                    FetchState::Fetching(_) => (),
                    FetchState::NotFetching(_) => (),
                    FetchState::Failed(_, _) => (),
                }
                false
            }
            Msg::GetFigshareLicenseList => {
                // GET from /account/licenses
                // JSON body: none
                let request = FetchWrapper(FigLicenseListRequest {});
                self.get_license_list = Fetch::new(request);
                self.link.send_future(
                    self.get_license_list
                        .fetch(Msg::SetFigshareLicenseListFetchState),
                );
                self.link
                    .send_message(Msg::SetFigshareLicenseListFetchState(FetchAction::Fetching));
                false
            }
            Msg::InitiateFigshareUpload => {
                // POST to /account/articles/{article_id}/files
                // JSON body: "md5", "name", "size"
                // Calculate MD5 hash of file to be uploaded
                let mut hasher = Md5::new();
                // Hard-coded temporary test data
                hasher.update(b"12345");
                let hash = hasher.finalize();
                let md5 = format!("{:x}", hash);
                let body = FigFileCreator {
                    md5,
                    name: "name".to_string(),
                    size: 5,
                };
                let request = FetchWrapper(FigUploadInitiateRequest {
                    body,
                    // Test only: uploads file to first Work-related article found, if any
                    // (API call will fail due to bad article ID if no articles exist)
                    article_id: self.article_publication_mapping.first().map_or(0, |m| m.0),
                });
                self.upload_get_id = Fetch::new(request);
                self.link
                    .send_future(self.upload_get_id.fetch(Msg::GetFigshareFileId));
                self.link
                    .send_message(Msg::GetFigshareFileId(FetchAction::Fetching));
                false
            }
            Msg::GetFigshareFileId(fetch_state) => {
                self.upload_get_id.apply(fetch_state);
                match self.upload_get_id.as_ref().state() {
                    FetchState::Fetched(body) => {
                        // Response contains full URL (in format root/account/articles/{article_id}/files/{file_id}).
                        // Save off for use when confirming upload completed.
                        // Alternatively we could extract and save the plain file ID.
                        self.file_location = body.location.clone();
                        // GET from /account/articles/{article_id}/files/{file_id}
                        // JSON body: none
                        let request = FetchWrapper(FigUploadGetUrlRequest {
                            // file_id: self.file_id.clone()
                            location: self.file_location.clone(),
                        });
                        self.upload_get_url = Fetch::new(request);
                        self.link
                            .send_future(self.upload_get_url.fetch(Msg::GetFigshareUploadUrl));
                        self.link
                            .send_message(Msg::GetFigshareUploadUrl(FetchAction::Fetching));
                    }
                    // TODO handle other responses
                    FetchState::Fetching(_) => (),
                    FetchState::NotFetching(_) => (),
                    FetchState::Failed(_, _) => (),
                }
                false
            }
            Msg::GetFigshareUploadUrl(fetch_state) => {
                self.upload_get_url.apply(fetch_state);
                match self.upload_get_url.as_ref().state() {
                    FetchState::Fetched(body) => {
                        // Response contains full upload_url (in format upload_root/{upload_token})
                        // and, separately, plain upload_token. Could alternatively extract full URL.
                        // GET from [upload API root]/{upload_token} (separate from main Figshare API)
                        // JSON body: none
                        let request = FetchWrapper(FigUploadGetPartsRequest {
                            // upload_url: body.upload_url.clone()
                            upload_token: body.upload_token.clone(),
                        });
                        self.upload_get_parts = Fetch::new(request);
                        self.link
                            .send_future(self.upload_get_parts.fetch(Msg::GetFigshareUploadParts));
                        self.link
                            .send_message(Msg::GetFigshareUploadParts(FetchAction::Fetching));
                    }
                    // TODO handle other responses
                    FetchState::Fetching(_) => (),
                    FetchState::NotFetching(_) => (),
                    FetchState::Failed(_, _) => (),
                }
                false
            }
            Msg::GetFigshareUploadParts(fetch_state) => {
                self.upload_get_parts.apply(fetch_state);
                match self.upload_get_parts.as_ref().state() {
                    FetchState::Fetched(body) => {
                        // Response contains upload token (again), and set of parts into
                        // which data needs to be split (inc. part_no and start/end offsets).
                        // For each part:
                        // PUT to [upload API root]/{upload_token}/{part_no}
                        // Body: raw file data (should be binary, but framework encodes as JSON)
                        // TODO: add support for multi-part files, including calculating offsets
                        // (currently only tested and working for files of exactly one part)
                        for part in &body.parts {
                            let request = FetchWrapper(FigUploadSendPartRequest {
                                upload_token: body.token.clone(),
                                part_no: part.part_no.to_string(),
                                // Hard-coded temporary test data
                                body: "12345".as_bytes().to_owned(),
                            });
                            self.upload_send_part = Fetch::new(request);
                            self.link.send_future(
                                self.upload_send_part.fetch(Msg::ConcludeFigshareUpload),
                            );
                            self.link
                                .send_message(Msg::ConcludeFigshareUpload(FetchAction::Fetching));
                        }
                    }
                    // TODO handle other responses
                    FetchState::Fetching(_) => (),
                    FetchState::NotFetching(_) => (),
                    FetchState::Failed(_, _) => (),
                }
                false
            }
            Msg::ConcludeFigshareUpload(fetch_state) => {
                self.upload_send_part.apply(fetch_state);
                match self.upload_send_part.as_ref().state() {
                    // Workaround for handling Figshare 200 OK response with
                    // plain text body "OK": Fetch logic expects JSON body
                    // (not trivial to change) therefore fails to handle.
                    // If the body text is "OK" as expected, assume success.
                    FetchState::Failed(_body, fetch_error) => {
                        if let FetchError::DeserializeError { error: _, content } = fetch_error {
                            if content.eq(&"OK".to_string()) {
                                // To mark the upload as completed:
                                // POST to /account/articles/{article_id}/files/{file_id}
                                // JSON body: none
                                // TODO: in practice, need to wait until all parts have successfully been uploaded.
                                // Options: save off number of parts and complete upload when corresponding number
                                // of success responses received; GET from {upload_token} endpoint again and test
                                // that all parts now have "status": "COMPLETE".
                                let request = FetchWrapper(FigUploadCompleteRequest {
                                    // file_id: self.file_id.clone()
                                    location: self.file_location.clone(),
                                });
                                self.upload_get_result = Fetch::new(request);
                                self.link.send_future(
                                    self.upload_get_result.fetch(Msg::GetFigshareUploadResult),
                                );
                                self.link.send_message(Msg::GetFigshareUploadResult(
                                    FetchAction::Fetching,
                                ));
                            }
                            // TODO handle other errors
                        }
                    }
                    // TODO handle other responses
                    // (including potentially retrying part send on failure)
                    FetchState::Fetching(_) => (),
                    FetchState::NotFetching(_) => (),
                    FetchState::Fetched(_) => (),
                }
                false
            }
            Msg::GetFigshareUploadResult(fetch_state) => {
                self.upload_get_result.apply(fetch_state);
                // TODO: process response received from Figshare
                // (on success: 202 Accepted with HTML body)
                // including testing that the status has been set to `available`
                // (i.e. post-upload checks succeeded), and clearing file_location
                false
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <>
                <button onclick=self.link.callback(|_| Msg::SubmitAsProject)>
                    { "Submit to Figshare as a Project" }
                </button>
                <button onclick=self.link.callback(|_| Msg::InitiateFigshareUpload)>
                    { "Upload test file" }
                </button>
            </>
        }
    }
}
