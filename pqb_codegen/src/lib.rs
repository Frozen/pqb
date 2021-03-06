#![recursion_limit = "128"]
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

//use syn::Ident;
use syn::Body;
//use syn::VariantData;
//use syn::Ty;

use proc_macro::TokenStream;

#[proc_macro_derive(Model)]
pub fn model(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = impl_model(&ast);

    // Return the generated impl
    gen.parse().unwrap()
}

fn impl_model(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;

    let mut out = vec![];
    match ast.body {
        Body::Struct(ref variant_data) => {
            let x = variant_data.fields();
            for field in x {
                if let Some(ref ident) = field.ident {
                    out.push(ident.clone());
                }
            }
        }
        _ => panic!("{:?} is not struct", name),
    };

    //    : row.get(stringify!(#e))
    let mut out2 = vec![];
    for e in &out {
        out2.push(quote!( #e: row.get(stringify!(#e)) ))
    }

    let mut out3 = vec![];
    for e in &out {
        out3.push(quote!( map.insert(stringify!(#e), &self.#e as &ToSql); ))
    }

    let q = quote! {

        impl #name {
            pub fn select<'a>() -> SelectQuery<'a> {
                SelectQuery::from_model(&#name::default())
            }

            pub fn alias<'a>(alias: &'a str) -> SelectQuery<'a> {
                SelectQuery::from_model_with_alias(&#name::default(), alias)
            }
        }


        impl DbModel for #name {

            fn table(&self) -> String {
                return convert_table_name(stringify!(#name));
            }

            fn fields() -> Vec<&'static str> {
                return vec![#(stringify!(#out)),*]
            }

            fn instance_fields(&self) -> Vec<&'static str> {
                return #name::fields();
            }

            fn as_map(&self) -> std::collections::HashMap<&'static str, &ToSql> {
                let mut map = std::collections::HashMap::new();
                #(#out3)*
                return map;
            }
        }

        impl<'a> ::std::convert::From<Row<'a>> for #name {
            fn from(row: Row) -> Self {
                #name {
                    #(#out2),*
                }
            }
        }


    };

    return q;
}

#[proc_macro_derive(ModelList)]
pub fn model_list(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = impl_model_list(&ast);

    // Return the generated impl
    gen.parse().unwrap()
}

fn impl_model_list(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;

    let q = quote! {

            impl<'a> std::iter::FromIterator<postgres::rows::Row<'a>> for #name {
                fn from_iter<I: IntoIterator<Item=Row<'a>>>(iter: I) -> Self {
                    #name {
                        rows: iter.into_iter().map(From::from).collect()
                    }
                }
            }

            impl std::convert::From<postgres::rows::Rows> for #name {
                fn from(rows: postgres::rows::Rows) -> Self {
                    #name {
                        rows: rows.iter().map(From::from).collect()
                    }
                }
            }

    //        // and we'll implement IntoIterator
    //        impl IntoIterator for #name {
    //            type Item = User;
    //            type IntoIter = ::std::vec::IntoIter<User>;
    //
    //            fn into_iter(self) -> Self::IntoIter {
    //                self.rows.into_iter()
    //            }
    //        }




        };

    //    println!("{}", q);
    return q;
}
