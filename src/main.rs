use clap::Parser;
use colored::*;
use directories::BaseDirs;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;
use std::fs;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    city: String,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "api-key")]
    api_key: String,
}

#[derive(Deserialize)]
struct Weather {
    id: u32,
    main: String,
    description: String,
}

fn main() {
    let args = Args::try_parse().unwrap();

    let dirs = BaseDirs::new().expect("unable to initialize BaseDirs");
    let config_file = dirs.config_dir().join("weather-cli.conf");

    let config: Config = serde_yaml::from_str(
        &fs::read_to_string(&config_file).expect("could not read config file"),
    )
    .expect("could not deserialize config file");

    const GEO_URL: &str = "http://api.openweathermap.org/geo/1.0/direct";

    let client = Client::new();
    let params = [
        ("appid", config.api_key.as_str()),
        ("q", args.city.as_str()),
        ("limit", "1"),
    ];
    let res = client
        .post(GEO_URL)
        .json(&params)
        .send()
        .expect(&format!("could not get geodata about {}", args.city));

    let json_value: Value = res.json().expect("could not parse JSON data");
    let geo_data = &json_value[0];
    println!("{json_value}");
    let lat = geo_data["lat"].as_str().expect("lat value exists");
    let lon = geo_data["lon"].as_str().expect("lon value exists");

    // API documentation: https://openweathermap.org/current
    const WEATHER_URL: &str = "https://api.openweathermap.org/data/2.5/weather";

    let params = [
        ("appid", config.api_key.as_str()),
        ("lat", &lat),
        ("lon", &lon),
    ];
    let res = client
        .post(WEATHER_URL)
        .form(&params)
        .send()
        .expect(&format!("could not get weather for {}", args.city));

    let mut json_value: Value = res.json().expect("could not parse JSON data");
    let weather = json_value["weather"]
        .as_array_mut()
        .expect("could not find array")
        .pop()
        .expect("could not find weather");
    let weather: Weather = serde_json::from_value(weather).expect("could not find weather data");

    println!(
        "{}: {}",
        args.city.bold().magenta(),
        weather.main.italic().blue()
    );
    println!("{}", weather.description.bright_black());
}
