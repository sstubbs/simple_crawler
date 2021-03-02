mod blocking;
mod concurrent;
mod non_blocking;
mod parallel;
mod utils;

use anyhow::{Context, Result};
use select::document::Document;
use select::predicate::Name;
use std::collections::HashMap;

/// The SimpleCrawler struct is how this library is used for example:
/// `let simple_crawler = SimpleCrawler::new()`
#[derive(Clone, Debug, PartialEq)]
pub struct SimpleCrawler {
    base_url: String,
    pub urls: HashMap<String, Page>,
}

impl SimpleCrawler {
    /// Create a SimpleCrawler for example `let simple_crawler = SimpleCrawler::new()`
    pub fn new() -> Self {
        SimpleCrawler {
            base_url: "".to_owned(),
            urls: HashMap::new(),
        }
    }

    /// Adds a url to be crawled for example `let simple_crawler = SimpleCrawler::new().url(&str)`
    /// the first url is the base url. Subsequent urls will be ignored if not on the same domain as
    /// the first url or not relative paths.
    pub fn url(self, url: &str) -> Result<Self> {
        let mut new = self;

        // Base url needed in case relative paths are added.
        // and to not include external domains
        let new_base_url =
            utils::check_base_url(url).with_context(|| format!("Base URL invalid"))?;
        if new.base_url.is_empty() {
            new.base_url = new_base_url.to_owned();
        }

        // These are the lookup urls.
        let new_url = utils::normalise_url(&new_base_url, url);
        if new_url.is_some() {
            new.urls.insert(new_url.unwrap().to_owned(), Page::new());
        }

        Ok(new)
    }

    fn get_urls(self) -> Result<Self> {
        let base_url = self.base_url.to_owned();
        let mut new = self;

        for (_, page) in new.urls.to_owned().iter_mut() {
            Document::from(page.body.as_ref())
                .find(Name("a"))
                .filter_map(|n| n.attr("href"))
                .for_each(|v| {
                    let new_url = utils::normalise_url(&base_url, v);
                    if new_url.is_some() {
                        let nu = new_url.unwrap();
                        if !new.urls.contains_key(&nu) {
                            new.urls.insert(nu, Page::new());
                        }
                    }
                });
            // TODO need to better manage stack usage at this point.
            // Clear body variable as it's no longer needed once links have been extracted.
            *page = Page::with_values("", &true);
        }

        Ok(new)
    }
}

/// The Page struct stores all the text extracted from crawled web pages temporarily until urls have
/// been extracted and the status of whether the page has been crawled or not.
#[derive(Clone, Debug, PartialEq)]
pub struct Page {
    body: String,
    crawled: bool,
}

impl Page {
    fn new() -> Self {
        Page {
            body: "".to_owned(),
            crawled: false,
        }
    }

    fn with_values(body: &str, crawled: &bool) -> Self {
        Page {
            body: body.to_owned(),
            crawled: crawled.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Page;
    use super::SimpleCrawler;
    use anyhow::Result;
    use std::collections::HashMap;

    fn add_url_test_data(url: &str) -> SimpleCrawler {
        let mut urls = HashMap::new();
        urls.insert(
            url.to_owned(),
            Page {
                body: "".to_owned(),
                crawled: false,
            },
        );
        SimpleCrawler {
            base_url: format!("{}/", url.to_owned()),
            urls,
        }
    }

    fn get_urls_test_data(url: &str) -> SimpleCrawler {
        let mut urls = HashMap::new();
        urls.insert(
            url.to_owned(),
            Page {
                body: "<a href=\"https://test.com/test_url\">aaa</a>".to_owned(),
                crawled: false,
            },
        );
        SimpleCrawler {
            base_url: format!("{}/", url.to_owned()),
            urls,
        }
    }

    #[test]
    fn add_url_test() -> Result<()> {
        // test data
        let url = "https://test.com";
        let test_simple_creator = add_url_test_data(url);

        // created object
        let simple_crawler = SimpleCrawler::new().url(url)?;
        assert_eq!(test_simple_creator, simple_crawler);
        Ok(())
    }

    #[test]
    fn get_urls_test() -> Result<()> {
        // test data
        let url = "https://test.com";
        let mut test_simple_creator = get_urls_test_data(url);
        test_simple_creator.urls.insert(
            "https://test.com/test_url".to_owned(),
            Page {
                body: "".to_owned(),
                crawled: false,
            },
        );

        // created object
        let simple_creator = get_urls_test_data(url).get_urls()?;

        assert_eq!(test_simple_creator, simple_creator);

        Ok(())
    }
}
