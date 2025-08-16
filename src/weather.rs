use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;

pub struct WeatherClient {
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    properties: Properties,
}

#[derive(Debug, Deserialize)]
struct Properties {
    timeseries: Vec<TimeSeries>,
}

#[derive(Debug, Deserialize)]
struct TimeSeries {
    #[allow(dead_code)]
    time: String,
    data: TimeSeriesData,
}

#[derive(Debug, Deserialize)]
struct TimeSeriesData {
    instant: InstantData,
    #[serde(rename = "next_1_hours")]
    next_1_hours: Option<NextHours>,
    #[serde(rename = "next_6_hours")]
    next_6_hours: Option<NextHours>,
}

#[derive(Debug, Deserialize)]
struct InstantData {
    details: InstantDetails,
}

#[derive(Debug, Deserialize)]
struct InstantDetails {
    air_temperature: f64,
    wind_speed: f64,
    relative_humidity: f64,
    air_pressure_at_sea_level: f64,
    cloud_area_fraction: f64,
    wind_from_direction: f64,
}

#[derive(Debug, Deserialize)]
struct NextHours {
    summary: Summary,
    details: Option<NextHoursDetails>,
}

#[derive(Debug, Deserialize)]
struct Summary {
    symbol_code: String,
}

#[derive(Debug, Deserialize)]
struct NextHoursDetails {
    precipitation_amount: Option<f64>,
}


