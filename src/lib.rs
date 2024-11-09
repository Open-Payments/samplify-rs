use proc_macro::TokenStream;
use quote::{quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

#[proc_macro_derive(Sampleable)]
pub fn sampleable_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct or enum.
    let name = input.ident.clone();

    // Match on the data type: struct or enum
    match input.data {
        Data::Struct(data_struct) => {
            // Handle structs
            expand_struct(name, data_struct)
        },
        Data::Enum(data_enum) => {
            // Handle enums
            expand_enum(name, data_enum)
        },
        _ => {
            unimplemented!("Sampleable can only be derived for structs and enums");
        }
    }
}

fn expand_struct(name: syn::Ident, data_struct: syn::DataStruct) -> TokenStream {
    // Extract the fields from the struct.
    let fields = match data_struct.fields {
        Fields::Named(fields_named) => fields_named.named,
        _ => unimplemented!("Sampleable can only be derived for structs with named fields"),
    };

    // Generate code for each field.
    let field_samples = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let field_type = &field.ty;

        let sample_code = generate_sample_code(field_type, &field_name_str, &quote!(config));

        quote! {
            #field_name: #sample_code
        }
    });

    // Generate the sample_with_config method.
    let expanded = quote! {
        impl #name {
            pub fn sample_with_config(config: &serde_json::Map<String, serde_json::Value>) -> Result<Self, String> {
                use rand::Rng;
                use rand::seq::SliceRandom;
                use serde_json::Value;

                Ok(Self {
                    #(#field_samples),*
                })
            }
        }
    };

    // Return the generated code.
    TokenStream::from(expanded)
}

fn expand_enum(name: syn::Ident, data_enum: syn::DataEnum) -> TokenStream {
    // Get the variants
    let variants = data_enum.variants;

    // Generate code to randomly select a variant
    let variant_names = variants.iter().map(|v| v.ident.clone());

    let variant_sample_cases = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_name_str = variant_name.to_string();

        match &variant.fields {
            Fields::Unit => {
                // Unit variant, no fields
                quote! {
                    #variant_name_str => {
                        #name::#variant_name
                    }
                }
            },
            Fields::Named(fields_named) => {
                // Struct variant
                let field_samples = fields_named.named.iter().map(|field| {
                    let field_name = &field.ident;
                    let field_name_str = field_name.as_ref().unwrap().to_string();
                    let field_type = &field.ty;

                    let sample_code = generate_sample_code(field_type, &field_name_str, &quote!(variant_data));

                    quote! {
                        #field_name: #sample_code
                    }
                });

                quote! {
                    #variant_name_str => {
                        if let Some(Value::Object(variant_data)) = variant_config.get(#variant_name_str) {
                            #name::#variant_name {
                                #(#field_samples),*
                            }
                        } else {
                            return Err(format!("Configuration for variant '{}' is missing or invalid", #variant_name_str));
                        }
                    }
                }
            },
            Fields::Unnamed(fields_unnamed) => {
                // Tuple variant
                let field_samples = fields_unnamed.unnamed.iter().enumerate().map(|(i, field)| {
                    let field_name_str = format!("field{}", i);
                    let field_type = &field.ty;

                    let sample_code = generate_sample_code(field_type, &field_name_str, &quote!(variant_data));

                    quote! {
                        #sample_code
                    }
                });

                quote! {
                    #variant_name_str => {
                        if let Some(Value::Object(variant_data)) = variant_config.get(#variant_name_str) {
                            #name::#variant_name(
                                #(#field_samples),*
                            )
                        } else {
                            return Err(format!("Configuration for variant '{}' is missing or invalid", #variant_name_str));
                        }
                    }
                }
            },
        }
    });

    // Generate the sample_with_config method for the enum
    let expanded = quote! {
        impl #name {
            pub fn sample_with_config(config: &serde_json::Map<String, serde_json::Value>) -> Result<Self, String> {
                use rand::Rng;
                use rand::seq::SliceRandom;
                use serde_json::Value;

                // Get the list of allowed variants from the config
                let variants: Vec<String> = if let Some(Value::Array(variant_array)) = config.get("variants") {
                    variant_array.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
                } else {
                    {
                        let mut vec = Vec::new();
                        #(
                            vec.push(String::from(stringify!(#variant_names)));
                        )*
                        vec
                    }
                };

                if variants.is_empty() {
                    return Err("No variants specified for enum sampling".to_string());
                }

                let selected_variant = variants.choose(&mut rand::thread_rng()).unwrap();

                // Get the 'variant_data' from the config
                let variant_config = if let Some(Value::Object(map)) = config.get("variant_data") {
                    map
                } else {
                    &serde_json::Map::new()
                };

                let result = match selected_variant.as_str() {
                    #(#variant_sample_cases),*,
                    _ => return Err(format!("Variant '{}' is not recognized", selected_variant)),
                };

                Ok(result)
            }
        }
    };

    TokenStream::from(expanded)
}

