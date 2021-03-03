use clap::{Arg, App, SubCommand};
use reqwest::blocking::{Client, Response};


type AppId = str;

fn main() {
    //TODO command names in enums
    let matches = App::new("Drogue Command Line Tool")
        .version("0.1")
        .author("Jb Trystram <jbtrystram@redhat.com>")
        .about("Allows to manage drogue apps and devices in a drogue-cloud instance")
        .arg(Arg::with_name("URL")
            .short("u")
            .long("url")
            .value_name("URL")
            .help("The url of the registry endpoint")
            .takes_value(true)
            .required(true)
        ).subcommand(
        SubCommand::with_name("create")
            .about("create a resource in the drogue-cloud registry")
            .subcommand(
                SubCommand::with_name("device")
                    .about("create a device")
                    .arg(
                        Arg::with_name("id")
                            .required(true)
                            .help("The id of the device"),
                    )
                    .arg(
                        Arg::with_name("data")
                            .short("d")
                            .long("data")
                            .required(false)
                            .help("The data for the device"),
                    ),
            )
            .subcommand(
                SubCommand::with_name("app")
                    .about("create an app")
                    .arg(
                        Arg::with_name("id")
                            .required(true)
                            .help("The id for the app. This must be unique."),
                    )
            )
        ).subcommand(
                SubCommand::with_name("remove")
                    .about("delete a resource in the drogue-cloud registry")
                    .subcommand(
                        SubCommand::with_name("device")
                            .about("delete a device")
                            .arg(
                                Arg::with_name("id")
                                    .required(true)
                                    .help("The id of the device"),
                            )
                    )
                    .subcommand(
                        SubCommand::with_name("app")
                            .about("delete an app")
                            .arg(
                                Arg::with_name("id")
                                    .required(true)
                                    .help("The id for the app."),
                            )
                    )
    ).get_matches();

    //TODO wrap the string url into a proper url type, and fail early if the url is incorrect.
    let url = matches.value_of("URL").unwrap();

    match matches.subcommand() {
        ("create", Some(create_matches)) => {
            match create_matches.subcommand() {
                ("app", Some(app_matches)) => {
                    let id = app_matches.value_of("id").unwrap();

                    match create_app(url, id) {
                        Ok(r) => {
                            match r.status() {
                                reqwest::StatusCode::CREATED => println!("App {} created.", id),
                                r => println!("Error : {}", r),
                            }
                        },
                        Err(e) => println!("Error sending request : {}", e)
                    }
                },
                ("device", Some(dev_matches)) => {
                    println!("creating device {} not yet implemented", dev_matches.value_of("id").unwrap());
                }
                _ => unreachable!(),
            }
        },
        ("remove", Some(delete_matches)) => {
            match delete_matches.subcommand() {
                ("app", Some(app_matches)) => {
                    let id = app_matches.value_of("id").unwrap();

                    match delete_app(url, id) {
                        Ok(r) => {
                            match r.status() {
                                reqwest::StatusCode::NO_CONTENT => println!("App {} deleted.", id),
                                r => println!("Error : {}", r),
                            }
                        },
                        Err(e) => println!("Error sending request : {}", e)
                    }
                },
                ("device", Some(dev_matches)) => {
                    println!("deleting device {} not yet implemented", dev_matches.value_of("id").unwrap());
                }
                _ => unreachable!(),
            }
        }
        ("", None) => println!("No subcommand was used"),
        _ => unreachable!(),
    }

}


fn create_app(url: &str, app: &AppId) -> Result<Response, reqwest::Error> {
    let client = Client::new();
    let url = url.to_owned() + "/api/v1/apps";
    // todo use serdejson ?
    let body = format!("{{\"metadata\":{{\"name\":\"{}\"}}}}", app);

    client.post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body)
        .send()
}

fn delete_app(url: &str, app: &AppId) -> Result<Response, reqwest::Error> {
    let client = Client::new();
    let url = format!("{}{}", url.to_owned()+"/api/v1/apps/", app);


   client.delete(&url)
        .send()
}