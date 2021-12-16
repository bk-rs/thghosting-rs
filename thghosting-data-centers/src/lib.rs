use std::net::Ipv4Addr;

pub mod html;

pub use html::{parse_html, HtmlEndpoint};

//
#[derive(Debug, Clone)]
pub struct DataCenter {
    pub id: String,
    pub city: String,
    pub available_services: Vec<AvailableService>,
    pub standard_bare_metal_bandwidth: Option<String>,
    pub ping: Option<Ipv4Addr>,
    pub test_download: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AvailableService {
    BareMetalServers,
    VirtualServers,
    PrivateCloud,
}
