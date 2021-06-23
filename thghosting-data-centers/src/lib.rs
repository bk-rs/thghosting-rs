use std::{io::Error as IoError, time::Duration};

use isahc::{
    config::Configurable as _, AsyncReadResponseExt as _, Error as IsahcError, HttpClient,
};
use scraper::{Html, Selector};

const FETCH_URL: &str = "https://www.thghosting.com/network/data-centers/";

pub async fn fetch() -> Result<String, FetchError> {
    let client = HttpClient::builder()
        .timeout(Duration::from_secs(3))
        .build()?;

    let mut res = client.get_async(FETCH_URL).await?;

    let res_body_text = res.text().await?;

    Ok(res_body_text)
}

#[derive(thiserror::Error, Debug)]
pub enum FetchError {
    #[error("IsahcError {0:?}")]
    IsahcError(#[from] IsahcError),
    #[error("IoError {0:?}")]
    IoError(#[from] IoError),
}

#[derive(Debug, Clone)]
pub struct DataCenter {
    pub id: String,
    pub city: String,
}

pub fn parse(html: &str) -> Result<Vec<DataCenter>, ParseError> {
    let document = Html::parse_document(html);

    let selector = Selector::parse("div.location").unwrap();

    let mut data_centers = vec![];

    for element in document.select(&selector) {
        let id = element
            .value()
            .attr("id")
            .ok_or(ParseError::IdMissing)?
            .to_owned();

        let city_selector = Selector::parse(".dc-city").unwrap();

        let city = element
            .select(&city_selector)
            .next()
            .ok_or(ParseError::CityMissing)?
            .inner_html();

        data_centers.push(DataCenter { id, city });
    }

    Ok(data_centers)
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("IdMissing")]
    IdMissing,
    #[error("CityMissing")]
    CityMissing,
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::error;

    #[test]
    fn test_parse() -> Result<(), Box<dyn error::Error>> {
        let html = include_str!("../tests/data-centers.html");

        let data_centers = parse(html)?;

        println!("{:?}", data_centers);

        Ok(())
    }
}
