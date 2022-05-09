use serde_json::Value;

pub fn remove_resource_version(mut value: Value) -> Value {

    let data = value.get_mut("metadata").unwrap().as_object_mut().unwrap();
    let _ = data.remove("resourceVersion");

    value
}