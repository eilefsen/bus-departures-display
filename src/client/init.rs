use std::error::Error;

use esp_idf_svc::{
    http::client::{Configuration as HttpConfig, EspHttpConnection},
    sys::EspError,
};

use crate::CONFIG;

use super::RequestConfig;

pub fn client() {
    let cfg = RequestConfig {
        url: "https://api.entur.io/journey-planner/v3/graphql",
        headers: vec![
            ("content-type", "application/json"),
            ("ET-Client-Name", "eilefsen-entur_display"),
        ],
        query: format!(
            r#"{{
                trip1: {}, trip2: {}
            }}"#,
            make_query(CONFIG.from_place1, CONFIG.to_place1),
            make_query(CONFIG.from_place2, CONFIG.to_place2),
        ),
    };
}

fn make_query(from: &str, to: &str) -> String {
    format!(
        r#"
			trip(
				from: {{
					place: "{}"
				}},
				to: {{
					place: "{}"
				}},
				numTripPatterns: 4
				modes: {{
					accessMode: foot
					egressMode: foot
					transportModes: [{{
						transportMode: bus
						transportSubModes: [localBus]
					}}]
				}}
			) {{
				tripPatterns {{
					legs {{
						expectedStartTime
						line {{
							publicCode
						}}
					}}
				}}
			}}
		"#,
        from, to
    )
}

pub fn connection() -> Result<EspHttpConnection, EspError> {
    let cfg = HttpConfig {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    };
    EspHttpConnection::new(&cfg)
}
