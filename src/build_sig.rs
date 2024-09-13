use chrono::{Datelike, Timelike};
use nu_protocol::{Flag, PositionalArg, Signature};
use proc_macro2::Literal;
use quote::quote;
fn shape_to_token(shape: &nu_protocol::SyntaxShape) -> proc_macro2::TokenStream {
    use nu_protocol::SyntaxShape;
    match shape {
        SyntaxShape::Any => quote! { nu_protocol::SyntaxShape::Any },
        SyntaxShape::Binary => quote! { nu_protocol::SyntaxShape::Binary },
        SyntaxShape::Block => quote! { nu_protocol::SyntaxShape::Block },
        SyntaxShape::Boolean => quote! { nu_protocol::SyntaxShape::Boolean },
        SyntaxShape::CellPath => quote! { nu_protocol::SyntaxShape::CellPath },
        SyntaxShape::Closure(opt_args) => {
            let args_tokens = opt_args.as_ref().map(|args| {
                    let args_tokens = args.iter().map(shape_to_token);
                    quote! { Some(vec![#(#args_tokens),*]) }
                })
                .unwrap_or_else(|| quote! { None });
            quote! { nu_protocol::SyntaxShape::Closure(#args_tokens) }
        }
        SyntaxShape::CompleterWrapper(s, _) => {
            let s = shape_to_token(s);
            quote! { nu_protocol::SyntaxShape::CompleterWrapper(Box::new(#s), Default::default()) }
        }
        SyntaxShape::DateTime => quote! { nu_protocol::SyntaxShape::DateTime },
        SyntaxShape::Directory => quote! { nu_protocol::SyntaxShape::Directory },
        SyntaxShape::Duration => quote! { nu_protocol::SyntaxShape::Duration },
        SyntaxShape::Error => quote! { nu_protocol::SyntaxShape::Error },
        SyntaxShape::Expression => quote! { nu_protocol::SyntaxShape::Expression },
        SyntaxShape::ExternalArgument => quote! { nu_protocol::SyntaxShape::ExternalArgument },
        SyntaxShape::Filepath => quote! { nu_protocol::SyntaxShape::Filepath },
        SyntaxShape::Filesize => quote! { nu_protocol::SyntaxShape::Filesize },
        SyntaxShape::Float => quote! { nu_protocol::SyntaxShape::Float },
        SyntaxShape::FullCellPath => quote! { nu_protocol::SyntaxShape::FullCellPath },
        SyntaxShape::GlobPattern => quote! { nu_protocol::SyntaxShape::GlobPattern },
        SyntaxShape::Int => quote! { nu_protocol::SyntaxShape::Int },
        SyntaxShape::ImportPattern => quote! { nu_protocol::SyntaxShape::ImportPattern },
        SyntaxShape::Keyword(kw, s) => {
            let kw = Literal::byte_string(kw);
            let s = shape_to_token(s);
            quote! { nu_protocol::SyntaxShape::Keyword(#kw, Box::new(#s)) }
        }
        SyntaxShape::List(s) => {
            let s = shape_to_token(s);
            quote! { nu_protocol::SyntaxShape::List(Box::new(#s)) }
        }
        SyntaxShape::MathExpression => quote! { nu_protocol::SyntaxShape::MathExpression },
        SyntaxShape::MatchBlock => quote! { nu_protocol::SyntaxShape::MatchBlock },
        SyntaxShape::Nothing => quote! { nu_protocol::SyntaxShape::Nothing },
        SyntaxShape::Number => quote! { nu_protocol::SyntaxShape::Number },
        SyntaxShape::OneOf(s) => {
            let s = s.iter().map(shape_to_token).collect::<Vec<_>>();
            quote! { nu_protocol::SyntaxShape::OneOf(vec![#(#s),*]) }
        }
        SyntaxShape::Operator => quote! { nu_protocol::SyntaxShape::Operator },
        SyntaxShape::Range => quote! { nu_protocol::SyntaxShape::Range },
        SyntaxShape::Record(entries) => { 
            let entries_tokens = entries
                .iter()
                .map(|(name, shape)| {
                    let name = Literal::string(name);
                    let shape = shape_to_token(shape);
                    quote! { (String::from(#name), #shape) }
                });
            quote! { nu_protocol::SyntaxShape::Record(vec![#(#entries_tokens),*]) }
        }
        SyntaxShape::RowCondition => quote! { nu_protocol::SyntaxShape::RowCondition },
        SyntaxShape::Signature => quote! { nu_protocol::SyntaxShape::Signature },
        SyntaxShape::String => quote! { nu_protocol::SyntaxShape::String },
        SyntaxShape::Table(columns) => {
            let columns_tokens = columns
                .iter()
                .map(|(name, shape)| {
                    let name = Literal::string(name);
                    let shape = shape_to_token(shape);
                    quote! { (String::from(#name), #shape) }
                });
            quote! { nu_protocol::SyntaxShape::Table(vec![#(#columns_tokens),*]) }
        }
        SyntaxShape::VarWithOptType => quote! { nu_protocol::SyntaxShape::VarWithOptType },
    }
}
fn type_to_token(type_: &nu_protocol::Type) -> proc_macro2::TokenStream {
    match type_ {
        nu_protocol::Type::Any => quote! { nu_protocol::Type::Any },
        nu_protocol::Type::Binary => quote! { nu_protocol::Type::Binary },
        nu_protocol::Type::Block => quote! { nu_protocol::Type::Block },
        nu_protocol::Type::Bool => quote! { nu_protocol::Type::Bool },
        nu_protocol::Type::CellPath => quote! { nu_protocol::Type::CellPath },
        nu_protocol::Type::Closure => quote! { nu_protocol::Type::Closure },
        nu_protocol::Type::Custom(custom) => quote! { nu_protocol::Type::Custom(#custom) },
        nu_protocol::Type::Date => quote! { nu_protocol::Type::Date },
        nu_protocol::Type::Duration => quote! { nu_protocol::Type::Duration },
        nu_protocol::Type::Error => quote! { nu_protocol::Type::Error },
        nu_protocol::Type::Filesize => quote! { nu_protocol::Type::Filesize },
        nu_protocol::Type::Float => quote! { nu_protocol::Type::Float },
        nu_protocol::Type::Int => quote! { nu_protocol::Type::Int },
        nu_protocol::Type::List(ty) => {
            let list_type = type_to_token(ty);
            quote! { nu_protocol::Type::List(Box::new(#list_type)) }
        },
        nu_protocol::Type::ListStream => quote! { nu_protocol::Type::ListStream },
        nu_protocol::Type::Nothing => quote! { nu_protocol::Type::Nothing },
        nu_protocol::Type::Number => quote! { nu_protocol::Type::Number },
        nu_protocol::Type::Range => quote! { nu_protocol::Type::Range },
        nu_protocol::Type::Record(fields) 
        | nu_protocol::Type::Table(fields)=> {
            let fields_types = fields
                .iter()
                .map(|(name, ty)| {
                    let name = Literal::string(name);
                    let ty = type_to_token(ty);
                    quote! { (#name, #ty) }
                });
            let ty = match type_ {
                nu_protocol::Type::Record(_) => quote! { Record },
                nu_protocol::Type::Table(_) => quote! { Table },
                _ => unreachable!()
            };
            quote! { nu_protocol::Type::#ty(Box::new([#(#fields_types),*])) }
        },
        nu_protocol::Type::Signature => quote! { nu_protocol::Type::Signature },
        nu_protocol::Type::String => quote! { nu_protocol::Type::String },
        nu_protocol::Type::Glob => quote! { nu_protocol::Type::Glob },
    }
}

pub fn value_to_token(value: &nu_protocol::Value) -> proc_macro2::TokenStream {
    use nu_protocol::{Value, Range};
    use std::ops::Bound;
    
    match value {
        Value::Bool { val, .. } => quote! { nu_protocol::Value::bool(#val, Span::unknown()) },
        Value::Int { val, .. } => quote! { nu_protocol::Value::int(#val, Span::unknown()) },
        Value::Float { val, .. } => quote! { nu_protocol::Value::float(#val, Span::unknown()) },
        Value::Filesize { val, .. } => quote! { nu_protocol::Value::filesize(#val, Span::unknown()) },
        Value::Duration { val, .. } => quote! { nu_protocol::Value::duration(#val, Span::unknown()) },
        Value::Date { val, .. } => {
            let year = val.year();
            let month = val.month();
            let day = val.day();
            let hour = val.hour();
            let minute = val.minute();
            let second = val.second();
            let nanosecond = val.nanosecond();
            let offset = val.offset().local_minus_utc();
            quote! { 
                unsafe {
                    chrono::NaiveDate::from_ymd_opt(#year, #month, #day).unwrap_unchecked()
                    .and_hms_nano_opt(#hour, #minute, #second, #nanosecond).unwrap_unchecked()
                    .and_local_timezone(FixedOffset::east_opt(#offset).unwrap_unchecked())
                    .unwrap()
                }
             }
        },
        Value::Range { val, .. } => {
            match **val {
                Range::IntRange(int_range) => {
                    let start = {
                        let s = int_range.start();
                        quote!{ nu_protocol::Value::int(#s) }
                    };
                    let step = {
                        let s = int_range.step();
                        quote!{ nu_protocol::Value::int(#s) }
                    };
                    let (end, range_inclusion) = match int_range.end() {
                        Bound::Excluded(e) => (quote!{ nu_protocol::Value::int(#e) }, quote!{ RangeInclusion::RightExclusive }),
                        Bound::Included(e) => (quote!{ nu_protocol::Value::int(#e) }, quote!{ RangeInclusion::Inclusive }),
                        Bound::Unbounded => (quote!{ nu_protocol::Value::nothing() }, quote!{ RangeInclusion::Inclusive }),
                    };
                    quote! { nu_protocol::Value::Range { val: Box::new(nu_protocol::Range::IntRange(IntRange::new(#start, #step, #end, #range_inclusion, Span::unknown()))), span: Span::unknown() } }
                }
                Range::FloatRange(float_range) => {
                    let start = {
                        let s = float_range.start();
                        quote!{ nu_protocol::Value::float(#s) }
                    };
                    let step = {
                        let s = float_range.step();
                        quote!{ nu_protocol::Value::float(#s) }
                    };
                    let (end, range_inclusion) = match float_range.end() {
                        Bound::Excluded(e) => (quote!{ nu_protocol::Value::float(#e) }, quote!{ RangeInclusion::RightExclusive }),
                        Bound::Included(e) => (quote!{ nu_protocol::Value::float(#e) }, quote!{ RangeInclusion::Inclusive }),
                        Bound::Unbounded => (quote!{ nu_protocol::Value::nothing() }, quote!{ RangeInclusion::Inclusive }),
                    };
                    quote! { nu_protocol::Value::Range { val: Box::new(nu_protocol::Range::FloatRange(FloatRange::new(#start, #step, #end, #range_inclusion, Span::unknown()))), span: Span::unknown() } }
                }
            }
        },
        Value::String { val, .. } => quote! { nu_protocol::Value::String { val: String::from(#val), span: Span::unknown() } },
        Value::Glob { val, .. } => quote! { nu_protocol::Value::Glob { val: String::from(#val), span: Span::unknown() } },
        Value::Record { val, .. } => {
            let keys = val.columns().map(|k| quote! { String::from(#k) });
            let values = val.values().map(value_to_token);
            quote! { nu_protocol::Value::record(Record::from_raw_cols_vals(vec![#(#keys),*], vec![#(#values),*], Span::unknown(), Span::unknown())) }
        }
        Value::List { vals, .. } => {
            let values = vals.iter().map(value_to_token);
            quote! { nu_protocol::Value::list(vec![#(#values),*], Span::unknown()) }
        }
        Value::Closure { .. } => quote!{ compile_error!("Closure not supported") },
        Value::Nothing { .. } => quote! { nu_protocol::Value::Nothing { span: Span::unknown() } },
        Value::Error { .. } => quote!{ compile_error!("Error not supported") },
        Value::Binary { val, .. } => quote! { nu_protocol::Value::binary(vec![#(#val),*], Span::unknown()) },
        Value::CellPath { val, .. } => {
            let members = val.members
                .iter()
                .map(|x| match x {
                    nu_protocol::ast::PathMember::String { val, optional, .. } => quote! { nu_protocol::ast::PathMember::string(#val, #optional, Span::unknown()) },
                    nu_protocol::ast::PathMember::Int { val, optional, .. } => quote! { nu_protocol::ast::PathMember::int(#val, #optional, Span::unknown()) },
                });
            quote! { nu_protocol::Value::CellPath { val: CellPath::new(vec![#(#members),*]), span: Span::unknown() } }
        },
        Value::Custom { .. } => quote!{ compile_error!("Error not supported") },
        
    }
}

pub fn option_to_token<T, F>(
    opt: &Option<T>,
    f: F,
) -> proc_macro2::TokenStream
where
    F: FnOnce(&T) -> proc_macro2::TokenStream,
{
    opt.as_ref()
        .map_or_else(
            || quote! { None }, 
            |x| { 
                let res = f(x); 
                quote! { Some(#res) } 
            }
        )
}

fn flag_to_token(flag: &Flag) -> proc_macro2::TokenStream {
    let long = &flag.long;
    let arg = option_to_token(&flag.arg, shape_to_token);
    let desc = &flag.desc;
    let short = option_to_token(&flag.short, |short| quote! { #short });
    let req = flag.required;
    let def_value = option_to_token(&flag.default_value, value_to_token);
    quote! { nu_protocol::Flag {
        long: String::from(#long),
        arg: #arg,
        desc: String::from(#desc),
        short: #short,
        required: #req,
        default_value: #def_value,
        var_id: None,
    }}
}
fn positional_arg_to_token(arg: &PositionalArg) -> proc_macro2::TokenStream {
    let name = &arg.name;
    let shape = shape_to_token(&arg.shape);
    let desc = &arg.desc;
    let default_value = option_to_token(&arg.default_value, value_to_token);
    quote! { nu_protocol::PositionalArg {
        name: String::from(#name),
        shape: #shape,
        desc: String::from(#desc),
        var_id: None,
        default_value: #default_value,
    } }
}

pub fn build_signature(name: &str, sig: Signature) -> proc_macro2::TokenStream {
    let description = (!sig.description.is_empty())
        .then(|| {
            let desc = sig.description; 
            quote! { .description(#desc) }
        });
    let extra_description = (!sig.extra_description.is_empty())
        .then(|| {
            let desc = sig.extra_description; 
            quote! { .extra_description(#desc) }
        });
    let input_output_types = (!sig.input_output_types.is_empty())
        .then(|| {
            let iter = sig.input_output_types.iter()
                .map(|(input, output)| {
                    let input = type_to_token(input);
                    let output = type_to_token(output);
                    quote! { (#input, #output) }
                });
            quote! { 
                .input_output_types(vec![#(#iter),*])
            }
        });
        // sig.named(name, shape, desc, short)
    
    let named = (!sig.named.is_empty()).then(||{
        let all_named_tokens = sig.named.iter()
            .map(flag_to_token);
        quote! { sig.named = vec![ #(#all_named_tokens),* ]; }
    });
    let required_positional = (!sig.required_positional.is_empty()).then(|| {
        let req_pos_tokens = sig.required_positional.iter()
            .map(positional_arg_to_token);
        quote! { sig.required_positional = vec![ #(#req_pos_tokens),* ]; }
    });
    let optional_positional = (!sig.optional_positional.is_empty()).then(|| {
        let opt_pos_tokens = sig.optional_positional
            .iter()
            .map(positional_arg_to_token);
        quote! { sig.optional_positional = vec![ #(#opt_pos_tokens),* ]; }
    });
    let default_rest_positional = nu_protocol::PositionalArg {
        name: String::from("args"),
        shape: nu_protocol::SyntaxShape::ExternalArgument,
        desc: String::from("all other arguments to the command"),
        var_id: None,
        default_value: None,
    };
    let rest_positional = (sig.rest_positional.is_some())
        .then_some(sig.rest_positional)
        .flatten()
        .filter(|rest_positional| rest_positional != &default_rest_positional)
        .map(|rest_positional| {
            let rest_positional = positional_arg_to_token(&rest_positional);
            quote! { sig.rest_positional = #rest_positional; }
        });
    quote! {
        {
            let mut sig = nu_protocol::Signature::build(#name)
                .category(nu_protocol::Category::Experimental)
                #description
                #extra_description
                #input_output_types;
            #named
            #required_positional
            #optional_positional
            #rest_positional
            sig
        }
    }
}
