#[macro_use]
extern crate log;

mod extractor;

use std::{
    io::Write,
    sync::atomic::{AtomicUsize, Ordering},
};

use regex::Regex;

async fn extract_from_url(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Downloading {}", url);

    let html = extractor::fetch_html(&url).await?;
    let title = extractor::extract_title(&html).await?;
    let original_url = extractor::extract_image_url(&html).await?;
    // returns a reqwest::Response
    let image = extractor::get_image(&original_url, &url).await?;

    // Extract the file extension from the URL
    let ext = original_url.rsplit('.').next().unwrap();

    // Create a filename
    let filename = format!("{} - {}.{}", title, url.split('/').last().unwrap(), ext);

    // Turn the response into bytes
    let image_bytes = image.bytes().await?;

    info!("Writing {}", filename);

    // Write the image to disk
    let mut file = std::fs::File::create(filename)?;
    file.write_all(&image_bytes)?;

    Ok(())
}

static THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    // Take in URLs as arguments. Remember to strip the first argument
    let args: Vec<String> = std::env::args().collect();
    let urls: Vec<String> = args.iter().skip(1).map(|x| x.to_string()).collect();

    if urls.len() == 0 {
        error!("No URLs provided! Usage: pixiv-dl <url> [<url> <url> ...]");
        std::process::exit(1);
    }

    // Create a regex to match the URLs
    let pixiv_url_re = Regex::new("https://www.pixiv.net/en/artworks/[0-9]+").unwrap();

    for url in urls {
        THREAD_COUNT.fetch_add(1, Ordering::SeqCst);

        if !pixiv_url_re.is_match(&url) {
            warn!(
                "\"{}\" does not look like a valid Pixiv URL! Will attempt to download anyway.",
                url
            );
        }

        info!("Attempting to download {}", url);

        tokio::spawn(async move {
            if let Err(e) = extract_from_url(&url).await {
                error!("{}", e);
            }
            THREAD_COUNT.fetch_sub(1, Ordering::SeqCst);
        });
    }

    // Wait for all threads to finish
    while THREAD_COUNT.load(Ordering::SeqCst) > 0 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    Ok(())
}
