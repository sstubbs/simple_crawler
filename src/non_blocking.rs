use super::{Page, SimpleCrawler};
use anyhow::{Context, Result};

impl SimpleCrawler {
    async fn request_body(self) -> Result<Self> {
        let mut new = self;

        for (url, page) in new.urls.iter_mut() {
            if !page.crawled {
                let body = reqwest::get(url)
                    .await
                    .with_context(|| format!("Request failed GET request for {}", url))?
                    .text()
                    .await
                    .with_context(|| format!("Request failed to extract text for {}", url))?;
                *page = Page::with_values(&body, &true);
            }
        }

        Ok(new)
    }

    /// asynchronous crawl gives better performance than it's `crawl_blocking` counterpart. I recommend this
    /// be used in it's place. Example use `let simple_crawler = SimpleCrawler::new().url(&str).crawl()`
    pub async fn crawl(self) -> Result<Self> {
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
                .request_body()
                .await
                .with_context(|| format!("Crawl failed with request"))?
                .get_urls()
                .with_context(|| format!("Crawl failed with getting urls"))?;
        }
        Ok(new)
    }
}
