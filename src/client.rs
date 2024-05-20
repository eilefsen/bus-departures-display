use esp_idf_svc::io::Write as _;
use std::error::Error;

use embedded_svc::http::client::Client;

use self::types::TopLevelData;

pub mod init;
pub mod types;

pub struct RequestConfig {
    url: &'static str,
    headers: Vec<(&'static str, &'static str)>,
    query: String,
}

fn read_write_request(cfg: RequestConfig) -> Result<TopLevelData, Box<dyn Error>> {
    let json = serde_json::json!({"query": cfg.query});
    let conn = init::connection()?;
    let mut client = Client::wrap(conn);

    log::info!("query: {}", cfg.query);
    let mut request = client.post(cfg.url, &cfg.headers)?;
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
