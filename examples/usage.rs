use samplify_rs::Sampleable;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize, Sampleable)]
enum Status {
    Active,
    Inactive,
    Suspended { reason: String },
}

#[derive(Debug, Serialize, Deserialize, Sampleable)]
struct Address {
    street: String,
    city: String,
    zipcode: String,
}

#[derive(Debug, Serialize, Deserialize, Sampleable)]
struct User {
    name: String,
    age: u32,
    address: Vec<Address>,
    email: Option<Box<Email>>,
    preferences: Vec<String>,
    status: Status,
}

#[derive(Debug, Serialize, Deserialize, Sampleable)]
struct Email {
    account: String,
    domain: String,
}

fn main() -> Result<(), String> {
    let config_json = r#"
    {
        "name": ["Alice", "Bob", "Charlie"],
        "age": [18, 65],
        "address": [
            {
                "street": ["123 Main St", "456 Elm St"],
                "city": ["New York", "Los Angeles"],
                "zipcode": ["10001", "90001"]
            },
            {
                "street": ["789 Oak St", "321 Pine St"],
                "city": ["Chicago", "San Francisco"],
                "zipcode": ["60601", "94101"]
            }
        ],
        "email": {
            "account": ["hari", "shankar", "harishankar"],
            "domain": ["gmail.com", "hotmail.com"]
        },
        "preferences": ["news", "updates", "offers", "events"],
        "status": {
            "variants": ["Active", "Inactive", "Suspended"],
            "variant_data": {
                "Suspended": {
                    "reason": ["Violation", "Payment Issue", "Other"]
                }
            }
        }
    }
    "#;

    // Parse the configuration
    let config_map: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(config_json).map_err(|e| e.to_string())?;

    // Generate a sample User
    let sample_user = User::sample_with_config(&config_map)?;

    println!("{:#?}", sample_user);

    Ok(())
}