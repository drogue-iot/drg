use base64::encode;
use chrono::{Duration, Utc};
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa,
    KeyIdMethod, KeyPair,
};
use serde_json::{json, Value};
use std::fs::File;
use std::io::Write;
use std::{fs, process::exit, str::from_utf8};

#[allow(non_camel_case_types)]
enum CertificateType {
    app,
    device,
}

#[allow(non_camel_case_types)]
struct cert {
    certificate: Certificate,
}

impl cert {
    fn get_certificate(cert_type: CertificateType, comman_name: &str, days: i64) -> Self {
        let mut params = CertificateParams::new(vec!["Drogue Iot".to_owned()]);
        params.not_before = Utc::now();
        params.not_after = Utc::now() + Duration::days(days);
        params
            .distinguished_name
            .push(DnType::OrganizationName, "Drogue IoT".to_owned());
        params
            .distinguished_name
            .push(DnType::OrganizationalUnitName, "Cloud".to_owned());
        params
            .distinguished_name
            .push(DnType::CommonName, comman_name.to_owned());

        match cert_type {
            CertificateType::app => {
                params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
            }
            CertificateType::device => {
                params
                    .extended_key_usages
                    .push(ExtendedKeyUsagePurpose::ServerAuth);
                params
                    .extended_key_usages
                    .push(ExtendedKeyUsagePurpose::ClientAuth);
                params.key_identifier_method = KeyIdMethod::Sha256;
            }
        };

        Self {
            certificate: Certificate::from_params(params).unwrap(),
        }
    }
}

pub fn create_trust_anchor(app_id: &str, keyout: Option<&str>, days: i64) -> Value {
    let app_certificate = cert::get_certificate(CertificateType::app, app_id, days);

    let pem_cert = &app_certificate.certificate.serialize_pem().unwrap();
    log::debug!("Self-signed certificate generated.");

    let private_key = &app_certificate.certificate.serialize_private_key_pem();
    log::debug!("Private key extracted.");

    // Private key printed to terminal, when keyout argument not specified.
    match keyout {
        Some(file_name) => write_to_file(file_name, &private_key),
        _ => println!("{}", &private_key),
    };

    json!({
        "trustAnchors": {
            "anchors": [
                {
                    "certificate": encode(pem_cert)
                }
            ]
        }
    })
}

pub fn create_device_certificate(
    device_id: &str,
    ca_key: &str,
    ca_cert: &str,
    cert_key: Option<&str>,
    cert_out: Option<&str>,
    days: i64,
) {
    let ca_key_content: KeyPair;

    match KeyPair::from_pem(&read_from_file(ca_key)) {
        Ok(s) => {
            ca_key_content = s;
        }
        Err(e) => {
            log::error!("Error reading CA key file. {}", e);
            exit(1);
        }
    };

    let ca_base64 = base64::decode(&ca_cert).unwrap();
    let ca_cert_pem = from_utf8(&ca_base64).unwrap();

    let ca_certificate = CertificateParams::from_ca_cert_pem(&ca_cert_pem, ca_key_content)
        .map_err(|e| {
            log::error!("Error : {}", e);
            exit(1);
        })
        .unwrap();

    let deivce_temp = cert::get_certificate(CertificateType::device, &device_id, days);
    let ca_cert_fin = Certificate::from_params(ca_certificate).unwrap();

    // Signing the device certificate with CA
    let csr = deivce_temp
        .certificate
        .serialize_pem_with_signer(&ca_cert_fin)
        .unwrap();

    match cert_out {
        Some(file_name) => write_to_file(file_name, &csr),
        _ => println!("{}", &csr),
    };

    match cert_key {
        Some(file_name) => write_to_file(
            file_name,
            &deivce_temp.certificate.serialize_private_key_pem(),
        ),
        _ => println!("{}", &deivce_temp.certificate.serialize_private_key_pem()),
    };
}

fn write_to_file(file_name: &str, content: &str) {
    let mut file = File::create(file_name);
    match file.as_mut() {
        Ok(file) => match file.write_all(&content.as_bytes()) {
            Ok(_) => log::info!("File created."),
            Err(e) => log::error!("Error writing to file: {}", e),
        },
        Err(e) => log::error!("Error opening the file: {}", e),
    };
}

fn read_from_file(file_name: &str) -> String {
    match fs::read_to_string(file_name) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Error reading from {}: {}", file_name, e);
            exit(1);
        }
    }
}
