use anyhow::{Context, Result};
use simple_crawler::SimpleCrawler;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct SimpleCrawlerMock {
    mock_server: MockServer,
}

impl SimpleCrawlerMock {
    async fn new() -> Result<Self> {
        Ok(SimpleCrawlerMock {
            mock_server: MockServer::start().await,
        })
    }

    async fn mock(self, meth: &str, url: &str, body: &str) -> Result<Self> {
        Mock::given(method(meth))
            .and(path(url))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&self.mock_server)
            .await;
        Ok(self)
    }
}

async fn setup_mocks() -> Result<String> {
    // start the mock server
    let mock = SimpleCrawlerMock::new()
        .await
        .with_context(|| format!("Failed to start mock server"))?;

    // url of mock server
    let mock_url = &mock.mock_server.uri().to_owned();

    // setup mocks
    mock
        // mock1
        .mock(
            "GET",
            "/crawl",
            format!(
                "<a href=\"{mock_url}/crawl2\">aaa</a>\
                <a href=\"{mock_url}/crawl3\">aaa</a>",
                mock_url = mock_url
            )
                .as_ref(),
        )
        .await
        .with_context(|| format!("Failed to add mock1"))?
        // mock 2
        .mock(
            "GET",
            "/crawl2",
            format!(
                "<a href=\"{mock_url}/crawl3\">aaa</a>\
                <a href=\"{mock_url}/crawl4\">aaa</a>",
                mock_url = mock_url
            )
                .as_ref(),
        )
        .await
        .with_context(|| format!("Failed to add mock2"))?
        // mock 3
        .mock(
            "GET",
            "/crawl3",
            format!(
                "<a href=\"{mock_url}/crawl5\">aaa</a>\
                <a href=\"{mock_url}/crawl6\">aaa</a>",
                mock_url = mock_url
            )
                .as_ref(),
        )
        .await
        .with_context(|| format!("Failed to add mock3"))?
        // mock 4
        .mock(
            "GET",
            "/crawl4",
            format!(
                "<a href=\"{mock_url}/crawl7\">aaa</a>\
                <a href=\"{mock_url}/crawl8\">aaa</a>",
                mock_url = mock_url
            )
                .as_ref(),
        )
        .await
        .with_context(|| format!("Failed to add mock3"))?;
    Ok(mock_url.to_owned())
}

fn mock_expected_results(mock_url: String) -> Vec<String> {
    let mut expected = vec![
        format!("{}/crawl", mock_url),
        format!("{}/crawl2", mock_url),
        format!("{}/crawl3", mock_url),
        format!("{}/crawl4", mock_url),
        format!("{}/crawl5", mock_url),
        format!("{}/crawl6", mock_url),
        format!("{}/crawl7", mock_url),
        format!("{}/crawl8", mock_url),
    ];
    expected.sort();
    expected
}

#[tokio::test]
async fn crawl_test() -> Result<()> {

    let mock_url = setup_mocks().await
        .with_context(|| format!("Failed to setup mock server"))?;

    // do crawl
    let simple_crawler = SimpleCrawler::new()
        .url(format!("{}/crawl", mock_url).as_str())?
        .crawl()
        .await;
    if simple_crawler.is_ok() {
        let expected = mock_expected_results(mock_url);
        let mut actual: Vec<String> = simple_crawler.unwrap().urls.keys().cloned().collect();
        actual.sort();
        assert_eq!(expected, actual);
    }

    Ok(())
}

#[tokio::test]
async fn crawl_concurrent_test() -> Result<()> {

    let mock_url = setup_mocks().await
        .with_context(|| format!("Failed to setup mock server"))?;

    // do crawl
    let simple_crawler = SimpleCrawler::new()
        .url(format!("{}/crawl", mock_url).as_str())?
        .crawl_concurrent(2)
        .await;
    if simple_crawler.is_ok() {
        let expected = mock_expected_results(mock_url);
        let mut actual: Vec<String> = simple_crawler.unwrap().urls.keys().cloned().collect();
        actual.sort();
        assert_eq!(expected, actual);
    }

    Ok(())
}

#[tokio::test]
async fn crawl_parallel_test() -> Result<()> {

    let mock_url = setup_mocks().await
        .with_context(|| format!("Failed to setup mock server"))?;

    // do crawl
    let simple_crawler = SimpleCrawler::new()
        .url(format!("{}/crawl", mock_url).as_str())?
        .crawl_parallel(2)
        .await;
    if simple_crawler.is_ok() {
        let expected = mock_expected_results(mock_url);
        let mut actual: Vec<String> = simple_crawler.unwrap().urls.keys().cloned().collect();
        actual.sort();
        assert_eq!(expected, actual);
    }

    Ok(())
}
