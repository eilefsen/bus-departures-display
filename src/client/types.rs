use std::error::Error;
use time::Duration;

use serde::Deserialize;
use time::{format_description::well_known::Iso8601, OffsetDateTime};

#[derive(Debug)]
pub struct Departure {
    pub start_time: OffsetDateTime,
    pub leaving_in: Duration,
    pub line_number: String,
}
impl Departure {
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

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TopLevelData {
    data: Data,
}
#[derive(Deserialize, Debug, Clone)]
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
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct TripPattern {
    legs: Vec<Leg>,
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Leg {
    expected_start_time: String,
    line: Option<Line>,
}
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Line {
    public_code: String,
}
