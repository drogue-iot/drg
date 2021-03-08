mod arguments;

use arguments::{Parameters, Verbs, Resources};

use reqwest::blocking::{Client, Response};
use reqwest::Url;
use std::str::FromStr;
use serde_json::json;

type AppId = str;
type DeviceId = str;

fn main() {
    let matches = arguments::parse_arguments();

    //TODO : The error is not nice to read. 
    let url = Url::parse(matches.value_of(Parameters::url).unwrap()).expect("Invalid URL.");

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
                Resources::app => create_app(&url, id, data),
                Resources::device => {
                    let app_id = sub_cmd.unwrap().value_of(Resources::app).unwrap();
                    create_device(&url, id, data, app_id)
                },
            }
        }
        Verbs::delete => {
            match resource {
                Resources::app => delete_app(&url, id),
                Resources::device => {
                    let app_id = sub_cmd.unwrap().value_of(Resources::app).unwrap();
                    delete_device(&url, app_id, id)
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


fn create_app(url: &Url, app: &AppId, data: Option<&str>) {
    let client = Client::new();
    let url = format!("{}api/v1/apps", url);
    let body = json!({
        "metadata": {
            "name": app,
        },
        "spec": {
            "data": data.unwrap_or(""),
        }
    });

    let res = client.post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body.to_string())
        .send();

    print_result(res, format!("App {}", app), Verbs::create)
}

fn create_device(url: &Url, id: &DeviceId, data: Option<&str>, app_id: &AppId) {
    let client = Client::new();
    let url = format!("{}api/v1/apps/{}/devices", url, app_id);
    println!("{}", url);
    let body = json!({
        "metadata": {
            "name": id,
            "application": app_id
        },
        "spec": {
            "data": data.unwrap_or(""),
        }
    });
    let res = client.post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body.to_string())
        .send();

    print_result(res, format!("Device {}", id), Verbs::create)
}

fn delete_app(url: &Url, app: &AppId) {
    let client = Client::new();
    let url = format!("{}api/v1/apps/{}", url, app);

   let res = client.delete(&url).send();
   print_result(res, format!("App {}", app), Verbs::delete)
}

fn delete_device(url: &Url, app: &AppId, device_id: &DeviceId) {
    let client = Client::new();
    let url = format!("{}api/v1/apps/{}/devices/{}", url, app, device_id);

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