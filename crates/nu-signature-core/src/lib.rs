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
    let trimmed = match literal_to_string(&str_item) {
        Ok(s) => s,
        Err(e) => return quote! { compile_error!(#e) }
    };
    // let test = trimmed.chars().map(|c| format!("'{}'", c)).collect::<Vec<_>>().join(", ");
    // return quote! { compile_error!(#test) };
    
    let (name, sig) = match parse::extract_declaration(trimmed.as_bytes()) {
        Ok(parsed) => parsed,
        Err(e) => return quote! { compile_error!(#e) }
    };

    build_sig::build_signature(&name, sig)
}

fn literal_to_string(item: &str) -> Result<String, String> {
    if item.chars().next() == Some('r') {
        let item_bytes = item.as_bytes();
        let mut begin = 0;
        while (begin < item_bytes.len()) && (item_bytes[begin] != b'"') {
            begin += 1;
        }
        let (begin, end) = (begin+1, item_bytes.len() - begin);
        Ok(unsafe { String::from_utf8_unchecked(Vec::from(&item_bytes[begin..end])) })
    } else {
        let mut res = String::with_capacity(item.len());
        let mut escape = false;
        for c in item.chars().skip(1).take(item.len() - 2) {
            match (c, escape) {
                ('\\', false) => {
                    escape = true;
                    continue;
                },
                (c, false) => res.push(c),
                ('\\', true) => res.push('\\'),
                ('"', true) => res.push('"'),
                ('n', true) => res.push('\n'),
                ('r', true) => res.push('\r'),
                ('t', true) => res.push('\t'),
                (_, true) => return Err(format!(r#"Invalid escape sequence: \{}"#, c)), 
            }
            escape = false;
        }
        Ok(res)
    }
}