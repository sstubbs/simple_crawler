use super::{Page, SimpleCrawler};
use anyhow::{Context, Result};
use futures::{stream, StreamExt};
use reqwest::Client;

impl SimpleCrawler {
    async fn request_body_concurrent(self, concurrent_requests: usize) -> Result<Self> {
        let mut new = self;
        let client = Client::new();

        let bodies = stream::iter(new.urls.to_owned())
            .map(|(url, _)| {
                let client = &client;
                async move {
                    let resp = client.get(&url).send().await.with_context(|| {
                        format!("Request concurrent failed GET request for {}", url)
                    })?;
                    let text = resp.text().await.with_context(|| {
                        format!("Request concurrent failed to extract text for {}", url)
                    })?;
                    let result: Result<(String, String)> = Ok((url, text));
                    result
                }
            })
            .buffer_unordered(concurrent_requests)
            .collect::<Vec<_>>()
            .await;

        for (url, page) in new.urls.iter_mut() {
            if !page.crawled {
                let body = bodies
                    .iter()
                    // TODO a cleaner way of accessing this nested variable must be possible.
                    .filter(|b| b.as_ref().is_ok() && (b.as_ref().unwrap().0 == url.as_ref()))
                    .nth(0);
                // TODO a cleaner way of accessing this nested variable must be possible.
                if body.as_ref().is_some() && body.as_ref().unwrap().as_ref().is_ok() {
                    *page = Page::with_values(
                        body.as_ref().unwrap().as_ref().unwrap().1.as_ref(),
                        &true,
                    );
                }
            }
        }

        Ok(new)
    }

    /// Crawl concurrently. This is a good mix of good performance for high and medium amounts of urls.
    /// It can be used with `let simple_crawler = SimpleCrawler::new().url(&str).crawl_concurrent(usize)`
    /// The usize specifies how many concurrent requests are required.
    pub async fn crawl_concurrent(self, concurrent_requests: usize) -> Result<Self> {
        let mut new = self;

        // TODO need to better manage stack usage at this point.
        while new
            .to_owned()
            .urls
            .iter()
            .filter(|(_, page)| page.crawled == false)
            .count()
            > 0
        {
            new = new
                .to_owned()
                .request_body_concurrent(concurrent_requests)
                .await
                .with_context(|| format!("Crawl concurrent failed with request"))?
                .get_urls()
                .with_context(|| format!("Crawl concurrent failed with getting urls"))?;
        }
        Ok(new)
    }
}
