use vterm_protocol::{Request, Response, HandshakeRequest, HandshakeResponse};
use schemars::schema_for;
use std::fs;

fn main() {
    let schema_req = schema_for!(Request);
    let schema_res = schema_for!(Response);
    let schema_hs_req = schema_for!(HandshakeRequest);
    let schema_hs_res = schema_for!(HandshakeResponse);

    let output = serde_json::json!({
        "request": schema_req,
        "response": schema_res,
        "handshake_request": schema_hs_req,
        "handshake_response": schema_hs_res,
    });

    fs::write("docs/schema.json", serde_json::to_string_pretty(&output).unwrap()).unwrap();
    println!("Schema written to docs/schema.json");
}
