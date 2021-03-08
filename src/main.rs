mod arguments;

use reqwest::blocking::{Client, Response};

use arguments::{Parameters, Verbs, Resources};
use std::str::FromStr;

type AppId = str;
type DeviceId = str;

fn main() {
    let matches = arguments::parse_arguments();

    //TODO wrap the string url into a proper url type, and fail early if the url is incorrect.
    let url = matches.value_of(Parameters::url).unwrap();

    let (cmd_name, cmd) = matches.subcommand();
    //deserialize the command into enum to take advantage of rust exhaustive match
    let verb = Verbs::from_str(cmd_name).unwrap();
    let (sub_cmd_name, sub_cmd) = cmd.unwrap().subcommand();
    let resource = Resources::from_str(sub_cmd_name).unwrap();
    let id = sub_cmd.unwrap().value_of(Parameters::id).unwrap();

    match verb {
        Verbs::create => {
            let data = sub_cmd.unwrap().value_of(Parameters::data);
            match resource {
                Resources::app => create_app(url, id, data),
                Resources::device => {
                    let app_id = sub_cmd.unwrap().value_of(Resources::app).unwrap();
                    create_device(url, id, data, app_id)
                },
            }
        }
        Verbs::delete => {
            match resource {
                Resources::app => delete_app(url, id),
                Resources::device => {
                    let app_id = sub_cmd.unwrap().value_of(Resources::app).unwrap();
                    delete_device(url, app_id, id)
                },
            }
        }
        Verbs::edit => {
            println!("uninmplemented")
        }
        Verbs::get => {
            println!("uninmplemented")
        }
    }
}


fn create_app(url: &str, app: &AppId, data: Option<&str>) {
    let client = Client::new();
    let url = url.to_owned() + "/api/v1/apps";
    // todo use serdejson and append data.
    let body = format!("{{\"metadata\":{{\"name\":\"{}\"}}}}", app);

    let res = client.post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body)
        .send();

    print_result(res, format!("App {}", app), Verbs::create)
}

    //TODO
fn create_device(url: &str, id: &DeviceId, data: Option<&str>, app_id: &AppId) {
    let client = Client::new();
    let url = format!("{}/api/v1/apps/{}/devices", url, app_id);
    // todo use serdejson and append data.
    let body = format!("{{\"metadata\":{{\"application\":\"{}\",\"name\":\"{}\"}}}}", app_id, id);

    let res = client.post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body)
        .send();

    print_result(res, format!("Device {}", id), Verbs::create)
}

fn delete_app(url: &str, app: &AppId) {
    let client = Client::new();
    let url = format!("{}/api/v1/apps/{}", url, app);

   let res = client.delete(&url).send();
   print_result(res, format!("App {}", app), Verbs::delete)
}

fn delete_device(url: &str, app: &AppId, device_id: &DeviceId) {
    let client = Client::new();
    let url = format!("{}/api/v1/apps/{}/devices/{}", url, app, device_id);

    let res = client.delete(&url).send();
    print_result(res, format!("Device {}", device_id), Verbs::delete)
}

fn print_result(res: Result<Response, reqwest::Error>, resource_name: String, op: Verbs) {
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