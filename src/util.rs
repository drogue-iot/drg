use crate::{Verbs};
use reqwest::blocking::Response;
use serde_json::{Value, Error, from_str};


pub fn print_result(res: Result<Response, reqwest::Error>, resource_name: String, op: Verbs) {
    match res {
        Ok(r) => {
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
                        r => println!("Error : {}", r),
                    }
                }
            }
        },
        Err(e) => println!("Error sending request : {}", e)
    }
}

pub fn json_parse(data: Option<&str>) -> Result<Value, Error> {
    from_str(data.unwrap_or("{}"))
}