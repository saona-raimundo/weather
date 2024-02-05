// Todo: parametrize all svg by the a common global parameter and definie to_y conversion accordingly

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
    daily: Daily,
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

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
struct Daily {
    time: Vec<String>,
    uv_index_max: Vec<f64>,
}

#[derive(Error, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[error("Failed to load data.\n{reason}")]
pub struct LoadError {
    reason: String,
}

impl Data {
    pub fn api_query(
        latitude: f64,
        longitude: f64,
        forecast_days: usize,) -> String {
        format!("https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=apparent_temperature,precipitation_probability,precipitation,wind_speed_10m,wind_direction_10m&forecast_days={}&daily=uv_index_max", latitude, longitude, forecast_days)
    }
    /// Load data from open-meteo.com
    pub async fn load(
        latitude: f64,
        longitude: f64,
        forecast_days: usize,
    ) -> Result<Self, LoadError> {
        let query = Self::api_query(
        latitude,
        longitude,
        forecast_days);
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
        let Data { hourly, daily, .. } = self;
        let Hourly {
            time,
            precipitation_probability,
            precipitation,
            apparent_temperature,
            ..
        } = hourly;
        let uv_index_max = daily.uv_index_max;
        let daily_time = daily.time;
        // let Daily {
        //     uv_index_max,
        //     time as daily_time
        //     ..
        // } = daily;
        // Precipitation
        let precipitation_with_probability = precipitation.into_iter().zip(precipitation_probability.into_iter()).collect::<Vec<_>>();
        // Wind todo


        view! {
            <div
                class="graph_container"
            >
                <div
                    class="svg_graph"
                >
                	<Precipitation
					    precipitation_with_probability = precipitation_with_probability
                        time = &time
					/>
                </div>
                <div
                    class="svg_graph"
                >
                	<Temperature
                		temperature = apparent_temperature
                        time = &time
                	/>
                </div>
                <div
                    class="svg_graph"
                >
                    <UV
                        uv_index_max = uv_index_max
                        daily_time = &daily_time
                    />
                </div>
            </div>
            <div>
                <p>"📅 " {time.first()} " - " {time.last()} </p>
            </div>
        }
        .into_view()
    }
}

