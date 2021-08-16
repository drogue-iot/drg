use anyhow::{anyhow, Result};
use base64::encode;
use chrono::{Duration, Utc};
use rand::rngs::OsRng;
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa,
    KeyIdMethod, KeyPair, PKCS_ECDSA_P256_SHA256, PKCS_ECDSA_P384_SHA384, PKCS_ED25519,
    PKCS_RSA_SHA256,
};
use rsa::{pkcs8::ToPrivateKey, RsaPrivateKey};
use serde_json::{json, Value};
use std::fs::File;
use std::io::Write;
use std::{fs, process::exit, str::from_utf8};
use strum_macros::{AsRefStr, EnumString};

pub const CERT_VALIDITY_DAYS: i64 = 365;

#[derive(AsRefStr, EnumString)]
pub enum SignAlgo {
    ECDSA,
    EdDSA,
    RSA,
    EdDSA384, // Not a key generation choice, only to support custom key of this type
}

#[allow(non_camel_case_types)]
enum CertificateType {
    app,
    device,
}

fn generate_certificate(
    cert_type: CertificateType,
    common_name: &str,
    organizational_unit: &str,
    key_pair_algorithm: Option<SignAlgo>,
    days: Option<&str>,
    key_input: Option<KeyPair>,
) -> Result<Certificate> {
    let mut params = CertificateParams::new(vec!["Drogue Iot".to_owned()]);

    let valid_for: i64 = match days {
        Some(d) => d.parse().unwrap(),
        _ => CERT_VALIDITY_DAYS,
    };

    params.not_before = Utc::now();
    params.not_after = Utc::now() + Duration::days(valid_for);
    params
        .distinguished_name
        .push(DnType::OrganizationName, "Drogue IoT".to_owned());
    params.distinguished_name.push(
        DnType::OrganizationalUnitName,
        organizational_unit.to_owned(),
    );
    params
        .distinguished_name
        .push(DnType::CommonName, common_name.to_owned());

    params.key_pair = key_input;

    params.alg = match key_pair_algorithm {
        Some(algo_name) => match algo_name {
            SignAlgo::ECDSA => &PKCS_ECDSA_P256_SHA256,
            SignAlgo::EdDSA => &PKCS_ED25519,
            SignAlgo::EdDSA384 => &PKCS_ECDSA_P384_SHA384,
            SignAlgo::RSA => {
                params.key_pair = params.key_pair.or(Some(generate_rsa_key()?));
                &PKCS_RSA_SHA256
            }
        },
        _ => &PKCS_ECDSA_P256_SHA256, // Default Signature algorithm
    };

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

    Certificate::from_params(params)
        .map_err(|e| anyhow!("Error Generating certificate for {} : {}", common_name, e))
}

pub fn create_trust_anchor(
    app_id: &str,
    keyout: Option<&str>,
    key_pair_algorithm: Option<SignAlgo>,
    days: Option<&str>,
    key_input: Option<KeyPair>,
) -> Result<Value> {
    const OU: &str = "Cloud";
    let is_input_key = key_input.is_some();

    let app_certificate = generate_certificate(
        CertificateType::app,
        app_id,
        OU,
        key_pair_algorithm,
        days,
        key_input,
    )?;

    let pem_cert = &app_certificate.serialize_pem()?;
    log::debug!("Self-signed certificate generated.");

    let private_key = &app_certificate.serialize_private_key_pem();
    log::debug!("Private key extracted.");

    // Private key printed to terminal, when keyout argument not specified.
    if !is_input_key {
        match keyout {
            Some(file_name) => write_to_file(file_name, &private_key, "App private key"),
            _ => {
                println!("Private key for an application is used to sign device certificates, see `drg trust add --help`\n");
                println!("{}", &private_key)
            }
        }
    };

    Ok(json!({
        "anchors": [
            {
                "certificate": encode(pem_cert)
            }
        ]
    }))
}

