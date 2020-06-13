use chrono::prelude::*;
use eval::eval;
use pad::{Alignment, PadStr};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use std::process::{Command, Stdio};

fn main() {
    match prompt(&["def", "calc", "rate", "wea", "time"].join("\n")) {
        Err(why) => println!("something wrong here: {}", why),
        Ok(msg) => {
            let mut vec = msg.split_whitespace().collect::<Vec<_>>();
            for _i in (vec.len()..4).rev() {
                vec.push("");
            }

            start(App {
                name: vec[0].to_string(),
                param1: vec[1].to_string(),
                param2: vec[2].to_string(),
                param3: vec[3].to_string(),
            })
            .unwrap();
        }
    }
}

fn prompt(cmd: &str) -> Result<String, io::Error> {
    let process = Command::new("dmenu")
        .args(&["-b", "-l", "20", "-fn", "Noto Sans Mono CJK SC:size=10"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    process.stdin.unwrap().write_all(cmd.as_bytes())?;

    let mut s = String::new();

    process.stdout.unwrap().read_to_string(&mut s)?;

    Ok(s)
}

fn req(url: &str) -> Result<ExchangeRate, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?.json::<ExchangeRate>()?;
    Ok(response)
}

fn command(name: &str, args: Vec<&str>) -> String {
    let result = Command::new(name).args(&args).output();

    String::from_utf8_lossy(&result.unwrap().stdout).to_string()
}

fn start(app: App) -> Result<String, io::Error> {
    match app.name.as_str() {
        "def" => translate(app),
        "rate" => rate(app),
        "calc" => calculator(app),
        "wea" => weather(app),
        "time" => timezone(app),
        _ => translate(app),
    }
}

fn translate(app: App) -> Result<String, io::Error> {
    let result = command("sdcv", vec![&app.param1]);
    prompt(
        &(Regex::new(r"\s\d\s").unwrap()).replace_all(
            &Regex::new("^-->.*$|^Found.*")
                .unwrap()
                .replace_all(&result, ""),
            "\n",
        ),
    )
}

fn rate(app: App) -> Result<String, io::Error> {
    let rate =
        req("http://data.fixer.io/api/latest?access_key=f961d54f7cbbc0e22f27c6b60fb6aadf").unwrap();
    let base_currency = rate.rates[&(app.param2.to_uppercase())];
    let target_currency = rate.rates[&(app.param3.to_uppercase())];
    let amount = app.param1.parse::<f32>().unwrap() * (target_currency / base_currency);
    prompt(&format!(
        "{} {} = {} {}",
        app.param1, app.param2, amount, app.param3
    ))
}

fn calculator(app: App) -> Result<String, io::Error> {
    prompt(&eval(&(app.param1)).unwrap().to_string())
}

fn weather(app: App) -> Result<String, io::Error> {
    let output = command(
        "ansiweather",
        vec!["-l", &app.param1, "-f", "5", "-a", "true", "-s", "true"],
    );
    let result = output.split("-").collect::<Vec<_>>().join("\n");
    println!("{}", result);
    prompt(&result)
}

fn timezone(_app: App) -> Result<String, io::Error> {
    let zones: HashMap<&str, i32> = [
        ("Otago", 12),
        ("Tokyo", 9),
        ("Shanghai", 8),
        ("Bangkok", 7),
        ("Vejle", 1),
        ("Newyork", -4),
        ("Chicago", -5),
        ("Cupertino", -7),
    ]
    .iter()
    .cloned()
    .collect();

    let utc = Utc::now();
    let result = zones
        .keys()
        .map(|city| {
            format!(
                "{} | {}",
                city.pad(10, ' ', Alignment::Right, true),
                utc.with_timezone(&FixedOffset::east(zones[city] * 3600))
                    .format("%b %d (%a) %H:%M %p %z")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    prompt(&result.to_string())
}

#[derive(Debug)]
struct App {
    name: String,
    param1: String,
    param2: String,
    param3: String,
}

#[derive(Deserialize, Debug)]
struct ExchangeRate {
    success: bool,
    timestamp: i64,
    base: String,
    date: String,
    rates: HashMap<String, f32>,
}
