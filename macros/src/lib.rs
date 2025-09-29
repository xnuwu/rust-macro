use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Data, Expr, Lit, ExprTuple, DeriveInput, Fields, ItemFn, LitStr, Type, Attribute, Meta, GenericArgument, PathArguments};

#[proc_macro_derive(Getter)]
pub fn derive_getter(input: TokenStream) -> TokenStream {
   let input = parse_macro_input!(input as DeriveInput);
   let name = &input.ident;

   let fields = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields_named) = &data_struct.fields {
            &fields_named.named
        } else {
            return syn::Error::new_spanned(&input.ident, "Only named fields are supported")
            .to_compile_error()
            .into();
        }
   } else {
     return syn::Error::new_spanned(&input.ident, "Only structs are supported")
            .to_compile_error()
            .into();
   };

   let getters = fields.iter().map(|field| {
    let field_name = &field.ident;
    let field_ty = &field.ty;

    quote! {
        pub fn #field_name(&self) -> &#field_ty {
            &self.#field_name
        }
    }
   });

   let expanded = quote! {
    impl #name {
        #(#getters)*
    }
   };

   TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn timed(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(input as ItemFn);
    let func_name = &func.sig.ident;
    let original_code = &func.block;

    func.block = parse_quote! {
        {
            let start = std::time::Instant::now();
            let ret = (|| #original_code)();
            println!("{} cost {}ms", stringify!(#func_name), start.elapsed().as_millis());
            return ret;
        }
    };

    TokenStream::from(quote! { #func })
}

#[proc_macro]
pub fn reverse_string(input: TokenStream) -> TokenStream {
    let s = parse_macro_input!(input as LitStr);
    let reversed: String = s.value().chars().rev().collect();

    let exploded = quote! {
        const r: &str = #reversed;
    };

    TokenStream::from(exploded)
}

#[proc_macro_derive(Entity, attributes(orm))]
pub fn derivd_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let table_name = name.to_string().trim().to_lowercase();

    let fields = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields_named) = &data_struct.fields {
            &fields_named.named
        } else {
            panic!("Only named fields supported");
        }
    } else {
        panic!("Only structs supported");
    };

    // Process fields to extract metadata
    let mut field_info = Vec::new();
    let mut primary_keys = Vec::new();
    
    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        let sql_type = rust_type_to_sql(field_type);
        
        let is_primary_key = has_orm_attribute(&field.attrs, "primary_key");
        let is_nullable = is_option_type(field_type);
        
        if is_primary_key {
            primary_keys.push(field_name.to_string());
        }
        
        field_info.push((field_name.to_string(), sql_type, is_primary_key, is_nullable));
    }

    let field_names: Vec<String> = field_info.iter().map(|(name, _, _, _)| name.clone()).collect();
    let field_names_str = field_names.join(",");
    let field_values_str = field_names.iter().map(|name| format!("self.{}", name)).collect::<Vec<_>>().join(",");

    // Generate CREATE TABLE statement
    let create_table_fields: Vec<String> = field_info.iter().map(|(name, sql_type, is_pk, is_nullable)| {
        let mut field_def = format!("{} {}", name, sql_type);
        if *is_pk {
            field_def.push_str(" PRIMARY KEY");
        }
        if !is_nullable && !is_pk {
            field_def.push_str(" NOT NULL");
        }
        field_def
    }).collect();
    
    let create_table_sql = format!(
        "CREATE TABLE {} ({})",
        table_name,
        create_table_fields.join(", ")
    );

    let expanded = quote! {
        impl #name {
            pub fn table_name() -> &'static str {
                #table_name
            }
            
            pub fn create_table() -> &'static str {
                #create_table_sql
            }

            pub fn insert(&self) -> String {
                format!("INSERT INTO {} ({}) VALUES ({})", Self::table_name(), #field_names_str, #field_values_str)
            }
            
            pub fn primary_keys() -> Vec<&'static str> {
                vec![#(#primary_keys),*]
            }
        }
    };

    TokenStream::from(expanded)
}

// Helper function to map Rust types to SQL types
fn rust_type_to_sql(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            let type_name = &type_path.path.segments.last().unwrap().ident;
            match type_name.to_string().as_str() {
                "String" => "TEXT".to_string(),
                "i8" | "i16" | "i32" => "INTEGER".to_string(),
                "i64" => "BIGINT".to_string(),
                "u8" | "u16" | "u32" => "INTEGER".to_string(),
                "u64" => "BIGINT".to_string(),
                "f32" => "REAL".to_string(),
                "f64" => "DOUBLE".to_string(),
                "bool" => "BOOLEAN".to_string(),
                "Option" => {
                    // Handle Option<T> types
                    if let PathArguments::AngleBracketed(args) = &type_path.path.segments.last().unwrap().arguments {
                        if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                            return rust_type_to_sql(inner_type);
                        }
                    }
                    "TEXT".to_string()
                }
                _ => "TEXT".to_string(), // Default to TEXT for unknown types
            }
        }
        _ => "TEXT".to_string(),
    }
}

// Helper function to check if a type is Option<T>
fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

// Helper function to check for ORM attributes
fn has_orm_attribute(attrs: &[Attribute], attr_name: &str) -> bool {
    for attr in attrs {
        if attr.path().is_ident("orm") {
            if let Meta::List(meta_list) = &attr.meta {
                let tokens = meta_list.tokens.to_string();
                return tokens.contains(attr_name);
            }
        }
    }
    false
}

#[proc_macro]
pub fn concat_strings(input: TokenStream) -> TokenStream {
    let expr_tuple = parse_macro_input!(input as ExprTuple);
    let mut concated_string = String::new();
    for expr in expr_tuple.elems.iter() {
        if let Expr::Lit(lit) = expr {
            if let Lit::Str(lit_str) = &lit.lit {
                concated_string.push_str(&lit_str.value());
            } else {
                panic!("Only string literals are supported");
            }
        } else {
            panic!("Only string literals are supported");
        }
    }
    let exploded = quote! {
        const CONCATENATED: &str = #concated_string;
    };

    TokenStream::from(exploded)
}