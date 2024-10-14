use samplify_rs::Sampleable;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map};

#[derive(Debug, Serialize, Deserialize, Sampleable)]
struct PaymentInstruction {
    currency: String,
    amount: f64,
}

fn main() -> Result<(), String> {
    let mut config = Map::new();

    config.insert("amount".to_string(), json!([10.0, 1000.0]));
    config.insert("currency".to_string(), json!(["USD", "EUR", "GBP"]));

    let sample_payment = PaymentInstruction::sample_with_config(&config)?;

    println!("{:?}", sample_payment);

    Ok(())
}