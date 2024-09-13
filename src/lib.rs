#[proc_macro]
pub fn signature(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    nu_signature_core::make_signature(item.into()).into()
}