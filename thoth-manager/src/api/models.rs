use serde::Deserialize;
use serde::Serialize;
use serde::de;
use serde::de::Deserializer;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Work {
    pub work_id: String,
    pub full_title: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub doi: String,
    pub cover_url: String,
    pub license: License,
    pub place: String,
    pub publication_date: Option<String>,
    pub contributions: Option<Vec<Contribution>>,
    pub imprint: Imprint,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum License {
    By,
    BySa,
    ByNd,
    ByNc,
    ByNcSa,
    ByNcNd,
    Zero,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Imprint {
    pub publisher: Publisher,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Publisher {
    pub publisher_id: String,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contribution {
    pub main_contribution: bool,
    pub contributor: Contributor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contributor {
    pub full_name: String,
}

impl<'de> Deserialize<'de> for License {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let l = String::deserialize(deserializer)?.to_lowercase();
        let license = match l.as_str() {
            "http://creativecommons.org/licenses/by/1.0/"
                | "http://creativecommons.org/licenses/by/2.0/"
                | "http://creativecommons.org/licenses/by/2.5/"
                | "http://creativecommons.org/licenses/by/3.0/"
                | "http://creativecommons.org/licenses/by/4.0/" => License::By,
            "http://creativecommons.org/licenses/by-sa/1.0/"
                  | "http://creativecommons.org/licenses/by-sa/2.0/"
                  | "http://creativecommons.org/licenses/by-sa/2.5/"
                  | "http://creativecommons.org/licenses/by-sa/3.0/"
                  | "http://creativecommons.org/licenses/by-sa/4.0/" => License::BySa,
            "http://creativecommons.org/licenses/by-nd/1.0/"
                  | "http://creativecommons.org/licenses/by-nd/2.0/"
                  | "http://creativecommons.org/licenses/by-nd/2.5/"
                  | "http://creativecommons.org/licenses/by-nd/3.0/"
                  | "http://creativecommons.org/licenses/by-nd/4.0/" => License::ByNd,
            "http://creativecommons.org/licenses/by-nc/1.0/"
                  | "http://creativecommons.org/licenses/by-nc/2.0/"
                  | "http://creativecommons.org/licenses/by-nc/2.5/"
                  | "http://creativecommons.org/licenses/by-nc/3.0/"
                  | "http://creativecommons.org/licenses/by-nc/4.0/" => License::ByNc,
            "http://creativecommons.org/licenses/by-nc-sa/1.0/"
                  | "http://creativecommons.org/licenses/by-nc-sa/2.0/"
                  | "http://creativecommons.org/licenses/by-nc-sa/2.5/"
                  | "http://creativecommons.org/licenses/by-nc-sa/3.0/"
                  | "http://creativecommons.org/licenses/by-nc-sa/4.0/" => License::ByNcSa,
            "http://creativecommons.org/licenses/by-nc-nd/1.0/"
                  | "http://creativecommons.org/licenses/by-nc-nd/2.0/"
                  | "http://creativecommons.org/licenses/by-nc-nd/2.5/"
                  | "http://creativecommons.org/licenses/by-nc-nd/3.0/"
                  | "http://creativecommons.org/licenses/by-nc-nd/4.0/" => License::ByNcNd,
            "https://creativecommons.org/publicdomain/zero/1.0/" => License::Zero,
            other => { return Err(de::Error::custom(format!("Invalid license '{}'", other))); },
        };
        Ok(license)
    }
}
