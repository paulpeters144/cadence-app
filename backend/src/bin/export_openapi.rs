use backend::ApiDoc;
use utoipa::OpenApi;
use std::fs;

fn main() {
    let openapi = ApiDoc::openapi();
    let json = openapi.to_pretty_json().unwrap();
    fs::write("openapi.json", json).expect("Unable to write openapi.json");
    println!("OpenAPI schema written to openapi.json");
}
