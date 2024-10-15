use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Sampleable)]
pub fn sampleable_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the struct name
    let struct_name = input.ident.clone();

    // Match only on structs with named fields
    let fields = if let Data::Struct(data_struct) = input.data {
        if let Fields::Named(fields_named) = data_struct.fields {
            fields_named.named
        } else {
            unimplemented!("Sampleable can only be derived for structs with named fields");
        }
    } else {
        unimplemented!("Sampleable can only be derived for structs");
    };

    // Generate sample code for each field
    let mut sample_fields = Vec::new();

    for field in fields.iter() {
        let field_name = &field.ident;
        let field_name_str = field_name.as_ref().unwrap().to_string();
        let field_type = &field.ty;

        // Generate sample code based on the field type
        let sample_code = match field_type {
            syn::Type::Path(type_path) => {
                let type_ident = &type_path.path.segments.last().unwrap().ident;
                let type_ident_str = type_ident.to_string();

                if ["f64", "f32"].contains(&type_ident_str.as_str()) {
                    // For floating-point numbers
                    quote! {
                        {
                            if let Some(config_value) = config.get(#field_name_str) {
                                if let Some(range_array) = config_value.as_array() {
                                    if range_array.len() == 2 {
                                        if let (Some(start), Some(end)) = (range_array[0].as_f64(), range_array[1].as_f64()) {
                                            rand::thread_rng().gen_range(start..end)
                                        } else {
                                            return Err(format!("Invalid range values for field '{}'", #field_name_str));
                                        }
                                    } else {
                                        return Err(format!("Range array for field '{}' must have exactly two elements", #field_name_str));
                                    }
                                } else {
                                    return Err(format!("Configuration for field '{}' must be an array", #field_name_str));
                                }
                            } else {
                                return Err(format!("Configuration for field '{}' is missing", #field_name_str));
                            }
                        }
                    }
                } else if ["i32", "i64", "usize", "u32", "u64"].contains(&type_ident_str.as_str()) {
                    // For integer numbers
                    quote! {
                        {
                            if let Some(config_value) = config.get(#field_name_str) {
                                if let Some(range_array) = config_value.as_array() {
                                    if range_array.len() == 2 {
                                        if let (Some(start), Some(end)) = (range_array[0].as_i64(), range_array[1].as_i64()) {
                                            rand::thread_rng().gen_range(start..end) as #field_type
                                        } else {
                                            return Err(format!("Invalid range values for field '{}'", #field_name_str));
                                        }
                                    } else {
                                        return Err(format!("Range array for field '{}' must have exactly two elements", #field_name_str));
                                    }
                                } else {
                                    return Err(format!("Configuration for field '{}' must be an array", #field_name_str));
                                }
                            } else {
                                return Err(format!("Configuration for field '{}' is missing", #field_name_str));
                            }
                        }
                    }
                } else if type_ident_str == "String" {
                    // For strings
                    quote! {
                        {
                            if let Some(config_value) = config.get(#field_name_str) {
                                if let Some(values_array) = config_value.as_array() {
                                    let values: Vec<String> = values_array.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect();
                                    if !values.is_empty() {
                                        values.choose(&mut rand::thread_rng()).unwrap().clone()
                                    } else {
                                        return Err(format!("Values array for field '{}' is empty", #field_name_str));
                                    }
                                } else {
                                    return Err(format!("Configuration for field '{}' must be an array", #field_name_str));
                                }
                            } else {
                                return Err(format!("Configuration for field '{}' is missing", #field_name_str));
                            }
                        }
                    }
                } else {
                    // Unsupported types default to Default::default()
                    quote! {
                        Default::default()
                    }
                }
            }
            _ => {
                // Unsupported types default to Default::default()
                quote! {
                    Default::default()
                }
            }
        };

        sample_fields.push(quote! {
            #field_name: #sample_code,
        });
    }

    // Generate the sample method
    let sample_method = quote! {
        impl #struct_name {
            pub fn sample(config: &serde_json::Map<String, serde_json::Value>) -> Result<Self, String> {
                use rand::Rng;
                use rand::seq::SliceRandom;

                Ok(Self {
                    #(#sample_fields)*
                })
            }
        }
    };

    // Return the generated code
    TokenStream::from(sample_method)
}