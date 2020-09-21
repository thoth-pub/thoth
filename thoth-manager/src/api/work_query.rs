use serde::Deserialize;
use serde::Serialize;

use crate::api::models::Work;

pub const WORK_QUERY: &str = "
    query WorkQuery($workId: Uuid!) {
        work(workId: $workId) {
            workId
            workType
            workStatus
            fullTitle
            title
            subtitle
            reference
            edition
            doi
            publicationDate
            place
            width
            height
            pageCount
            pageBreakdown
            imageCount
            tableCount
            videoCount
            license
            copyrightHolder
            landingPage
            lccn
            oclc
            shortAbstract
            longAbstract
            generalNote
            toc
            coverUrl
            coverCaption
            contributions {
                mainContribution
                contributor {
                    fullName
                }
            }
            imprint {
                publisher {
                    publisherId
                    publisherName
                    publisherShortname
                    publisherUrl
                }
            }
        }
    }
";

query_builder!{
    WorkRequest,
    WorkRequestBody,
    WORK_QUERY,
    WorkResponseBody,
    WorkResponseData,
    FetchWork,
    FetchActionWork
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkResponseData {
    pub work: Option<Work>,
}

impl Default for WorkResponseData {
    fn default() -> WorkResponseData {
        WorkResponseData {
            work: None
        }
    }
}