#[component]
fn UV<'a>(uv_index_max: Vec<f64>, daily_time: &'a [String]) -> impl IntoView {
    const MAX_UV: f64 = 11.0; 
    const MIN_UV: f64 = 0.0; 
    const ENJOY_UV: f64 = 2.5;  // below this, You can safely enjoy being outside!
    const SEEK_UV: f64 = 7.5;   // below this, Seek shade during midday hours! Slip on a shirt, slop on sunscreen and slap on hat!
                                // above this, Avoid being outside during midday hours! Make sure you seek shade! Shirt, sunscreen and hat are a must! 
    const LOWER_MARGIN: f64 = 10.0;
    let uv_size = (daily_time.len()*24, 30.0 + LOWER_MARGIN); // MAX_UV - MIN_UV + LOWER_MARGIN);
    let uv_to_y = |uv: f64| {
        30.0 / MAX_UV * 
        (MAX_UV - uv)
            .min(MAX_UV - MIN_UV)
            .max(0.0)
    };
    let uv_color_high = (255, 0, 255);
    let uv_color_low = (0, 255, 0);
    let uv_to_color = |uv: f64| {
        format!(
            "color-mix(in hsl shorter hue, rgb({}, {}, {}) {}%, rgb({}, {}, {}))",
            uv_color_high.0,
            uv_color_high.1,
            uv_color_high.2,
            100.0 - uv_to_y(uv) / (uv_to_y(MAX_UV) - uv_to_y(MIN_UV)).abs() * 100.0,
            uv_color_low.0,
            uv_color_low.1,
            uv_color_low.2
        )
    };

    view! {
        <svg
            viewBox={ format!("0 0 {} {}", uv_size.0, uv_size.1) }
            xmlns="http://www.w3.org/2000/svg"
            width="100%"
        >
            // x-axis
            <text x="0" y=(uv_to_y(MAX_UV) + 2.0) font-size="2px">{format!("{MAX_UV}")}</text>
            <line 
                x1="0" 
                x2={uv_size.0} 
                y1={uv_to_y(MAX_UV)}
                y2={uv_to_y(MAX_UV)}
                stroke={uv_to_color(MAX_UV)}
                stroke-width="0.2"
            >
                <title>{format!("{MAX_UV}")}</title>
            </line>
            <text x="0" y=(uv_to_y(MIN_UV)) font-size="2px">{format!("{MIN_UV}")}</text>
            <line 
                x1="0" 
                x2={uv_size.0} 
                y1={uv_to_y(0.0)}
                y2={uv_to_y(0.0)}
                stroke={uv_to_color(0.0)}
                stroke-width="0.2"
            >
                <title>{format!("{MIN_UV}")}</title>
            </line>
            <text x="0" y=(uv_to_y(ENJOY_UV)) font-size="2px">{format!("{ENJOY_UV}")}</text>
            <line 
                x1="0" 
                x2={uv_size.0} 
                y1={uv_to_y(ENJOY_UV)}
                y2={uv_to_y(ENJOY_UV)}
                stroke={uv_to_color(ENJOY_UV)}
                stroke-width="0.2"
            >
                <title>{format!("{ENJOY_UV}")}</title>
            </line>
            <text x="0" y=(uv_to_y(SEEK_UV)) font-size="2px">{format!("{SEEK_UV}")}</text>
            <line 
                x1="0" 
                x2={uv_size.0} 
                y1={uv_to_y(SEEK_UV)}
                y2={uv_to_y(SEEK_UV)}
                stroke={uv_to_color(SEEK_UV)}
                stroke-width="0.2"
            >
                <title>{format!("{SEEK_UV}")}</title>
            </line>

            {
                uv_index_max
                .into_iter()
                .enumerate()
                .map(|(i, uv)| 
                    (0..24).map(|j| {
                        view! { 
                            <circle 
                                cx={i * 24 + j} 
                                cy={uv_to_y(uv)}
                                r="0.5" 
                                fill={uv_to_color(uv)}
                            >
                                <title>{uv}</title>
                            </circle>
                        }
                    }).collect_view()
                )
                .collect_view()
            }
        </svg>
        <h2>{"UV ☀"}</h2>      
    }
}


