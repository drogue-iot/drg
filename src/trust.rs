use base64::encode;
use chrono::{Duration, Utc};
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, CertificateSigningRequest, CustomExtension,
    DnType, ExtendedKeyUsagePurpose, IsCa, KeyIdMethod, KeyPair,
};
use serde_json::{json, Value};
use std::fs::File;
use std::io::Write;
use std::{fs, process::exit, str::from_utf8};

#[allow(non_camel_case_types)]
struct cert {
    certificate: Certificate,
}

impl cert {
    fn get_certificate(params: CertificateParams) -> Self {
        Self {
            certificate: Certificate::from_params(params).unwrap(),
        }
    }
}

pub fn create_trust_anchor(app: &str, keyout: &str) -> Value {
    let mut params = CertificateParams::new(vec!["Drogue IoT".to_string()]);
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.not_before = Utc::now();
    params.not_after = Utc::now() + Duration::days(365);
    params
        .distinguished_name
        .push(DnType::OrganizationName, "Drogue IoT".to_owned());
    params
        .distinguished_name
        .push(DnType::CommonName, &app.to_owned());
    params
        .distinguished_name
        .push(DnType::OrganizationalUnitName, "Cloud".to_owned());

    let app_certificate = cert::get_certificate(params);

    let pem_cert = &app_certificate.certificate.serialize_pem().unwrap();
    log::debug!("Self-signed certificate generated.");

    let private_key = &app_certificate.certificate.serialize_private_key_pem();
    log::debug!("Private key extracted.");

    let mut app_key_file = File::create(keyout);
    match app_key_file.as_mut() {
        Ok(file) => match file.write_all(&private_key.as_bytes()) {
            Ok(_) => {
                log::debug!("Key exported to file")
            }
            Err(_) => {
                log::error!("Error writing key to file.")
            }
        },
        Err(e) => {
            log::error!("Error opening the file. {}", e)
        }
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
    cert_key: &str,
    cert_out: &str,
) {
    let ca_key_content: KeyPair;

    match fs::read_to_string(ca_key) {
        Ok(s) => match KeyPair::from_pem(&s.to_string()) {
            Ok(s) => {
                ca_key_content = s;
            }
            Err(e) => {
                log::error!("Error reading CA key file. {}", e);
                exit(1);
            }
        },
        Err(_) => {
            log::error!("Error reading CA key file.");
            exit(1);
        }
    };

    let ca_base64 = base64::decode(&ca_cert).unwrap();
    let ca_cert_pem = from_utf8(&ca_base64).unwrap();
    let ca_certificate = CertificateParams::from_ca_cert_pem(&ca_cert_pem, ca_key_content)
        .map_err(|e| {
            log::error!("Error: {}", e);
            exit(1);
        })
        .unwrap();

    let mut params = CertificateParams::new(vec![device_id.to_owned()]);
    params.is_ca = IsCa::SelfSignedOnly;
    params.not_before = Utc::now();
    params.not_after = Utc::now() + Duration::days(365);
    params
        .distinguished_name
        .push(DnType::OrganizationName, "Drogue IoT".to_owned());
    params
        .distinguished_name
        .push(DnType::CommonName, &device_id.to_owned());
    params
        .distinguished_name
        .push(DnType::OrganizationalUnitName, "Cloud".to_owned());

    // const OID_EXT_KEY_USAGE :&[u64] = &[2, 5, 29, 15];
    // const DIGITAL_SIGNATURE: &[u8] = &[2, 5, 29, 15, 4];

    // params.custom_extensions.push(CustomExtension::from_oid_content(OID_EXT_KEY_USAGE, DIGITAL_SIGNATURE.to_vec()));
    // params.custom_extensions.push(CustomExtension::from_oid_content(OID_EXT_KEY_USAGE, KEY_AGREEMENT.to_vec()));
    params
        .extended_key_usages
        .push(ExtendedKeyUsagePurpose::ServerAuth);
    params
        .extended_key_usages
        .push(ExtendedKeyUsagePurpose::ClientAuth);

    params.key_identifier_method = KeyIdMethod::Sha256;

    let deivce_temp = cert::get_certificate(params);
    let ca_cert_fin = cert::get_certificate(ca_certificate);

    let csr = deivce_temp
        .certificate
        .serialize_pem_with_signer(&ca_cert_fin.certificate)
        .unwrap();

    write(cert_out, &csr);
    write(
        cert_key,
        &deivce_temp.certificate.serialize_private_key_pem(),
    );
}

fn write(file_name: &str, content: &str) {
    let mut app_key_file = File::create(file_name);
    match app_key_file.as_mut() {
        Ok(file) => match file.write_all(&content.as_bytes()) {
            Ok(_) => {
                log::debug!("File created. ")
            }
            Err(_) => {
                log::error!("Error writing to file.")
            }
        },
        Err(e) => {
            log::error!("Error opening the file. {}", e)
        }
    };
}