impl WeatherClient {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("yr-weather-mcp/0.1.0 (https://github.com/example/yr-weather-mcp)"),
        );
        
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");
            
        Self { client }
    }
    
    pub async fn get_weather_by_coords(&self, lat: f64, lon: f64, location_name: &str, forecast_type: &str) -> Result<String> {
        
        let url = format!(
            "https://api.met.no/weatherapi/locationforecast/2.0/compact?lat={:.4}&lon={:.4}",
            lat, lon
        );
        
        let response = self.client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<WeatherResponse>()
            .await?;
        
        match forecast_type {
            "current" => {
                // Get the current weather (first timeseries entry)
                let current = response.properties.timeseries
                    .first()
                    .ok_or_else(|| anyhow::anyhow!("No weather data available"))?;
                
                let details = &current.data.instant.details;
                
                // Get weather condition from next_1_hours or next_6_hours
                let default_symbol = "unknown".to_string();
                let weather_symbol = current.data.next_1_hours
                    .as_ref()
                    .or(current.data.next_6_hours.as_ref())
                    .map(|n| &n.summary.symbol_code)
                    .unwrap_or(&default_symbol);
                
                let weather_desc = get_weather_description(weather_symbol);
                
                // Get precipitation if available
                let precipitation = current.data.next_1_hours
                    .as_ref()
                    .and_then(|n| n.details.as_ref())
                    .and_then(|d| d.precipitation_amount)
                    .unwrap_or(0.0);
                
                let wind_direction = get_wind_direction(details.wind_from_direction);
                
                let result = format!(
                    "🌍 **Current Weather**\n\
                    📍 **Location:** {}\n\
                    🗺️ **Coordinates:** {:.4}°, {:.4}°\n\n\
                    🌡️ **Temperature:** {:.1}°C\n\
                    ☁️ **Condition:** {}\n\
                    💧 **Humidity:** {:.0}%\n\
                    🌬️ **Wind:** {:.1} m/s from {}\n\
                    🌧️ **Precipitation (1h):** {:.1} mm\n\
                    ☁️ **Cloud coverage:** {:.0}%\n\
                    🔵 **Air pressure:** {:.0} hPa\n\n\
                    *Data from YR.no (Norwegian Meteorological Institute)*",
                    location_name,
                    lat,
                    lon,
                    details.air_temperature,
                    weather_desc,
                    details.relative_humidity,
                    details.wind_speed,
                    wind_direction,
                    precipitation,
                    details.cloud_area_fraction,
                    details.air_pressure_at_sea_level
                );
                
                Ok(result)
            }
            "tomorrow" => {
                // Get tomorrow's weather (24 hours from now)
                let tomorrow_index = 24; // Hourly data, so 24 entries = 24 hours
                let tomorrow = response.properties.timeseries
                    .get(tomorrow_index)
                    .ok_or_else(|| anyhow::anyhow!("No forecast data for tomorrow"))?;
                
                let details = &tomorrow.data.instant.details;
                let default_symbol = "unknown".to_string();
                let weather_symbol = tomorrow.data.next_6_hours
                    .as_ref()
                    .map(|n| &n.summary.symbol_code)
                    .unwrap_or(&default_symbol);
                
                let weather_desc = get_weather_description(weather_symbol);
                let precipitation = tomorrow.data.next_6_hours
                    .as_ref()
                    .and_then(|n| n.details.as_ref())
                    .and_then(|d| d.precipitation_amount)
                    .unwrap_or(0.0);
                
                let result = format!(
                    "📅 **Tomorrow's Weather**\n\
                    📍 **Location:** {}\n\
                    🗺️ **Coordinates:** {:.4}°, {:.4}°\n\n\
                    🌡️ **Temperature:** {:.1}°C\n\
                    ☁️ **Condition:** {}\n\
                    💧 **Humidity:** {:.0}%\n\
                    🌬️ **Wind:** {:.1} m/s\n\
                    🌧️ **Precipitation (6h):** {:.1} mm\n\n\
                    *Data from YR.no (Norwegian Meteorological Institute)*",
                    location_name,
                    lat,
                    lon,
                    details.air_temperature,
                    weather_desc,
                    details.relative_humidity,
                    details.wind_speed,
                    precipitation
                );
                
                Ok(result)
            }
            "weekly" => {
                // Get weekly forecast (every 24 hours for 7 days)
                let mut forecast = format!(
                    "📆 **7-Day Weather Forecast**\n\
                    📍 **Location:** {}\n\
                    🗺️ **Coordinates:** {:.4}°, {:.4}°\n\n",
                    location_name, lat, lon
                );
                
                for day in 0..7 {
                    let index = day * 24; // 24 hours per day
                    if let Some(entry) = response.properties.timeseries.get(index) {
                        let details = &entry.data.instant.details;
                        let default_symbol = "unknown".to_string();
                        let weather_symbol = entry.data.next_6_hours
                            .as_ref()
                            .or(entry.data.next_1_hours.as_ref())
                            .map(|n| &n.summary.symbol_code)
                            .unwrap_or(&default_symbol);
                        
                        let weather_desc = get_weather_description(weather_symbol);
                        let precipitation = entry.data.next_6_hours
                            .as_ref()
                            .or(entry.data.next_1_hours.as_ref())
                            .and_then(|n| n.details.as_ref())
                            .and_then(|d| d.precipitation_amount)
                            .unwrap_or(0.0);
                        
                        let day_name = match day {
                            0 => "Today",
                            1 => "Tomorrow",
                            2 => "Day 3",
                            3 => "Day 4",
                            4 => "Day 5",
                            5 => "Day 6",
                            6 => "Day 7",
                            _ => "Unknown",
                        };
                        
                        forecast.push_str(&format!(
                            "**{}**: {} | 🌡️ {:.1}°C | 💧 {:.1}mm\n",
                            day_name,
                            weather_desc,
                            details.air_temperature,
                            precipitation
                        ));
                    }
                }
                
                forecast.push_str("\n*Data from YR.no (Norwegian Meteorological Institute)*");
                Ok(forecast)
            }
            _ => {
                // Default to current weather - duplicate the logic to avoid recursion
                let current = response.properties.timeseries
                    .first()
                    .ok_or_else(|| anyhow::anyhow!("No weather data available"))?;
                
                let details = &current.data.instant.details;
                let default_symbol = "unknown".to_string();
                let weather_symbol = current.data.next_1_hours
                    .as_ref()
                    .or(current.data.next_6_hours.as_ref())
                    .map(|n| &n.summary.symbol_code)
                    .unwrap_or(&default_symbol);
                
                let weather_desc = get_weather_description(weather_symbol);
                let precipitation = current.data.next_1_hours
                    .as_ref()
                    .and_then(|n| n.details.as_ref())
                    .and_then(|d| d.precipitation_amount)
                    .unwrap_or(0.0);
                
                let wind_direction = get_wind_direction(details.wind_from_direction);
                
                let result = format!(
                    "🌍 **Current Weather**\n\
                    📍 **Location:** {}\n\
                    🗺️ **Coordinates:** {:.4}°, {:.4}°\n\n\
                    🌡️ **Temperature:** {:.1}°C\n\
                    ☁️ **Condition:** {}\n\
                    💧 **Humidity:** {:.0}%\n\
                    🌬️ **Wind:** {:.1} m/s from {}\n\
                    🌧️ **Precipitation (1h):** {:.1} mm\n\
                    ☁️ **Cloud coverage:** {:.0}%\n\
                    🔵 **Air pressure:** {:.0} hPa\n\n\
                    *Data from YR.no (Norwegian Meteorological Institute)*",
                    location_name,
                    lat,
                    lon,
                    details.air_temperature,
                    weather_desc,
                    details.relative_humidity,
                    details.wind_speed,
                    wind_direction,
                    precipitation,
                    details.cloud_area_fraction,
                    details.air_pressure_at_sea_level
                );
                
                Ok(result)
            }
        }
    }
}

fn get_weather_description(symbol_code: &str) -> &str {
    match symbol_code {
        s if s.starts_with("clearsky") => "Clear sky ☀️",
        s if s.starts_with("fair") => "Fair 🌤️",
        s if s.starts_with("partlycloudy") => "Partly cloudy ⛅",
        s if s.starts_with("cloudy") => "Cloudy ☁️",
        s if s.starts_with("lightrain") => "Light rain 🌦️",
        s if s.starts_with("rain") => "Rain 🌧️",
        s if s.starts_with("heavyrain") => "Heavy rain ⛈️",
        s if s.starts_with("lightsnow") => "Light snow 🌨️",
        s if s.starts_with("snow") => "Snow ❄️",
        s if s.starts_with("heavysnow") => "Heavy snow 🌨️❄️",
        s if s.starts_with("fog") => "Fog 🌫️",
        _ => "Unknown",
    }
}

fn get_wind_direction(degrees: f64) -> &'static str {
    match degrees as i32 {
        d if d < 23 || d >= 338 => "North",
        d if d < 68 => "Northeast",
        d if d < 113 => "East",
        d if d < 158 => "Southeast",
        d if d < 203 => "South",
        d if d < 248 => "Southwest",
        d if d < 293 => "West",
        _ => "Northwest",
    }
}