#[component]
fn Temperature<'a>(temperature: Vec<f64>, time: &'a [String]) -> impl IntoView {
    const MAX_TEMPERATURE: f64 = 30.; 
    const MIN_TEMPERATURE: f64 = -10.; 
    let temperature_size = (temperature.len(), MAX_TEMPERATURE - MIN_TEMPERATURE); // We limit to very hot and very cold
    let temperature_to_y = |temperature: f64| {
        (MAX_TEMPERATURE - temperature)
            .min(MAX_TEMPERATURE - MIN_TEMPERATURE)
            .max(0.0)
    };
    let temperature_color_high = (255, 0, 0);
    let temperature_color_low = (0, 0, 255);
    let temperature_to_color = |temperature: f64| {
        format!(
            "color-mix(in oklab, rgb({}, {}, {}) {}%, rgb({}, {}, {}))",
            temperature_color_high.0,
            temperature_color_high.1,
            temperature_color_high.2,
            100.0 - temperature_to_y(temperature) / (MAX_TEMPERATURE - MIN_TEMPERATURE) * 100.0,
            temperature_color_low.0,
            temperature_color_low.1,
            temperature_color_low.2
        )
    };

    view! {
	    <svg
	        viewBox={ format!("0 0 {} {}", temperature_size.0, temperature_size.1) }
	        xmlns="http://www.w3.org/2000/svg"
	        width="100%"
	    >
	        { 
	            let mut view = Vec::new();
	            let mut iter = temperature.iter();
	            let mut counter = 0;
	            let color = &temperature_to_color(MAX_TEMPERATURE);
	            while iter.nth(8).is_some() {
                    let day = &time.get(counter * 24).map(|s| &s[8..10]).unwrap_or("??");
	                view.push( view!{
                        <text x={8 + counter * 24} y=2.0 font-size="2px">{format!("{day}/8:00")}</text>
	                    <line x1={8 + counter * 24} x2={8 + counter * 24} y1=0 y2={temperature_size.1} stroke={color} stroke-width="0.1">
	                        <title>"8:00"</title>
	                    </line>
	                });
	                if iter.nth(11).is_some() {
	                    view.push( view!{
                            <text x={20 + counter * 24} y=2.0 font-size="2px">{format!("{day}/20:00")}</text>
	                        <line x1={20 + counter * 24} x2={20 + counter * 24} y1=0 y2={temperature_size.1} stroke={color} stroke-width="0.1">
	                            <title>"20:00"</title>
	                        </line>
	                    })
	                }
	                counter += 1;
	            }
	            view
	        }
	        // x-axis
            <text x="0" y=(temperature_to_y(MAX_TEMPERATURE) + 2.0) font-size="2px">{format!("{MAX_TEMPERATURE}°C")}</text>
	        <line 
	            x1="0" 
	            x2={temperature_size.0} 
	            y1={temperature_to_y(MAX_TEMPERATURE)}
	            y2={temperature_to_y(MAX_TEMPERATURE)}
	            stroke={temperature_to_color(MAX_TEMPERATURE)}
	            stroke-width="0.2"
	        >
	        	<title>{format!("{MAX_TEMPERATURE}°C")}</title>
	    	</line>
            <text x="0" y=(temperature_to_y(0.0)) font-size="2px">{format!("0°C")}</text>
	        <line 
	            x1="0" 
	            x2={temperature_size.0} 
	            y1={temperature_to_y(0.0)}
	            y2={temperature_to_y(0.0)}
	            stroke={temperature_to_color(0.0)}
	            opacity="0.5" 
	            stroke-width="0.1"
	        >
	        	<title>"0°C"</title>
	    	</line>
            <text x="0" y=(temperature_to_y(5.0)) font-size="2px">{format!("5°C")}</text>
	        <line 
	            x1="0" 
	            x2={temperature_size.0} 
	            y1={temperature_to_y(5.0)}
	            y2={temperature_to_y(5.0)}
	            stroke={temperature_to_color(5.0)}
	            opacity="0.8" 
	            stroke-width="0.1"
	        >
	        	<title>"5°C"</title>
	    	</line>
            <text x="0" y=(temperature_to_y(10.0)) font-size="2px">{format!("10°C")}</text>
	        <line 
	            x1="0" 
	            x2={temperature_size.0} 
	            y1={temperature_to_y(10.0)}
	            y2={temperature_to_y(10.0)}
	            stroke={temperature_to_color(10.0)}
	            opacity="0.8" 
	            stroke-width="0.1"
	        >
	        	<title>"10°C"</title>
	    	</line>
            <text x="0" y=(temperature_to_y(MIN_TEMPERATURE)) font-size="2px">{format!("{MIN_TEMPERATURE}°C")}</text>
	        <line 
	            x1="0" 
	            x2={temperature_size.0} 
	            y1={temperature_to_y(MIN_TEMPERATURE)} 
	            y2={temperature_to_y(MIN_TEMPERATURE)}
	            stroke={temperature_to_color(MIN_TEMPERATURE)} 
	            opacity="0.1" 
	            stroke-width="0.2"
	        >
	        	<title>{format!("{MIN_TEMPERATURE}°C")}</title>
	    	</line>
	        {temperature
	            .into_iter()
	            .enumerate()
	            .map(|(i, temperature)| 
	                view! { 
	                    <circle 
	                        cx={i} 
	                        cy={temperature_to_y(temperature)}
	                        r="0.5" 
	                        fill={temperature_to_color(temperature)}
	                    >
	                        <title>{temperature} "°C"</title>
	                    </circle>
	                }
	            )
	            .collect_view()
	        }
	    </svg>
	    <h2>{"Temperature 🌡"}</h2>    	
    }
}
#[component]
fn Precipitation<'a>(precipitation_with_probability: Vec<(f64, f64)>, time: &'a [String]) -> impl IntoView {
	const MAX_PRECIPITATION: f64 = 30.0; // mm
	const LOWER_MARGIN: f64 = 10.0; // mm

	let (precipitation, precipitation_probability): (Vec<_>, Vec<_>) = precipitation_with_probability.into_iter().unzip();
    let precipitation_size = (precipitation.len() as f64, MAX_PRECIPITATION + LOWER_MARGIN);
    let precipitation_to_y = |mm: f64| MAX_PRECIPITATION - mm.min(MAX_PRECIPITATION);
    let precipitation_color = (78, 104, 129);
    let color = &format!("rgb({}, {}, {})", precipitation_color.0, precipitation_color.1, precipitation_color.2);

    view! {
        <svg
            viewBox={ format!("0 0 {} {}", precipitation_size.0, precipitation_size.1) }
            xmlns="http://www.w3.org/2000/svg"
            width="100%"
        >
        	// Vertical lines
            {
                let mut view = Vec::new();
                let mut iter = precipitation.iter();
                let mut counter = 0;
                while iter.nth(8).is_some() {
                    let day = &time.get(counter * 24).map(|s| &s[8..10]).unwrap_or("??");
                    view.push( view!{
                        <text x={8 + counter * 24} y=2.0 font-size="2px">{format!("{day}/8:00")}</text>
                        <line x1={8 + counter * 24} x2={8 + counter * 24} y1=0 y2={precipitation_size.1} stroke={color} stroke-width="0.1">
                            <title>"8:00"</title>
                        </line>
                    });
                    if iter.nth(11).is_some() {
                        view.push( view!{
                            <text x={20 + counter * 24} y=2.0 font-size="2px">{format!("{day}/20:00")}</text>
                            <line x1={20 + counter * 24} x2={20 + counter * 24} y1=0 y2={precipitation_size.1} stroke={color} stroke-width="0.1">
                                <title>"20:00"</title>
                            </line>
                        })
                    }
                    counter += 1;
                }
                view
            }
            // Horizontal lines
            <text x="0" y=(precipitation_to_y(0.0)) font-size="2px">{format!("Nothing")}</text>
            <line x1="0" x2={precipitation_size.0} y1={precipitation_to_y(0.0)} y2={precipitation_to_y(0.0)} opacity={0.2} stroke={color} stroke-width="0.1">
            	<title>"Nothing"</title>
            </line>
            <text x="0" y=(precipitation_to_y(2.5)) font-size="2px">{format!("Light")}</text>
            <line x1="0" x2={precipitation_size.0} y1={precipitation_to_y(2.5)} y2={precipitation_to_y(2.5)} opacity={0.4} stroke={color} stroke-width="0.1">
            	<title>"Light"</title>
            </line>
            <text x="0" y=(precipitation_to_y(7.6)) font-size="2px">{format!("Moderate")}</text>
            <line x1="0" x2={precipitation_size.0} y1={precipitation_to_y(7.6)} y2={precipitation_to_y(7.6)} opacity={0.6} stroke={color} stroke-width="0.1">
            	<title>"Moderate"</title>
            </line>
            <text x="0" y=(precipitation_to_y(50.0) + 2.0) font-size="2px">{format!("Heavy")}</text>
            <line x1="0" x2={precipitation_size.0} y1={precipitation_to_y(50.0)} y2={precipitation_to_y(50.0)} stroke={color} stroke-width="0.1">
            	<title>"Heavy"</title>
            </line>
            // Point series
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
                            r={ if mm > 0.0 {0.5} else {0.3} }
                            fill={color}
                        >
                            <title>{mm} "mm with " {probability} "%"</title>
                        </circle>
                    }
                )
                .collect_view()
            }
        </svg>
        <h2>{"Precipitation 🌦"}</h2>
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
