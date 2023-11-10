use leptos::*;
use thiserror::Error;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Data {
    latitude: f64,
    longitude: f64,
    generationtime_ms: f64,
    utc_offset_seconds: f64,
    timezone: String,
    timezone_abbreviation: String,
    elevation: f64,
    hourly: Hourly,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
struct Hourly {
    time: Vec<String>,
    apparent_temperature: Vec<f64>,
    precipitation_probability: Vec<f64>,
    precipitation: Vec<f64>,
    wind_speed_10m: Vec<f64>,
    wind_direction_10m: Vec<f64>,
}

#[derive(Error, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[error("Failed to load data.\n{reason}")]
pub struct LoadError {
    reason: String,
}

impl Data {
    /// Load data from open-meteo.com
    pub async fn load(
        latitude: f64,
        longitude: f64,
        forecast_days: usize,
    ) -> Result<Self, LoadError> {
        let query = format!("https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=apparent_temperature,precipitation_probability,precipitation,wind_speed_10m,wind_direction_10m&forecast_days={}", latitude, longitude, forecast_days);
        log::debug!("{:?}", query);
        let resp = reqwest::get(query).await;
        let data = match resp {
            Ok(resp) => resp.json::<Data>().await.map_err(|e| LoadError {
                reason: format!("failed to parse forecast data.\nCauses:\n\n{}", e),
            }),
            Err(e) => Err(LoadError {
                reason: format!("failed to retrieve forecast data.\nCauses:\n\n{}", e),
            }),
        };
        data
    }
}

impl leptos::IntoView for Data {
    fn into_view(self) -> View {
        let Data { hourly, .. } = self;
        let Hourly { time, precipitation_probability, precipitation, apparent_temperature, .. }  = hourly;
        // Precipitation
        let precipitation_size = (time.len(), 8); // We limit at very heavy rain: 8 mm/hour
        let precipitation_to_y = |mm: f64| {8.0 - mm.min(8.0)};
        let precipitation_color = (78, 104, 129);
        // Temperature
        let max_temperature = 30.;
        let min_temperature = -10.;
        let temperature_size = (time.len(), max_temperature-min_temperature); // We limit to very hot (30) and very cold (-10)
        let temperature_to_y = |temperature: f64| { (30. - temperature).min(max_temperature).max(min_temperature) };
        let temperature_color_high = (255, 0, 0);
        let temperature_color_low = (0, 0, 255);
        let temperature_to_color = |temperature: f64| { 
            format!("color-mix(in oklab, rgb({}, {}, {}) {}%, rgb({}, {}, {}))",
                temperature_color_high.0, temperature_color_high.1, temperature_color_high.2,
                100.0 - temperature_to_y(temperature) / (max_temperature-min_temperature) * 100.0,
                temperature_color_low.0, temperature_color_low.1, temperature_color_low.2
            )
        };
        // Wind
        // todo

        view! {
            <div
                style="display: flex; align-items: flex-end; flex-wrap: wrap;"
            >
                <div
                    style="min-width: 20em;"
                >
                    <svg
                        viewBox={ format!("0 0 {} {}", precipitation_size.0, precipitation_size.1 - min_temperature.ceil() as isize) }
                        xmlns="http://www.w3.org/2000/svg"
                        width="100%"
                    >
                        <line x1="0" x2={precipitation_size.0} y1={precipitation_size.1} y2={precipitation_size.1} opacity={10.0 / 100.0} stroke={format!("rgb({}, {}, {})", precipitation_color.0, precipitation_color.1, precipitation_color.2)} stroke-width="1"/>
                        <line x1="0" x2={precipitation_size.0} y1="0" y2="0" opacity={20.0 / 100.0} stroke={format!("rgb({}, {}, {})", precipitation_color.0, precipitation_color.1, precipitation_color.2)} stroke-width="1"/>
                        {precipitation
                            .into_iter()
                            .zip(precipitation_probability.into_iter())
                            .enumerate()
                            .map(|(i, (mm, probability))| 
                                view! { 
                                    <circle 
                                        cx={i} 
                                        cy={precipitation_to_y(mm)} 
                                        opacity={probability / 100.0} 
                                        r="0.2" 
                                        fill={format!("rgb({}, {}, {})", precipitation_color.0, precipitation_color.1, precipitation_color.2)}
                                        stroke={format!("rgb({}, {}, {})", precipitation_color.0, precipitation_color.1, precipitation_color.2)}
                                    >
                                        <title>{mm} "mm with " {probability} "%"</title>
                                    </circle>
                                }
                            )
                            .collect_view()
                        }
                    </svg>
                    <h2>{"Precipitation 🌦"}</h2>
                </div>
                <div
                    style="min-width: 20em;"
                >
                    <svg
                        viewBox={ format!("0 0 {} {}", temperature_size.0, temperature_size.1) }
                        xmlns="http://www.w3.org/2000/svg"
                        width="100%"
                    >
                        // x-axis
                        <line 
                            x1="0" 
                            x2={temperature_size.0} 
                            y1="0" 
                            y2="0" 
                            stroke={temperature_to_color(max_temperature)}
                            stroke-width="0.2"
                        />
                        <line 
                            x1="0" 
                            x2={temperature_size.0} 
                            y1={max_temperature} 
                            y2={max_temperature} 
                            stroke={temperature_to_color(0.0)}
                            opacity="0.5" 
                            stroke-width="0.1"
                        />
                        <line 
                            x1="0" 
                            x2={temperature_size.0} 
                            y1={temperature_size.1} 
                            y2={temperature_size.1} 
                            stroke={temperature_to_color(min_temperature)} 
                            opacity="0.1" 
                            stroke-width="0.2"
                        />
                        {apparent_temperature
                            .into_iter()
                            .enumerate()
                            .map(|(i, temperature)| 
                                view! { 
                                    <circle 
                                        cx={i} 
                                        cy={temperature_to_y(temperature)}
                                        r="0.2" 
                                        fill={temperature_to_color(temperature)}
                                        stroke={temperature_to_color(temperature)}
                                    >
                                        <title>{temperature} "°C"</title>
                                    </circle>
                                }
                            )
                            .collect_view()
                        }
                    </svg>
                    <h2>{"Temperature 🌡"}</h2>
                </div>
            </div>
            // <div>
            //     <h2>{"Wind"}</h2>
            // </div>
        }
        .into_view()
    }
}

// const EMOJI: [char; 13] = [
//     '🗺', '🌡', '🧭', '🌣', '🌤', '🌥', '☁', '🌦', '⛈', '🌧', '🌩', '🌨', '🌪',
// ];

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that loading data can be performed.
    #[test]
    fn test_load_data() -> Result<(), LoadError> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _data = rt.block_on(Data::load(1., 1., 1))?;
        Ok(())
    }

    #[test]
    fn data_deserialization() -> Result<(), serde_json::Error> {
        let raw_data = r#"
{
  "latitude": 48.3,
  "longitude": 16.3,
  "generationtime_ms": 0.10895729064941406,
  "utc_offset_seconds": 0,
  "timezone": "GMT",
  "timezone_abbreviation": "GMT",
  "elevation": 305,
  "hourly_units": {
    "time": "iso8601",
    "apparent_temperature": "°C",
    "precipitation_probability": "%",
    "precipitation": "mm",
    "wind_speed_10m": "km/h",
    "wind_direction_10m": "°"
  },
  "hourly": {
    "time": [
      "2023-11-10T00:00",
      "2023-11-10T01:00",
      "2023-11-10T02:00",
      "2023-11-10T03:00",
      "2023-11-10T04:00",
      "2023-11-10T05:00",
      "2023-11-10T06:00",
      "2023-11-10T07:00",
      "2023-11-10T08:00",
      "2023-11-10T09:00",
      "2023-11-10T10:00",
      "2023-11-10T11:00",
      "2023-11-10T12:00",
      "2023-11-10T13:00",
      "2023-11-10T14:00",
      "2023-11-10T15:00",
      "2023-11-10T16:00",
      "2023-11-10T17:00",
      "2023-11-10T18:00",
      "2023-11-10T19:00",
      "2023-11-10T20:00",
      "2023-11-10T21:00",
      "2023-11-10T22:00",
      "2023-11-10T23:00",
      "2023-11-11T00:00",
      "2023-11-11T01:00",
      "2023-11-11T02:00",
      "2023-11-11T03:00",
      "2023-11-11T04:00",
      "2023-11-11T05:00",
      "2023-11-11T06:00",
      "2023-11-11T07:00",
      "2023-11-11T08:00",
      "2023-11-11T09:00",
      "2023-11-11T10:00",
      "2023-11-11T11:00",
      "2023-11-11T12:00",
      "2023-11-11T13:00",
      "2023-11-11T14:00",
      "2023-11-11T15:00",
      "2023-11-11T16:00",
      "2023-11-11T17:00",
      "2023-11-11T18:00",
      "2023-11-11T19:00",
      "2023-11-11T20:00",
      "2023-11-11T21:00",
      "2023-11-11T22:00",
      "2023-11-11T23:00"
    ],
    "apparent_temperature": [
      1.3,
      2.3,
      3.1,
      0.8,
      1.3,
      1.6,
      2,
      2.3,
      2.7,
      3.7,
      4.8,
      6.7,
      8.9,
      8.4,
      9.1,
      5.6,
      4,
      4.1,
      4.2,
      4.2,
      4.1,
      3.5,
      3.2,
      3.2,
      2.5,
      2.3,
      2.6,
      2.1,
      1.9,
      2.1,
      2.4,
      2.9,
      3,
      2,
      1.7,
      3,
      2.5,
      2.9,
      3.1,
      3.1,
      1.2,
      -0.4,
      -0.8,
      -1.1,
      -0.9,
      -1.5,
      -1.7,
      -1.8
    ],
    "precipitation_probability": [
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      5,
      11,
      16,
      19,
      23,
      26,
      38,
      49,
      61,
      63,
      66,
      68,
      60,
      53,
      45,
      31,
      17,
      3,
      3,
      3,
      3,
      2,
      1,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      3,
      7,
      10,
      8,
      5,
      3,
      3,
      3
    ],
    "precipitation": [
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0.6,
      0.5,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0.7,
      0.9,
      0.4,
      0.2,
      0.1,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0
    ],
    "wind_speed_10m": [
      5.7,
      3.7,
      4.1,
      2.8,
      4.2,
      4.5,
      7.7,
      8.1,
      10,
      13.3,
      18.3,
      17.9,
      11.3,
      10.9,
      5.1,
      14.1,
      17.8,
      17.3,
      15.9,
      14.5,
      13,
      14.4,
      14.8,
      12.6,
      15.1,
      14.8,
      12.3,
      13.7,
      14.4,
      11.9,
      10.1,
      8.7,
      12.6,
      21,
      24.9,
      21,
      24.2,
      23.7,
      22.7,
      17.4,
      20.6,
      24.9,
      24.6,
      24.5,
      22.3,
      24.1,
      24.1,
      25.6
    ],
    "wind_direction_10m": [
      252,
      299,
      285,
      130,
      149,
      151,
      139,
      111,
      131,
      147,
      158,
      170,
      158,
      136,
      219,
      266,
      262,
      265,
      267,
      264,
      264,
      267,
      271,
      270,
      271,
      271,
      275,
      268,
      271,
      270,
      268,
      275,
      272,
      264,
      272,
      275,
      275,
      279,
      293,
      283,
      276,
      274,
      276,
      274,
      273,
      267,
      268,
      268
    ]
  }
}
		"#;
        let _data: Data = serde_json::from_str(&raw_data)?;
        Ok(())
    }
}
