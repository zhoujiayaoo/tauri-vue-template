use serde_json::json;
use serde_json::Value;


pub  fn parse_jps_str(input: &str) -> Value {
    let mut json_array = Vec::new();

    for line in input.lines() {
        let mut parts = line.split_whitespace();
        if let (Some(pid), Some(name)) = (parts.next(), parts.next()) {
            if name != "Jps" {
                let json_object = json!({
                    "java_pid": pid,
                    "java_name": name
                });
                json_array.push(json_object);
            }
        }
    }

    json!(json_array)
}