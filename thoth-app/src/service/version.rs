use serde_json::Value;
use thoth_api::errors::ThothError;
use yew::callback::Callback;
use yew::format::Nothing;
use yew::format::Text;
use yew::services::fetch::FetchService;
use yew::services::fetch::FetchTask;
use yew::services::fetch::Request;
use yew::services::fetch::Response;

pub fn check_version(callback: Callback<Result<String, ThothError>>) -> FetchTask {
    let handler = move |response: Response<Text>| {
        if let (meta, Ok(body)) = response.into_parts() {
            if meta.status.is_success() {
                let parsed_body: Result<Value, _> = serde_json::from_str(&body);
                match parsed_body {
                    Ok(data) => {
                        match data["version"].as_str() {
                            Some(version) => callback.emit(Ok(version.to_string())),
                            None => callback.emit(Err(ThothError::InternalError(
                                "No version information found".to_string(),
                            ))),
                        }
                    }
                    Err(e) => callback.emit(Err(ThothError::InternalError(e.to_string()))),
                }
            } else {
                callback.emit(Err(ThothError::InternalError(meta.status.to_string())))
            }
        } else {
            callback.emit(Err(ThothError::InternalError(
                "Could not parse HTTP response".to_string(),
            )))
        }
    };

    let builder = Request::builder()
        .method("GET")
        .uri("/manifest.json")
        .header("Content-Type", "application/json");
    let request = builder.body(Nothing).unwrap();

    FetchService::fetch(request, handler.into()).unwrap()
}
