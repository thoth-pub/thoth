#[tokio::main]
pub async fn generate_onix_3(url: String, work_id: String) -> Result<(), reqwest::Error> {
    let query = format!(
        "{{
            work(workId: \"{}\") {{
                workId
                title
                workType
        }}}}",
        work_id
    );
    let echo_json: serde_json::Value = reqwest::Client::new()
        .post(&url)
        .json(&serde_json::json!({ "query": query }))
        .send()
        .await?
        .json()
        .await?;
    println!("{:#?}", echo_json["data"]["work"]);
    Ok(())
}
