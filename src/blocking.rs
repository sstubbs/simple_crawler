use super::{Page, SimpleCrawler};
use anyhow::{Context, Result};

impl SimpleCrawler {
    #[cfg(feature = "blocking")]
    fn request_body_blocking(self) -> Result<Self> {
        let mut new = self;

        for (url, page) in new.urls.iter_mut() {
            if !page.crawled {
                let body = reqwest::blocking::get(url)
                    .with_context(|| format!("Request blocking failed GET request for {}", url))?
                    .text()
                    .with_context(|| {
                        format!("Request blocking failed to extract text for {}", url)
                    })?;
                *page = Page::with_values(&body, &true);
            }
        }

        Ok(new)
    }

    /// This is only available if the blocking feature has been enabled in this library. I
    /// recommend this isn't used and the `crawl` function be used instead as it's asynchronous
    /// and performs better.
    /// This can be used with `let simple_crawler = SimpleCrawler::new().url(&str).crawl_blocking()`
    #[cfg(feature = "blocking")]
    pub fn crawl_blocking(self) -> Result<Self> {
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
                .request_body_blocking()
                .with_context(|| format!("Crawl blocking failed with request"))?
                .get_urls()
                .with_context(|| format!("Crawl blocking failed with getting urls"))?;
        }
        Ok(new)
    }
}
