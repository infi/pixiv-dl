use regex::Regex;
use std::error::Error;

pub async fn fetch_html(url: &str) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let res = client.get(url).send().await?;
    let html = res.text().await?;
    Ok(html)
}

pub async fn extract_title(html: &str) -> Result<String, Box<dyn Error>> {
    let re = Regex::new(r#""illustTitle":"[^"]+""#)?;
    let caps = re.captures(html).unwrap();
    let title = caps.get(0).unwrap().as_str();
    let title = title.replace("\"illustTitle\":\"", "");
    let title = title.replace("\"", "");
    Ok(title)
}

pub async fn extract_image_url(html: &str) -> Result<String, Box<dyn Error>> {
    let re = Regex::new(r#""original":"[^"]+""#)?;
    let caps = re.captures(html).unwrap();
    let title = caps.get(0).unwrap().as_str();
    let title = title.replace("\"original\":\"", "");
    let title = title.replace("\"", "");
    Ok(title)
}

pub async fn get_image(url: &str, post_url: &str) -> Result<reqwest::Response, Box<dyn Error>> {
    let client = reqwest::Client::new();
    // Set the "Referer" header to the url of the page that contains the image
    let res = client.get(url).header("Referer", post_url).send().await?;
    Ok(res)
}