pub fn create_device_certificate(
    app_id: &str,
    device_id: &str,
    ca_key: &str,
    ca_cert: &str,
    cert_key: Option<&str>,
    cert_out: Option<&str>,
    key_pair_algorithm: Option<SignAlgo>,
    days: Option<&str>,
    key_input: Option<KeyPair>,
) -> Result<()> {
    let ca_key_content = KeyPair::from_pem(from_utf8(&read_from_file(ca_key)).unwrap_or_default())
        .or_else(|_| KeyPair::from_der(&read_from_file(ca_key)))
        .map_err(|e| anyhow!("Error reading CA key file. {}", e))?;

    let ca_base64 = base64::decode(&ca_cert)?;
    let ca_cert_pem = from_utf8(&ca_base64)?;

    let ca_certificate = CertificateParams::from_ca_cert_pem(&ca_cert_pem, ca_key_content)
        .map_err(|e| anyhow!("Error: {}", e))?;

    let ca_cert_fin = Certificate::from_params(ca_certificate)?;

    // Checking equality of public keys of Cert from application object and supplied CA key
    verify_public_key(ca_cert_pem, &ca_cert_fin.serialize_der()?)?;

    let is_input_key = key_input.is_some();

    let device_csr = generate_certificate(
        CertificateType::device,
        &device_id,
        &app_id,
        key_pair_algorithm,
        days,
        key_input,
    )?;

    // Signing the device certificate with CA
    let device_cert = device_csr.serialize_pem_with_signer(&ca_cert_fin)?;

    match cert_out {
        Some(file_name) => write_to_file(file_name, &device_cert, "Device certificate"),
        _ => {
            println!("This signed device certificate needs to be presented at the time of authentication.\n");
            println!("{}", &device_cert)
        }
    };

    if !is_input_key {
        match cert_key {
            Some(file_name) => write_to_file(
                file_name,
                &device_csr.serialize_private_key_pem(),
                "Device private key",
            ),
            _ => {
                println!(
                    "Device private key needs to be presented at the time of authentication.\n"
                );
                println!("{}", &device_csr.serialize_private_key_pem())
            }
        }
    };

    Ok(())
}

fn generate_rsa_key() -> Result<KeyPair> {
    const RSA_BIT_SIZE: usize = 2048;

    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, RSA_BIT_SIZE)?;
    let pkcs8_key = &private_key.to_pkcs8_der()?;

    KeyPair::from_der(pkcs8_key.as_ref()).map_err(|e| anyhow!("RSA key generation failed: {}", e))
}

pub fn verify_input_key(key_input: &str) -> Result<(KeyPair, SignAlgo)> {
    let key = KeyPair::from_der(&read_from_file(key_input))?;

    let alg = if key.is_compatible(&PKCS_RSA_SHA256) {
        SignAlgo::RSA
    } else if key.is_compatible(&PKCS_ECDSA_P256_SHA256) {
        SignAlgo::ECDSA
    } else if key.is_compatible(&PKCS_ED25519) {
        SignAlgo::EdDSA
    } else if key.is_compatible(&PKCS_ECDSA_P384_SHA384) {
        SignAlgo::EdDSA384
    } else {
        return Err(anyhow!("Unknown signature algorithm."));
    };

    Ok((key, alg))
}

fn verify_public_key(ca_cert: &str, local_cert: &[u8]) -> Result<()> {
    let ca_x509 = x509_parser::pem::parse_x509_pem(&ca_cert.as_bytes())?.1;
    let ca_x509_der = x509_parser::parse_x509_certificate(&ca_x509.contents)?.1;

    let local_certificate = x509_parser::parse_x509_certificate(&local_cert)?.1;

    let ca_public_key = ca_x509_der
        .tbs_certificate
        .subject_pki
        .subject_public_key
        .data;
    let local_public_key = local_certificate
        .tbs_certificate
        .subject_pki
        .subject_public_key
        .data;

    if ca_public_key.eq(local_public_key) {
        Ok(())
    } else {
        Err(anyhow!(
            "Invalid CA key: trust anchor and private key mismatch"
        ))
    }
}

fn write_to_file(file_name: &str, content: &str, resource_type: &str) {
    let mut file = File::create(file_name);
    match file.as_mut() {
        Ok(file) => match file.write_all(&content.as_bytes()) {
            Ok(_) => {
                println!(
                    "{} was successfully written to file {}.",
                    resource_type, file_name
                );
            }
            Err(e) => log::error!("Error writing to file: {}", e),
        },
        Err(e) => log::error!("Error opening the file: {}", e),
    };
}

fn read_from_file(file_name: &str) -> Vec<u8> {
    fs::read(file_name)
        .map_err(|e| {
            log::error!("Error reading from {}: {}", file_name, e);
            exit(1);
        })
        .unwrap()
}

