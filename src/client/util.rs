use esp_idf_svc::{
    http::client::{Configuration as HttpConfig, EspHttpConnection},
    sys::EspError,
};

pub fn make_query(from: &str, to: &str) -> String {
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
					transportModes: [{{
						transportMode: bus
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
