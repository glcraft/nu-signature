mod parse;
mod build_sig;
use proc_macro2::TokenTree;
use quote::quote;

pub fn make_signature(item: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    // let str_item = item.to_string();
    let mut iter = item.into_iter();
    
    let Some(TokenTree::Literal(lit_item)) = iter.next() else {
        return quote! { compile_error!("make_signature expects a literal string containing the signature") };
    };
    if iter.next().is_some() {
        return quote! { compile_error!("make_signature only expects a literal string containing the signature") };
    }
    let str_item = lit_item.to_string();
    let trimmed = trim_string(str_item.as_bytes());
    // panic!("make_signature: {}", std::str::from_utf8(trimmed).unwrap());
    
    let (name, sig) = match parse::extract_declaration(trimmed) {
        Ok(parsed) => parsed,
        Err(e) => return quote! { compile_error!(#e) }
    };

    build_sig::build_signature(&name, sig)
}

fn trim_string(item: &[u8]) -> &[u8] {
    let (begin, end) = if item[0] == b'r' {
        let mut begin = 0;
        while (begin < item.len()) && (item[begin] != b'"') {
            begin += 1;
        }
        (begin+1, item.len() - begin)
    } else {
        (1, item.len() - 1)
    };
    &item[begin..end]
}

