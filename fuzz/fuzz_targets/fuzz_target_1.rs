#![no_main]

use std::{fmt::Display, ops::RangeBounds};

use libfuzzer_sys::fuzz_target;

#[derive(arbitrary::Arbitrary, Debug)]
struct Signature {
    name: String,
    signature: Parameters,
}

impl Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "extern {} {}", self.name, self.signature)
    }
}

#[derive(arbitrary::Arbitrary, Debug)]
struct Parameters {
    params: Vec<Parameter>,
    rest: Option<Parameter>,
}


impl Display for Parameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for param in self.params.iter() {
            write!(f, "\n{}", param)?;
        }
        if let Some(param) = &self.rest {
            write!(f, "...{}", param)?;
        }
        write!(f, "\n];")
    }
}

#[derive(arbitrary::Arbitrary, Debug)]
struct Parameter {
    name: String,
    desc: Option<String>,
    ty: ParameterType,
}


impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.ty {
            ParameterType::Switch => write!(f, "--{}", self.name),
            ParameterType::ShortSwitch => write!(f, "-{}", self.name.chars().next().unwrap_or('a')),
            ParameterType::Flag(dt) => write!(f, "--{}{}", self.name, dt),
            ParameterType::ShortFlag(dt) => write!(f, "-{}{}", self.name.chars().next().unwrap_or('a'), dt),
            ParameterType::PositionalArg(dt) => write!(f, "{}{}", self.name, dt),
        }?;
        if let Some(desc) = &self.desc {
            write!(f, "# {}", desc)?;
        }
        Ok(())
    }
}

#[derive(arbitrary::Arbitrary, Debug)]
enum ParameterType {
    Switch,
    ShortSwitch,
    Flag(ParameterData),
    ShortFlag(ParameterData),
    PositionalArg(ParameterData),
}

#[derive(arbitrary::Arbitrary, Debug)]
struct ParameterData {
    type_: DataType,
    optional: bool,
    default_value: Option<String>,
}

impl Display for ParameterData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", if self.optional { "?" } else { "" }, self.type_)?;
        if let Some(default_value) = &self.default_value {
            write!(f, " = {}", default_value)?;
        }
        Ok(())
    }
}

#[derive(arbitrary::Arbitrary, Debug)]
enum DataType {
    Integer,
    Float,
    String,
    Boolean,
    List(Box<DataType>),
    Record(Vec<(String, DataType)>),
    Table(Vec<(String, DataType)>),
    Nothing,
}
impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Integer => write!(f, "integer"),
            DataType::Float => write!(f, "float"),
            DataType::String => write!(f, "string"),
            DataType::Boolean => write!(f, "boolean"),
            DataType::List(dt) => write!(f, "list<{}>", dt),
            DataType::Record(fields) | DataType::Table(fields) => {
                match self {
                    DataType::Record(_) => write!(f, "record")?,
                    DataType::Table(_) => write!(f, "table")?,
                    _ => unreachable!(),
                }
                write!(f, "<")?;
                for (i, (name, dt)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", name, dt)?;
                }
                write!(f, ">")
            }
            DataType::Nothing => write!(f, "nothing"),
        }
    }
}

fuzz_target!(|data: Signature| {
    let res = nu_signature_core::make_signature(proc_macro2::TokenTree::Literal(proc_macro2::Literal::string(&data.to_string())).into());
    let res = res.to_string();
    assert!(res.contains("compile_error"));
});
