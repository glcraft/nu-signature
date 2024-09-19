#![no_main]

use std::{fmt::Display, ops::{Deref, RangeBounds}};

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: Signature| {
    let data_token = proc_macro2::TokenTree::Literal(proc_macro2::Literal::string(&data.to_string()));
    let res = nu_signature_core::make_signature(data_token.clone().into());
    let res = res.to_string();
    assert!(!res.contains("compile_error"), "Failed to make signature:\nnu signature:\n{}\nTokenTree:\n{:?}\nRust signature:\n{}", data, data_token, res);
});

#[derive(arbitrary::Arbitrary, Debug)]
struct Signature {
    name: Named,
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
    name: Named,
    desc: Option<Named>,
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
    default_value: Option<Named>,
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

fn char_from_ranges(index: u32, ranges: &[impl RangeBounds<char>]) -> char {
    use std::ops::Bound;
    assert!(!ranges.is_empty() && !ranges.iter().any(|r| matches!((r.start_bound(), r.end_bound()), (Bound::Unbounded, _) | (_, Bound::Unbounded))), "Invalid range");
    let ranges_numb = ranges
        .iter()
        .map(|r| {
            let start = match r.start_bound() {
                Bound::Included(c) => *c as u32,
                Bound::Excluded(c) => *c as u32 + 1,
                Bound::Unbounded => unreachable!("Invalid range"),
            };
            let end = match r.end_bound() {
                Bound::Included(c) => *c as u32 + 1,
                Bound::Excluded(c) => *c as u32,
                Bound::Unbounded => unreachable!("Invalid range"),
            };
            (start, end)
        });
    assert!(!ranges_numb.clone().any(|(start, end)| start > end), "Invalid range");
    let total = ranges_numb.clone().fold(0, |acc, (start, end)| {
        acc + (end - start)
    });
    let mut index = index % total;
    for (start, end) in ranges_numb {
        if index < (end - start) {
            return unsafe { char::from_u32_unchecked(start + index) };
        }
        index -= end - start;
    }
    'a'
}

#[derive(Debug)]
struct Named(String);

impl arbitrary::Arbitrary<'_> for Named {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        if u.is_empty() {
            return Err(arbitrary::Error::NotEnoughData);
        }
        let name = u.arbitrary_iter::<u32>()?
            .take(20)
            .enumerate()
            .map(|(i, c)| {
                if i == 0 {
                    Ok(char_from_ranges(c?, &['a'..='z', 'A'..='Z']))
                } else {
                    Ok(char_from_ranges(c?, &['a'..='z', 'A'..='Z', '0'..='9']))
                }
            }
        );
        let name = name.collect::<Result<String, _>>()?;
        if name.len() < 2 {
            return Err(arbitrary::Error::NotEnoughData);
        }
        Ok(Named(name))
    }
}
impl Deref for Named {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Display for Named {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}