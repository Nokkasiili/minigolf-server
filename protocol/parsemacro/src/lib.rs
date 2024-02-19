use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;

use syn::{parse_macro_input, DeriveInput, Type};

//#[derive(Debug, FromMeta)]
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(parse))]
struct ParseAttribute {
    #[darling(default)]
    tag: String,
    #[darling(default)]
    space: bool,
    #[darling(default)]
    notab: bool,
    #[darling(default)]
    notag: bool,

    #[darling(default)]
    opt_f: bool,
}

#[proc_macro_derive(Parse, attributes(parse))]
pub fn parse_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ParseAttribute {
        tag,
        space,
        notab,
        notag,
        opt_f,
    } = FromDeriveInput::from_derive_input(&input).expect("can't parse attribute");

    let item_name = &input.ident;

    let expanded = match &input.data {
        syn::Data::Struct(data) => {
            let fields = match &data.fields {
                syn::Fields::Named(fields) => &fields.named,
                _ => panic!("Parse can only be used with named fields in structs"),
            };

            let delimiter = {
                if space {
                    ' '
                } else {
                    '\t'
                }
            };

            let packet_number = fields.iter().any(|f| is_packet_number_type(&f.ty));

            let field_parsing = fields
                .iter()
                .filter(|f| !is_packet_number_type(&f.ty))
                .enumerate()
                .map(|(i, field)| {
                    let field_name = &field.ident;
                    let field_type = &field.ty;

                    if i == 0 && notab {
                        quote! {
                            let (input, #field_name) = <#field_type>::parse(input)?;
                        }
                    } else if opt_f {
                        quote! {
                            let (input, _) = opt(char(#delimiter))(input)?;
                            let (input, #field_name) = <#field_type>::parse(input)?;
                        }
                    } else {
                        quote! {
                            let (input, _) = char(#delimiter)(input)?;
                            let (input, #field_name) = <#field_type>::parse(input)?;
                        }
                    }
                });

            let as_string_impl = fields
                .iter()
                .filter(|f| !is_packet_number_type(&f.ty))
                .enumerate()
                .map(|(i, field)| {
                    let field_name = &field.ident;
                    if (i == 0 && notab) || opt_f {
                        quote! {
                            result.push_str(&self.#field_name.as_string());
                        }
                    } else {
                        quote! {
                            result.push(#delimiter);
                            result.push_str(&self.#field_name.as_string());

                        }
                    }
                });

            let field_names = fields.iter().map(|field| {
                let field_name = &field.ident;
                quote! {
                    #field_name,
                }
            });

            let packet_string = {
                if packet_number {
                    quote! {
                        result.push_str("d ");
                        result.push_str(&self.packet_number.as_string());
                        result.push_str(" ");
                    }
                } else {
                    quote! {}
                }
            };

            let input_string = {
                if notag {
                    quote! {}
                } else {
                    quote! {
                        let (input, _) = tag(#tag)(input)?;
                    }
                }
            };

            let packet_number_impl = {
                if packet_number {
                    quote! {
                    Some(self.packet_number)
                    }
                } else {
                    quote! {
                        None
                    }
                }
            };

            quote! {
                impl Packet for #item_name{
                                fn packet_number(&self) -> Option<PacketNumber>{
                                    #packet_number_impl
                                }
                }
                            impl Parse for #item_name {
                                fn parse(input: &str) -> IResult<&str,Self> {
                                    use nom::bytes::complete::tag;
                                    use nom::combinator::opt;
                                    use nom::character::complete::char;


                                    let (input, packet_number) = if #packet_number {
                                        let (input, _) = tag("d ")(input)?;
                                        let (input, packet_number) = <PacketNumber>::parse(input)?;
                                        let (input, _) = char(' ')(input)?;
                                        (input, packet_number)
                                    } else {
                                        (input, PacketNumber(0))
                                    };
                                    #input_string
                                    #(
                                        #field_parsing
                                    )*

                                    let (input, _) = opt(char('\n'))(input)?;

                                    Ok((input, #item_name { #(#field_names)* }))
                                }

                                fn as_string(&self) -> String {
                                    let mut result = String::new();
                                    #packet_string
                                    result.push_str(#tag);
                                    #(
                                        #as_string_impl
                                    )*
            /**
                                    if #useless_tab{
                                        result.push('\t');
                                    }*/
                                    if #notag == false{
                                        result.push('\n');
                                    }
                                    result
                                }

                            }
                        }
        }

        syn::Data::Enum(data) => {
            let variants = data.variants.iter().map(|variant| {
                let variant_name = &variant.ident;

                quote! {
                    map(<#variant_name>::parse, #item_name::#variant_name)
                }
            });

            let variants_string_match = data.variants.iter().map(|variant| {
                let variant_name = &variant.ident;

                quote! {
                    #item_name::#variant_name(i) => i.as_string(),
                }
            });
            let variants_packet_number_match = data.variants.iter().map(|variant| {
                let variant_name = &variant.ident;

                quote! {
                    #item_name::#variant_name(i) => i.packet_number(),
                }
            });
            let chunks: Vec<_> = variants.clone().collect();

            let alt_variants = chunks.chunks(21).map(|chunk| {
                let alt_variants = chunk.to_owned();
                quote! {
                    alt((#(#alt_variants),*))
                }
            });

            quote! {
            impl Parse for #item_name {
                fn parse(input: &str) -> IResult<&str,Self> {
                    use nom::branch::alt;
                    use nom::combinator::map;

                    let (input,command) = alt((#(#alt_variants),*))(input)?;
                    Ok((input,command))
                    }
                fn as_string(&self) -> String{
                    match &self{
                        #(#variants_string_match)*
                        _ => unreachable!()
                    }
                }
            }
            impl Packet for #item_name {

                fn packet_number(&self) -> Option<PacketNumber>{
                    match &self{
                        #(#variants_packet_number_match)*
                        _ => unreachable!()
                    }
                }
            }
            }
        }

        _ => panic!("Parse can only be used with enums or structs"),
    };

    fn is_packet_number_type(ty: &Type) -> bool {
        if let Type::Path(type_path) = ty {
            if let Some(ident) = type_path.path.get_ident() {
                return ident == "PacketNumber";
            }
        }
        false
    }

    expanded.into()
}