// Helper function to generate sample code based on the field type.
fn generate_sample_code(field_type: &Type, field_name_str: &str, config_var: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    if is_option(field_type) {
        let inner_type = get_inner_type(field_type);
        let inner_sample_code = generate_sample_code(&inner_type, field_name_str, config_var);

        quote! {
            {
                if let Some(config_value) = #config_var.get(#field_name_str) {
                    if config_value.is_null() {
                        None
                    } else {
                        Some(#inner_sample_code)
                    }
                } else {
                    None
                }
            }
        }
    } else if is_vec(field_type) {
        let inner_type = get_inner_type(field_type);
        let inner_sample_code = generate_sample_code_for_vec_elements(&inner_type, field_name_str, config_var);

        quote! {
            {
                #inner_sample_code
            }
        }
    } else if is_box(field_type) {
        let inner_type = get_inner_type(field_type);
        let inner_sample_code = generate_sample_code(&inner_type, field_name_str, config_var);
        
        quote! {
            Box::new(#inner_sample_code)
        }
    } else if is_primitive(field_type) {
        generate_primitive_sample_code(field_type, field_name_str, config_var)
    } else {
        // Assume it's a nested struct or enum that implements Sampleable.
        quote! {
            {
                if let Some(Value::Object(map)) = #config_var.get(#field_name_str) {
                    <#field_type>::sample_with_config(map)?
                } else {
                    return Err(format!("Configuration for '{}' must be an object", #field_name_str));
                }
            }
        }
    }
}

fn generate_sample_code_for_vec_elements(element_type: &Type, field_name_str: &str, config_var: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    if is_primitive(element_type) {
        // For Vec of primitive types, pick random elements
        let element_type_str = match element_type {
            Type::Path(type_path) => {
                type_path.path.segments.last().unwrap().ident.to_string()
            },
            _ => "".to_string(),
        };
        let parse_value = match element_type_str.as_str() {
            "String" => quote! {
                v.as_str().map(|s| s.to_string())
            },
            "i32" | "i64" | "u32" | "u64" | "usize" | "isize" => quote! {
                v.as_i64().map(|n| n as #element_type)
            },
            "f32" | "f64" => quote! {
                v.as_f64().map(|n| n as #element_type)
            },
            "bool" => quote! {
                v.as_bool()
            },
            _ => quote! {
                None
            },
        };

        quote! {
            {
                if let Some(config_value) = #config_var.get(#field_name_str) {
                    if let Value::Array(values_array) = config_value {
                        let values: Vec<#element_type> = values_array.iter()
                            .filter_map(|v| #parse_value)
                            .collect();
                        if values.is_empty() {
                            return Err(format!("Values array for field '{}' is empty or contains invalid types", #field_name_str));
                        }
                        let mut rng = rand::thread_rng();
                        let sample_size = rng.gen_range(1..=values.len());
                        let samples = values.choose_multiple(&mut rng, sample_size)
                            .cloned()
                            .collect::<Vec<#element_type>>();
                        samples
                    } else {
                        return Err(format!("Configuration for '{}' must be an array", #field_name_str));
                    }
                } else {
                    Vec::<#element_type>::new()
                }
            }
        }
    } else {
        // For Vec of complex types
        quote! {
            {
                if let Some(config_value) = #config_var.get(#field_name_str) {
                    if let Value::Array(array) = config_value {
                        let mut vec = Vec::new();
                        for item in array {
                            if let Value::Object(item_config) = item {
                                vec.push(<#element_type>::sample_with_config(&item_config)?);
                            } else {
                                return Err(format!("Each item in '{}' must be an object", #field_name_str));
                            }
                        }
                        vec
                    } else {
                        return Err(format!("Configuration for '{}' must be an array", #field_name_str));
                    }
                } else {
                    Vec::<#element_type>::new()
                }
            }
        }
    }
}

// Helper functions to identify types.

fn is_option(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => type_path.path.segments.last().unwrap().ident == "Option",
        _ => false,
    }
}

fn is_vec(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => type_path.path.segments.last().unwrap().ident == "Vec",
        _ => false,
    }
}

fn get_inner_type(ty: &Type) -> Type {
    match ty {
        Type::Path(type_path) => {
            if let syn::PathArguments::AngleBracketed(args) = &type_path.path.segments.last().unwrap().arguments {
                if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                    inner_type.clone()
                } else {
                    panic!("Expected a type argument");
                }
            } else {
                panic!("Expected angle bracketed arguments");
            }
        }
        _ => panic!("Expected a type path"),
    }
}

fn is_primitive(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => {
            let ident = &type_path.path.segments.last().unwrap().ident;
            ["f64", "f32", "i32", "i64", "u32", "u64", "usize", "isize", "String", "bool"].contains(&ident.to_string().as_str())
        }
        _ => false,
    }
}

fn is_box(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => type_path.path.segments.last().unwrap().ident == "Box",
        _ => false,
    }
}

fn generate_primitive_sample_code(field_type: &Type, field_name_str: &str, config_var: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let type_ident = match field_type {
        Type::Path(type_path) => &type_path.path.segments.last().unwrap().ident,
        _ => panic!("Expected a type path"),
    };
    let type_ident_str = type_ident.to_string();

    if ["f64", "f32"].contains(&type_ident_str.as_str()) {
        // Floating-point numbers
        quote! {
            {
                if let Some(config_value) = #config_var.get(#field_name_str) {
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
                    return Err(format!("Configuration for '{}' is missing", #field_name_str));
                }
            }
        }
    } else if ["i32", "i64", "u32", "u64", "usize", "isize"].contains(&type_ident_str.as_str()) {
        // Integer numbers
        quote! {
            {
                if let Some(config_value) = #config_var.get(#field_name_str) {
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
                    return Err(format!("Configuration for '{}' is missing", #field_name_str));
                }
            }
        }
    } else if type_ident_str == "String" {
        // Strings
        quote! {
            {
                if let Some(config_value) = #config_var.get(#field_name_str) {
                    if let Some(values_array) = config_value.as_array() {
                        let values: Vec<String> = values_array.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                        if !values.is_empty() {
                            values.choose(&mut rand::thread_rng()).unwrap().clone()
                        } else {
                            return Err(format!("Values array for field '{}' is empty", #field_name_str));
                        }
                    } else if let Some(value_str) = config_value.as_str() {
                        value_str.to_string()
                    } else {
                        return Err(format!("Configuration for '{}' must be an array or string", #field_name_str));
                    }
                } else {
                    return Err(format!("Configuration for '{}' is missing", #field_name_str));
                }
            }
        }
    } else if type_ident_str == "bool" {
        // Booleans
        quote! {
            {
                if let Some(config_value) = #config_var.get(#field_name_str) {
                    if let Some(value_bool) = config_value.as_bool() {
                        value_bool
                    } else {
                        return Err(format!("Configuration for '{}' must be a boolean", #field_name_str));
                    }
                } else {
                    return Err(format!("Configuration for '{}' is missing", #field_name_str));
                }
            }
        }
    } else {
        // Unsupported primitive type
        quote! {
            return Err(format!("Unsupported type for field '{}'", #field_name_str));
        }
    }
}