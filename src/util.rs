use crate::{Verbs};
use reqwest::{blocking::Response, Url};
use serde_json::{Value, from_str};
use anyhow::{Result, Context};

pub fn print_result(r: Response, resource_name: String, op: Verbs) {
    match op {
        Verbs::create => {
            match r.status() {
                reqwest::StatusCode::CREATED => println!("{} created.", resource_name),
                r => println!("Error : {}", r),
            }
        }, Verbs::delete => {
            match r.status() {
                reqwest::StatusCode::NO_CONTENT => println!("{} deleted.", resource_name),
                r => println!("Error : {}", r),
            }
        }, Verbs::get => {
            match r.status() {
                reqwest::StatusCode::OK => println!("{}", r.text().expect("Empty response")),
                r => println!("Error : {}", r),
            }
        }, Verbs::edit => {
            match r.status() {
                reqwest::StatusCode::OK => println!("{} edited.", resource_name),
                r => println!("Error: {}", r),
            }
        }
    }
}

pub fn json_parse(data: Option<&str>) -> Result<Value> {
    let parsed = from_str(data.unwrap_or("{}")).with_context(|| format!("Can't parse data args: \'{}\' into json", data.unwrap()))?;
    Ok(parsed)
}

pub fn url_validation(url: Option<&str>) -> Result<Url> {
    let parse = Url::parse(url.unwrap()).with_context(|| format!("URL args: \'{}\' is not valid", url.unwrap()))?;
    Ok(parse)
}