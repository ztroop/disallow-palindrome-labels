use lazy_static::lazy_static;

extern crate wapc_guest as guest;
use guest::prelude::*;
use k8s_openapi::api::core::v1 as apicore;
use std::collections::BTreeMap;

extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::{logging, protocol_version_guest, request::ValidationRequest, validate_settings};

mod settings;
use settings::Settings;

use slog::{info, o, warn, Logger};

lazy_static! {
    static ref LOG_DRAIN: Logger = Logger::root(
        logging::KubewardenDrain::new(),
        o!("policy" => "disallow-palindrome-labels")
    );
}

#[no_mangle]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
    register_function("validate_settings", validate_settings::<Settings>);
    register_function("protocol_version", protocol_version_guest);
}

fn has_palindrome_label(labels: BTreeMap<String, String>) -> Option<String> {
    for (key, _) in labels.into_iter() {
        if key == key.chars().rev().collect::<String>() {
            return Some(key);
        }
    }
    None
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<Settings> = ValidationRequest::new(payload)?;

    info!(LOG_DRAIN, "starting validation");

    match serde_json::from_value::<apicore::Pod>(validation_request.request.object) {
        Ok(pod) => {
            let name = pod.metadata.name.unwrap();
            let labels = match pod.metadata.labels {
                Some(l) => l,
                None => BTreeMap::new(),
            };

            match has_palindrome_label(labels) {
                Some(key) => {
                    info!(
                        LOG_DRAIN, "rejecting pod";
                        "name" => &name
                    );
                    kubewarden::reject_request(
                        Some(format!("pod {} with label {} is not accepted", &name, &key)),
                        None,
                    )
                }
                None => {
                    info!(LOG_DRAIN, "accepting resource");
                    kubewarden::accept_request()
                }
            }
        }
        Err(_) => {
            warn!(LOG_DRAIN, "cannot unmarshal resource");
            kubewarden::accept_request()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kubewarden_policy_sdk::test::Testcase;

    #[test]
    fn accept_pod_with_valid_labels() -> Result<(), ()> {
        let request_file = "test_data/pod_creation_valid_labels.json";
        let tc = Testcase {
            name: String::from("Valid labels"),
            fixture_file: String::from(request_file),
            expected_validation_result: true,
            settings: Settings {},
        };

        let res = tc.eval(validate).unwrap();
        assert!(
            res.mutated_object.is_none(),
            "Something mutated with test case: {}",
            tc.name,
        );

        Ok(())
    }

    #[test]
    fn accept_pod_with_no_labels() -> Result<(), ()> {
        let request_file = "test_data/pod_creation_no_labels.json";
        let tc = Testcase {
            name: String::from("Valid labels"),
            fixture_file: String::from(request_file),
            expected_validation_result: true,
            settings: Settings {},
        };

        let res = tc.eval(validate).unwrap();
        assert!(
            res.mutated_object.is_none(),
            "Something mutated with test case: {}",
            tc.name,
        );

        Ok(())
    }

    #[test]
    fn reject_pod_with_invalid_labels() -> Result<(), ()> {
        let request_file = "test_data/pod_creation_invalid_labels.json";
        let tc = Testcase {
            name: String::from("Invalid labels"),
            fixture_file: String::from(request_file),
            expected_validation_result: false,
            settings: Settings {},
        };

        let res = tc.eval(validate).unwrap();
        assert!(
            res.mutated_object.is_none(),
            "Something mutated with test case: {}",
            tc.name,
        );

        Ok(())
    }
}
