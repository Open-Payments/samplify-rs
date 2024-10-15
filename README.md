# samplify-rs
***A Powerful and Flexible Sample Data Generator for Rust***

**samplify-rs** is a Rust library designed to simplify the process of generating sample data for testing, prototyping, and development purposes. Leveraging Rust’s powerful procedural macros and conditional compilation features, samplify-rs allows you to automatically generate realistic and customizable sample instances of your data structures without polluting your production code.

## Features

- Automatic Derivation: Use the Sampleable derive macro to automatically implement sample generation for your structs and enums.
- Field-Level Customization: Annotate your fields with attributes to specify value ranges, patterns, choices, lengths, and inclusion probabilities.
- Support for Complex Structures: Handle deeply nested and complex data structures with optional fields and variations effortlessly.
- Conditional Compilation: Enable or disable sample generation code using a feature flag, keeping your production builds clean and efficient.
- Extensibility: Easily integrate with existing projects and extend functionality without modifying original data structures.

## Getting Started

1. **Add samplify-rs to Your Project**

Add the following to your Cargo.toml:
```toml
[dependencies]
samplify-rs = "0.1.0"
```
2. **Include the macro in your code**
```rust
use samplify_rs::Sampleable;
```

3. **Annotate Your Data Structures**

Use `#[cfg_attr(feature = "sample", derive(Sampleable))]` on your structs and enums.

```rust

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "sample", derive(Sampleable))]
struct PaymentInstruction {
    currency: String,
    amount: f64,
}

```

## Key Benefits

- **Non-Intrusive**: Does not require modification of your production codebase; sample code is conditionally compiled.
- **Customizable Data Generation**: Fine-tune how sample data is generated for each field.
- **Improved Testing**: Quickly generate realistic test data to enhance your testing processes.
- **Lightweight**: Excludes sample generation code from production builds, ensuring optimal performance and binary size.

## Example

```rust
use samplify_rs::Sampleable;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize, Sampleable)]
struct PaymentInstruction {
    currency: String,
    amount: f64,
}

fn main() -> Result<(), String> {
    let config_json = r#"
    {
        "amount": [10.0, 1000.0],
        "currency": ["USD", "EUR", "GBP"]
    }
    "#;
    let config_map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(config_json).map_err(|e| e.to_string())?;
    let sample_payment = PaymentInstruction::sample(&config_map)?;

    println!("{:?}", sample_payment);

    Ok(())
}
```