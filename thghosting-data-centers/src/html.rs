use std::{
    net::{AddrParseError, Ipv4Addr},
    str,
};

use http_api_client_endpoint::{
    http::{Error as HttpError, Method},
    Body, Endpoint, Request, Response,
};
use scraper::{Html, Selector};

pub const URL: &str = "https://www.thghosting.com/network/data-centers/";

use crate::{AvailableService, DataCenter};

//
//
//
#[derive(Debug, Clone)]
pub struct HtmlEndpoint;

impl Endpoint for HtmlEndpoint {
    type RenderRequestError = HtmlEndpointError;

    type ParseResponseOutput = Vec<DataCenter>;
    type ParseResponseError = HtmlEndpointError;

    fn render_request(&self) -> Result<Request<Body>, Self::RenderRequestError> {
        let request = Request::builder()
            .method(Method::GET)
            .uri(URL)
            .body(vec![])
            .map_err(HtmlEndpointError::MakeRequestFailed)?;

        Ok(request)
    }

    fn parse_response(
        &self,
        response: Response<Body>,
    ) -> Result<Self::ParseResponseOutput, Self::ParseResponseError> {
        let html =
            str::from_utf8(response.body()).map_err(|_| HtmlEndpointError::ResponseBodyInvalid)?;

        parse_html(html).map_err(HtmlEndpointError::ParseHtmlError)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum HtmlEndpointError {
    #[error("MakeRequestFailed {0}")]
    MakeRequestFailed(HttpError),
    #[error("ResponseBodyInvalid")]
    ResponseBodyInvalid,
    #[error("ParseHtmlError {0}")]
    ParseHtmlError(ParseHtmlError),
}

//
//
//
pub fn parse_html(html: impl AsRef<str>) -> Result<Vec<DataCenter>, ParseHtmlError> {
    let document = Html::parse_document(html.as_ref());

    let location_selector = Selector::parse("div.location").unwrap();

    let mut data_centers = vec![];

    for location_element in document.select(&location_selector) {
        let id = location_element
            .value()
            .attr("id")
            .ok_or(ParseHtmlError::IdMissing)?
            .to_owned();

        let city_selector = Selector::parse(".dc-city").unwrap();
        let city = location_element
            .select(&city_selector)
            .next()
            .ok_or(ParseHtmlError::CityMissing)?
            .inner_html();

        let mut available_services: Vec<AvailableService> = vec![];
        let mut standard_bare_metal_bandwidth: Option<String> = None;
        let mut ping: Option<Ipv4Addr> = None;
        let mut test_download: Option<String> = None;

        let tr_selector = Selector::parse("table tr").unwrap();
        for tr_element in location_element.select(&tr_selector) {
            let td_selector = Selector::parse("td").unwrap();
            let mut td_element_iter = tr_element.select(&td_selector);
            let head_element = td_element_iter
                .next()
                .ok_or(ParseHtmlError::AttrElementInvalid)?;
            let _ = td_element_iter
                .next()
                .ok_or(ParseHtmlError::AttrElementInvalid)?;
            let value_element = td_element_iter
                .next()
                .ok_or(ParseHtmlError::AttrElementInvalid)?;
            if td_element_iter.next().is_some() {
                return Err(ParseHtmlError::AttrElementInvalid);
            }
            match head_element.inner_html().as_str() {
                "Available Services" => {
                    let a_selector = Selector::parse("a").unwrap();
                    for ele in value_element.select(&a_selector) {
                        if let Some(title) = ele.value().attr("title") {
                            match title {
                                "Bare Metal Servers" => {
                                    available_services.push(AvailableService::BareMetalServers)
                                }
                                "Virtual Servers" => {
                                    available_services.push(AvailableService::VirtualServers)
                                }
                                "Private Cloud" => {
                                    available_services.push(AvailableService::PrivateCloud)
                                }
                                _ => return Err(ParseHtmlError::AvailableServiceUnknown),
                            }
                        }
                    }
                }
                "Available Networks" => {}
                "Standard Bare Metal Bandwidth" => {
                    let s = value_element.inner_html();
                    match s.as_str() {
                        "" => {}
                        _ => {
                            standard_bare_metal_bandwidth = Some(s);
                        }
                    }
                }
                "Ping/Trace Route" => {
                    let s = value_element.inner_html();
                    match s.as_str() {
                        "-" | "" => {}
                        _ => {
                            let v = s
                                .parse()
                                .map_err(|err| ParseHtmlError::PingInvalid(s, err))?;
                            ping = Some(v);
                        }
                    }
                }
                "Certifications" => {}
                "Test Download" => {
                    let s = value_element.inner_html();
                    match s.as_str() {
                        "" => {}
                        _ => {
                            let a_selector = Selector::parse("a").unwrap();
                            if let Some(v) = value_element
                                .select(&a_selector)
                                .next()
                                .and_then(|ele| ele.value().attr("href"))
                                .map(ToOwned::to_owned)
                            {
                                test_download = Some(v);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        let url_selector = Selector::parse(".popover-container a").unwrap();
        let url = location_element
            .select(&url_selector)
            .next()
            .and_then(|ele| ele.value().attr("href"))
            .map(ToOwned::to_owned);

        data_centers.push(DataCenter {
            id,
            city,
            available_services,
            standard_bare_metal_bandwidth,
            ping,
            test_download,
            url,
        });
    }

    Ok(data_centers)
}

#[derive(thiserror::Error, Debug)]
pub enum ParseHtmlError {
    #[error("IdMissing")]
    IdMissing,
    #[error("CityMissing")]
    CityMissing,
    #[error("AttrElementInvalid")]
    AttrElementInvalid,
    #[error("AvailableServiceUnknown")]
    AvailableServiceUnknown,
    #[error("PingInvalid {0} {1}")]
    PingInvalid(String, AddrParseError),
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{env, fs, path::PathBuf};

    #[test]
    fn test_parse_html() {
        let manifest_path = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            PathBuf::from(&manifest_dir)
        } else {
            PathBuf::new()
        };

        let html_path_1 = manifest_path
            .join("thghosting-data-centers")
            .join("tests")
            .join("data-centers.html");
        let html_path = if html_path_1.exists() {
            html_path_1
        } else {
            manifest_path.join("tests").join("data-centers.html")
        };
        println!("html_path:{:?}", html_path);

        let html = match fs::read_to_string(html_path) {
            Ok(x) => x,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return,
            Err(err) => panic!("{}", err),
        };

        let data_centers = parse_html(html).unwrap();

        println!("{:?}", data_centers);

        let dc_london = data_centers.iter().find(|dc| dc.id == "london").unwrap();
        assert_eq!(dc_london.city, "London");
        assert_eq!(
            dc_london.available_services,
            vec![
                AvailableService::BareMetalServers,
                AvailableService::VirtualServers
            ]
        );
        assert_eq!(
            dc_london.standard_bare_metal_bandwidth,
            Some("100TB".to_owned())
        );
        assert_eq!(dc_london.ping, Some("82.163.78.28".parse().unwrap()));
        assert_eq!(
            dc_london.test_download,
            Some("http://82.163.78.28/speedtest.256mb".to_owned())
        );
        assert_eq!(
            dc_london.url,
            Some("https://info.ingenuitycloudservices.com/us/data-center/london".to_owned())
        );
    }
}