#[cfg(test)]
mod trust_test {
    use super::*;
    use std::path::Path;

    const CERT: &str = "LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tDQpNSUlCb2pDQ0FVaWdBd0lCQWdJQktqQUtCZ2dxaGtqT1BRUURBakExTVE0d0RBWURWUVFEREFWaGNIQTBNVEVUDQpNQkVHQTFVRUNnd0tSSEp2WjNWbElFbHZWREVPTUF3R0ExVUVDd3dGUTJ4dmRXUXdIaGNOTWpFd09EQTVNRGN4DQpNelF3V2hjTk1qSXdPREE1TURjeE16UXdXakExTVE0d0RBWURWUVFEREFWaGNIQTBNVEVUTUJFR0ExVUVDZ3dLDQpSSEp2WjNWbElFbHZWREVPTUF3R0ExVUVDd3dGUTJ4dmRXUXdXVEFUQmdjcWhrak9QUUlCQmdncWhrak9QUU1CDQpCd05DQUFRRkNmcXh1bWZGU1pzTFFrelVrYVMzZUtyQ3RFcjhqbUtjWnJ1NGZWR2lXV1ZXSHdDbzZPQTdxbURwDQpPNlJscURWSERUUHpKYU9paEg2d0NmL21qc0habzBrd1J6QVZCZ05WSFJFRURqQU1nZ3BFY205bmRXVWdTVzkwDQpNQjBHQTFVZERnUVdCQlFzV3A3cnlNZ2lUdFVnYWk5WkNEOXBxTlAraWpBUEJnTlZIUk1CQWY4RUJUQURBUUgvDQpNQW9HQ0NxR1NNNDlCQU1DQTBnQU1FVUNJRmFpZFltZWdpRWhUV1pRTXQxYXhoKzd5SElXdTRIRVdtdjlPbmVKDQp6ZXRxQWlFQWhBS3EyWjhZZWVGa0pqTC95UnJ0ZlVxd0w1N3lDL2dQVHUwemZRNEJwczA9DQotLS0tLUVORCBDRVJUSUZJQ0FURS0tLS0tDQo=";

    #[test]
    fn test_create_trust_anchor() {
        let resp = create_trust_anchor("app10", Some("key.pem"), None, None, None).unwrap();
        assert!(
            resp["anchors"][0]["certificate"].is_string(),
            "Invalid JSON response."
        );
        assert!(
            Path::new("key.pem").is_file(),
            "Error exporting private key to file."
        );

        let cert_ca = resp["anchors"][0]["certificate"]
            .to_string()
            .replace("\"", "");

        let resp_cert_base64 = base64::decode(&cert_ca).unwrap();
        let resp_cert_pem = from_utf8(&resp_cert_base64).unwrap();

        assert!(
            x509_parser::pem::parse_x509_pem(resp_cert_pem.as_bytes()).is_ok(),
            "Invalid x509 certificate"
        );
    }

    #[test]
    fn test_create_device_certificate() {
        assert!(
            create_device_certificate(
                "app10",
                "d5",
                "keys/test-app-key.pem",
                CERT,
                Some("device-key.pem"),
                Some("device-cert.pem"),
                None,
                None,
                None
            )
            .is_ok(),
            "Unable to generate device certificate."
        );

        assert!(
            Path::new("device-key.pem").is_file(),
            "Error exporting private key to file."
        );

        assert!(
            Path::new("device-cert.pem").is_file(),
            "Error exporting certificate to file."
        );
    }

    #[test]
    fn test_key_certificate_mismatch() {
        assert!(
            create_device_certificate(
                "app10",
                "d5",
                "keys/test-incorrect-key.pem",
                CERT,
                None,
                None,
                None,
                None,
                None
            )
            .is_err(),
            "CA key and certificate mismatch should terminate with an error."
        );
    }

    #[test]
    fn test_rsa_key_cert_load() {
        let key_input = verify_input_key("keys/test-rsa-gen.pk8").unwrap();
        assert!(
            create_trust_anchor(
                "app40",
                None,
                Some(key_input.1),
                Some("256"),
                Some(key_input.0)
            )
            .is_ok(),
            "Adding custom RSA key failed."
        );
    }

    #[test]
    fn test_rsa_key_gen() {
        assert!(
            create_trust_anchor("app40", None, Some(SignAlgo::RSA), None, None).is_ok(),
            "RSA Key generation failed."
        );
    }
}
