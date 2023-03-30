use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Field, Fields, ItemStruct, Path};

// refer to https://github.com/dtolnay/syn/blob/master/examples/heapsize/heapsize_derive/src/lib.rs

pub fn syntax_node(attr: TokenStream, input: TokenStream) -> TokenStream {
    let path = syn::parse2::<Path>(attr).unwrap();

    if path.segments.len() != 2 {
        panic!("Must be used with a form of 'Enum::Variant'");
    }

    let enum_name = &path.segments[0].ident;

    let item = syn::parse2::<ItemStruct>(input.clone()).unwrap();
    let name = &item.ident;

    let fields = match item.fields {
        Fields::Named(fields) => fields.named,
        _ => panic!("SyntaxNode can only be derived for structs with named fields"),
    };

    let mut id_field: Option<Field> = None;
    let mut arg_fields = Vec::new();

    for f in fields {
        let ident_name = f.ident.to_token_stream().to_string();
        let ty_name = f.ty.to_token_stream().to_string();
        if ident_name == "id" && ty_name == "usize" {
            id_field = Some(f);
        } else {
            arg_fields.push(f);
        }
    }

    if id_field.is_none() {
        panic!("SyntaxNode must have a field named 'id' of type 'usize'");
    }

    let params = arg_fields.iter().map(|f| {
        let ident = f.ident.clone();
        let ty = f.ty.clone();
        quote! {
            #ident: #ty
        }
    });

    let args = arg_fields.iter().map(|f| {
        let ident = f.ident.clone();
        quote! {
            #ident
        }
    });

    let output = quote! {
        #input

        impl SyntaxNode for #name {
            fn id(&self) -> usize {
                self.id
            }
        }

        impl #name {
            pub fn new_wrapped(#(#params),*) -> #enum_name {
                #path(Ptr::new(Self {
                    id: Self::generate_id(),
                    #(#args),*
                }))
            }
        }
    };
    output.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_syntax_node() {
        let attr = quote! { Enum::Variant };
        let input = quote! {
            #[derive(Debug)]
            struct Foo {
                id: usize,
                bar: String,
                baz: i32,
            }
        };
        let output = syntax_node(attr, input).to_string();

        let expected = quote! {
            #[derive(Debug)]
            struct Foo {
                id: usize,
                bar: String,
                baz: i32,
            }

            impl SyntaxNode for Foo {
                fn id(&self) -> usize {
                    self.id
                }
            }

            impl Foo {
                pub fn new_wrapped(bar: String, baz: i32) -> Enum {
                    Enum::Variant(Ptr::new(Self {
                        id: Self::generate_id(),
                        bar,
                        baz // comma missing. unwanted, but it's ok
                    }))
                }
            }
        }
        .to_string();

        assert_eq!(output, expected);
    }
}
