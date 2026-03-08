use chrono::Utc;

fn main() {
    // Get the current UTC time
    let now = Utc::now();
    
    // Format to ISO 8601 down to the minute (e.g., "2026-03-08T14:30Z")
    let build_date = now.format("%Y-%m-%dT%H:%MZ").to_string();
    
    // Expose the date as a compile-time environment variable
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);
}
