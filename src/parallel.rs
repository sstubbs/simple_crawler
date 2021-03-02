use super::{Page, SimpleCrawler};
use anyhow::{Context, Result};
use futures::{stream, StreamExt};
use reqwest::Client;

impl SimpleCrawler {
    async fn request_body_parallel(self, parallel_requests: usize) -> Result<Self> {
        let mut new = self;
        let client = Client::new();

        let bodies = stream::iter(new.urls.to_owned())
            .map(|(url, _)| {
                let client = client.clone();
                tokio::spawn(async move {
                    let resp = client.get(&url).send().await.with_context(|| {
                        format!("Request parallel failed GET request for {}", url)
                    })?;
                    let text = resp.text().await.with_context(|| {
                        format!("Request parallel failed to extract text for {}", url)
                    })?;
                    let result: Result<(String, String)> = Ok((url, text));
                    result
                })
            })
            .buffer_unordered(parallel_requests)
            .collect::<Vec<_>>()
            .await;

        for (url, page) in new.urls.iter_mut() {
            if !page.crawled {
                let body = bodies
                    .iter()
                    // TODO a cleaner way of accessing this nested variable must be possible.
                    .filter(|b| {
                        (b.as_ref().is_ok() && b.as_ref().unwrap().is_ok())
                            && (b.as_ref().unwrap().as_ref().unwrap().0 == url.as_ref())
                    })
                    .nth(0);
                // TODO a cleaner way of accessing this nested variable must be possible.
                if body.as_ref().is_some() && body.as_ref().unwrap().as_ref().is_ok() {
                    *page = Page::with_values(
                        body.as_ref()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .1
                            .as_ref(),
                        &true,
                    );
                }
            }
        }

        Ok(new)
    }

    /// Crawls in parallel. For larger amounts of urls this can increase performance however there is
    /// overhead involved with creating new tokio tasks and for smaller amounts of work a
    /// standard `crawl` or `crawl_concurrent` may
    /// be a better option. It can be used with `let simple_crawler = SimpleCrawler::new().url(&str).crawl_parallel(usize)`
    /// The usize specifies how many parallel requests are required.
    pub async fn crawl_parallel(self, parallel_requests: usize) -> Result<Self> {
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
                .request_body_parallel(parallel_requests)
                .await
                .with_context(|| format!("Crawl parallel failed with request"))?
                .get_urls()
                .with_context(|| format!("Crawl parallel failed with getting urls"))?;
        }
        Ok(new)
    }
}
