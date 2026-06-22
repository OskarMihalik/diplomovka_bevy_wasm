use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an HTTP client
    let client = reqwest::Client::new();

    // Define the JSON payload
    let body = json!({
        "email": "a@mail.com",
        "password": "a@mail.com"
    });

    // Build and send the POST request
    let response = client
        .post("https://diplomovka-backend.onrender.com/login")
        .header("accept", "*/*")
        .header("accept-language", "sk-SK,sk;q=0.9,cs;q=0.8,en-US;q=0.7,en;q=0.6")
        .header("content-type", "application/json")
        .header("dnt", "1")
        .header("origin", "https://diplomovka-bevy-wasm.onrender.com")
        .header("priority", "u=1, i")
        .header("referer", "https://diplomovka-bevy-wasm.onrender.com/")
        .header("sec-ch-ua", r#""Google Chrome";v="147", "Not.A/Brand";v="8", "Chromium";v="147""#)
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", r#""Linux""#)
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "cross-site")
        .header("user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36")
        .json(&body)
        .send()
        .await?;

    // Print the output details
    println!("Status: {}", response.status());

    let response_text = response.text().await?;
    println!("Response Body:\n{}", response_text);

    Ok(())
}
