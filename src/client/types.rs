use std::error::Error;
use time::Duration;

use serde::Deserialize;
use time::{format_description::well_known::Iso8601, OffsetDateTime};

#[derive(Debug, Clone)]
pub struct Departure {
    pub start_time: OffsetDateTime,
    pub leaving_in: Duration,
    pub line_number: String,
}

impl Default for Departure {
    fn default() -> Departure {
        Departure {
            start_time: OffsetDateTime::now_utc(),
            leaving_in: Duration::new(0, 0),
            line_number: "".to_string(),
        }
    }
}

impl Departure {
    pub fn format_time(self) -> (i64, i64) {
        let minutes = self.leaving_in.whole_minutes();
        let seconds = self.leaving_in.whole_seconds().max(0);
        (minutes, seconds - (minutes * 60))
    }

    pub fn from_top_level_data(data: TopLevelData) -> Vec<Departure> {
        let mut departures = Departure::from_trip(data.data.trip1);
        departures.append(&mut Departure::from_trip(data.data.trip2));
        departures
    }
    fn from_trip(trip: Trip) -> Vec<Departure> {
        trip.trip_patterns
            .into_iter()
            .flat_map(|tp| tp.legs)
            .filter_map(|leg| Departure::from_leg(leg).ok())
            .collect()
    }
    fn from_leg(leg: Leg) -> Result<Departure, Box<dyn Error>> {
        let start = OffsetDateTime::parse(leg.expected_start_time.as_str(), &Iso8601::DEFAULT)?;
        let now = OffsetDateTime::now_utc();

        log::info!("{}", now);
        let diff = start - now;
        // let leaving = format!(
        //     "{}:{:02}",
        //     diff.whole_minutes(),
        //     diff.whole_seconds() - (diff.whole_minutes() * 60)
        // );

        match leg.line {
            Some(l) => Ok(Departure {
                start_time: start,
                leaving_in: diff,
                line_number: l.public_code,
            }),
            None => Err("Leg line is null".into()),
        }
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TopLevelData {
    data: Data,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct Data {
    trip1: Trip,
    trip2: Trip,
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Trip {
    trip_patterns: Vec<TripPattern>,
}
impl Default for Trip {
    fn default() -> Self {
        Self {
            trip_patterns: vec![TripPattern::default()],
        }
    }
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct TripPattern {
    legs: Vec<Leg>,
}
impl Default for TripPattern {
    fn default() -> Self {
        Self {
            legs: vec![Leg::default()],
        }
    }
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Leg {
    expected_start_time: String,
    line: Option<Line>,
}
impl Default for Leg {
    fn default() -> Self {
        Self {
            expected_start_time: "".to_string(),
            line: None,
        }
    }
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Line {
    public_code: String,
}
impl Default for Line {
    fn default() -> Self {
        Self {
            public_code: "0".to_string(),
        }
    }
}
