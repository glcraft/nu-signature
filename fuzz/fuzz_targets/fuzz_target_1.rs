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
    rest: Option<RestParameter>,
}

impl Display for Parameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for param in self.params.iter() {
            write!(f, "\n{}", param)?;
        }
        if let Some(r) = &self.rest {
            write!(f, "\n{}", r)?;
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
            write!(f, " # {}", desc)?;
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
enum ParameterData {
    Simple(DataType),
    Optional(DataType),
    WithDefaultValue(Value),
    OptionalWithDefaultValue(Value),
}

impl Display for ParameterData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterData::Simple(dt) => write!(f, ":{}", dt),
            ParameterData::Optional(dt) => write!(f, "?:{}", dt),
            ParameterData::WithDefaultValue(v) => write!(f, ":{} = {}", v.ty(), v),
            ParameterData::OptionalWithDefaultValue(v) => write!(f, "?:{} = {}", v.ty(), v),
        }
    }
}
#[derive(arbitrary::Arbitrary, Debug)]
struct RestParameter{
    name: Named,
    desc: Option<Named>,
    ty: DataType,
}

impl Display for RestParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "...{}: {}", self.name, self.ty)?;
        if let Some(desc) = &self.desc {
            write!(f, " # {}", desc)?;
        }
        Ok(())
    }
}

#[derive(arbitrary::Arbitrary, Clone, Debug)]
enum DataType {
    Integer,
    Float,
    String,
    // Boolean,
    List(Option<Box<DataType>>),
    Record(Vec<(Named, DataType)>),
    Table(Vec<(Named, DataType)>),
    // Nothing,
}
impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Integer => write!(f, "int"),
            DataType::Float => write!(f, "float"),
            DataType::String => write!(f, "string"),
            // DataType::Boolean => write!(f, "bool"),
            DataType::List(dt) => {
                match dt {
                    Some(dt) => write!(f, "list<{}>", dt),
                    None => write!(f, "list"),
                }
            }
            DataType::Record(fields) | DataType::Table(fields) => {
                match self {
                    DataType::Record(_) => write!(f, "record")?,
                    DataType::Table(_) => write!(f, "table")?,
                    _ => unreachable!(),
                }
                if fields.is_empty() {
                    return Ok(());
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
            // DataType::Nothing => write!(f, "nothing"),
        }
    }
}

#[derive(arbitrary::Arbitrary, Debug)]
enum Value {
    Integer(i64),
    Float(f64),
    String(Named),
    // Boolean(bool),
    List(Vec<Value>),
    Record(Vec<(Named, Value)>),
    Table(Vec<(Named, Value)>),
    // Nothing,
}

fn float_to_string(i: f64, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = i.to_string();
    if s.contains('.') {
        write!(f, "{}", s)
    } else {
        write!(f, "{}.0", s)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(i) => write!(f, "{:.8}", i),
            Value::String(s) => write!(f, "\"{}\"", s),
            // Value::Boolean(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Value::List(l) => {
                write!(f, "[{}]", l.iter().map(Self::to_string).collect::<Vec<_>>().join(", "))?;
                Ok(())
            }
            Value::Record(r) => {
                write!(f, "{{{}}}", r.iter().map(|(name, value)| format!("{}: {}", name, value)).collect::<Vec<_>>().join(", "))?;
                Ok(())
            }
            Value::Table(t) => {
                write!(f, "[{}]", t.iter().map(|(name, value)| format!("{{{}: {}}}", name, value)).collect::<Vec<_>>().join(", "))?;
                Ok(())
            }
            // Value::Nothing => write!(f, "null"),
        }
    }
}
impl Value {
    fn ty(&self) -> DataType {
        match self {
            Value::Integer(_) => DataType::Integer,
            Value::Float(_) => DataType::Float,
            Value::String(_) => DataType::String,
            // Value::Boolean(_) => DataType::Boolean,
            Value::List(v) => DataType::List(if v.is_empty() { None } else { Some(Box::new(v[0].ty())) }),
            Value::Record(fields) => DataType::Record(fields.iter().map(|(name, value)| (name.clone(), value.ty())).collect()),
            Value::Table(fields) => DataType::Table(fields.iter().map(|(name, value)| (name.clone(), value.ty())).collect()),
            // Value::Nothing => DataType::Nothing,
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

#[derive(Clone, Debug)]
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
        if name.is_empty() {
            return Err(arbitrary::Error::NotEnoughData);
        }
        Ok(Named(name))
    }
    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (1, Some(20))
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