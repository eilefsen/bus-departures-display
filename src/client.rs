use esp_idf_svc::{
    http::client::{EspHttpConnection, Response},
    io::Write as _,
    sys::EspError,
};
use std::error::Error;

use embedded_svc::http::client::Client;

use self::types::TopLevelData;

pub mod types;
pub mod util;

pub const SLEEP_SECONDS: u64 = 20;

pub struct EnturClient {
    url: &'static str,
    headers: Vec<(&'static str, &'static str)>,
    query: String,
    client: Client<EspHttpConnection>,
}

impl EnturClient {
    pub fn new(
        url: &'static str,
        headers: Vec<(&'static str, &'static str)>,
        query: String,
    ) -> Result<Self, EspError> {
        let conn = util::connection()?;
        let client = Client::wrap(conn);
        Ok(EnturClient {
            url,
            headers,
            query,
            client,
        })
    }

    pub fn write_request(&mut self) -> Result<Response<&mut EspHttpConnection>, Box<dyn Error>> {
        let json = serde_json::json!({"query": self.query});

        log::info!("query: {}", self.query);
        let mut request = self.client.post(self.url, &self.headers)?;
        request.write_fmt(format_args!("{}", json))?;
        let response = request.submit()?;
        Ok(response)
    }

    pub fn read_request(
        &self,
        mut response: Response<&mut EspHttpConnection>,
    ) -> Result<TopLevelData, Box<dyn Error>> {
        let mut buffer = [0; 2048];
        response.read(&mut buffer)?;
        let c = String::from_utf8_lossy(&buffer);
        let content = c.trim_matches('\0');
        log::info!("Response content: {:?}", content);
        let status = response.status();
        log::info!("Response status code: {}", status);

        let response_data: TopLevelData = serde_json::from_str(content)?;

        Ok(response_data)
    }

    pub fn read_write_request(&self) -> Result<TopLevelData, Box<dyn Error>> {
        let json = serde_json::json!({"query": self.query});
        let conn = util::connection()?;
        let mut client = Client::wrap(conn);

        log::info!("query: {}", self.query);
        let mut request = client.post(self.url, &self.headers)?;
        request.write_fmt(format_args!("{}", json))?;
        let mut response = request.submit()?;
        let mut buffer = [0; 2048];
        response.read(&mut buffer)?;
        let c = String::from_utf8_lossy(&buffer);
        let content = c.trim_matches('\0');
        log::info!("Response content: {:?}", content);
        let status = response.status();
        log::info!("Response status code: {}", status);

        let response_data: TopLevelData = serde_json::from_str(content)?;

        Ok(response_data)
    }
}
