#![no_main]

use core::f64;
use std::{fmt::Display, ops::{Deref, RangeBounds}};

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: Signature| {
    SHORTS.lock().map(|mut s| s.clear()).expect("failed to clear shorts");
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
    pub required_positional: Vec<PositionalArg>,
    pub optional_positional: Vec<OptionalPositionalArg>,
    pub rest_positional: Option<RestParameter>,
    pub named: Vec<Flag>,
    pub input_output_types: Vec<(ReturnType, ReturnType)>,
}

impl Display for Parameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for param in &self.required_positional {
            write!(f, "\n{}", param)?;
        }
        for param in &self.optional_positional {
            write!(f, "\n{}", param)?;
        }
        if let Some(r) = &self.rest_positional {
            write!(f, "\n{}", r)?;
        }
        for param in &self.named {
            write!(f, "\n{}", param)?;
        }
        write!(f, "\n]")?;
        match self.input_output_types.len() {
            0 => write!(f, ";"),
            1 => write!(f, ": {} -> {}", self.input_output_types[0].0, self.input_output_types[0].1),
            _ => {
                write!(f, ": [")?;
                for param in &self.input_output_types {
                    write!(f, " {} -> {}", param.0, param.1)?;
                }
                write!(f, " ];")
            }
        }
    }
}

#[derive(arbitrary::Arbitrary, Debug)]
struct Flag {
    name: Named,
    short: bool,
    desc: Option<Named>,
    ty: Option<ParameterType>,
}
static SHORTS: once_cell::sync::Lazy<std::sync::Mutex<std::collections::HashSet<char>>> = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(std::collections::HashSet::<char>::new()));

fn get_short(name: &str) -> char {
    let mut shorts = SHORTS.lock().expect("failed to lock shorts");
    for c in name.chars() {
        if !shorts.contains(&c) {
            shorts.insert(c);
            return c;
        }
    }
    let chars = ('a'..='z').chain('A'..='Z').chain('0'..='9');
    for c in chars {
        if !shorts.contains(&c) {
            shorts.insert(c);
            return c;
        }
    }
    panic!("Failed to generate short for {}", name)
}

impl Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "--{}", self.name)?;
        if self.short {
            write!(f, "(-{})", get_short(&self.name))?;
        }
        if let Some(ty) = &self.ty {
            write!(f, ": {}", ty)?;
        }
        if let Some(desc) = &self.desc {
            write!(f, " # {}", desc)?;
        }
        Ok(())
    }
}

#[derive(arbitrary::Arbitrary, Debug)]
struct PositionalArg {
    name: Named,
    desc: Option<Named>,
    ty: Option<ParameterType>,
}

impl Display for PositionalArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(ty) = &self.ty {
            write!(f, ": {}", ty)?;
        }
        if let Some(desc) = &self.desc {
            write!(f, " # {}", desc)?;
        }
        Ok(())
    }
}

#[derive(arbitrary::Arbitrary, Debug)]
struct OptionalPositionalArg {
    name: Named,
    desc: Option<Named>,
    ty: OptionalValue,
}

impl Display for OptionalPositionalArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}?: {}", self.name, self.ty)?;
        if let Some(desc) = &self.desc {
            write!(f, " # {}", desc)?;
        }
        Ok(())
    }
}


#[derive(arbitrary::Arbitrary, Debug)]
enum OptionalValue {
    HasValue(Value),
    NoValue(ParameterType),
}
impl Display for OptionalValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionalValue::HasValue(v) => write!(f, "{} = {}", v.ty(), v),
            OptionalValue::NoValue(ty) => write!(f, "{}", ty),
        }
    }
}
#[derive(arbitrary::Arbitrary, Debug)]
struct RestParameter{
    name: Named,
    desc: Option<Named>,
    ty: ParameterType,
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
enum ParameterType {
    Integer,
    Float,
    String,
    // Boolean,
    List(Option<Box<ParameterType>>),
    Record(Vec<(Named, ParameterType)>),
    Table(Vec<(Named, ParameterType)>),
    // Nothing,
}
impl Display for ParameterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterType::Integer => write!(f, "int"),
            ParameterType::Float => write!(f, "float"),
            ParameterType::String => write!(f, "string"),
            // DataType::Boolean => write!(f, "bool"),
            ParameterType::List(dt) => {
                match dt {
                    Some(dt) => write!(f, "list<{}>", dt),
                    None => write!(f, "list"),
                }
            }
            ParameterType::Record(fields) | ParameterType::Table(fields) => {
                match self {
                    ParameterType::Record(_) => write!(f, "record")?,
                    ParameterType::Table(_) => write!(f, "table")?,
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

#[derive(arbitrary::Arbitrary, Clone, Debug)]
enum ReturnType {
    Integer,
    Float,
    String,
    Boolean,
    List(Option<Box<ReturnType>>),
    Record(Vec<(Named, ReturnType)>),
    Table(Vec<(Named, ReturnType)>),
    Nothing,
}
impl Display for ReturnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer => write!(f, "int"),
            Self::Float => write!(f, "float"),
            Self::String => write!(f, "string"),
            Self::Boolean => write!(f, "bool"),
            Self::List(dt) => {
                match dt {
                    Some(dt) => write!(f, "list<{}>", dt),
                    None => write!(f, "list"),
                }
            }
            Self::Record(fields) | Self::Table(fields) => {
                match self {
                    Self::Record(_) => write!(f, "record")?,
                    Self::Table(_) => write!(f, "table")?,
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
            Self::Nothing => write!(f, "nothing"),
        }
    }
}

#[derive(arbitrary::Arbitrary, Debug)]
enum Value {
    Integer(i64),
    Float(Float),
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
            Value::Float(i) => write!(f, "{:.8}", i.deref()),
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
    fn ty(&self) -> ParameterType {
        match self {
            Value::Integer(_) => ParameterType::Integer,
            Value::Float(_) => ParameterType::Float,
            Value::String(_) => ParameterType::String,
            // Value::Boolean(_) => DataType::Boolean,
            Value::List(v) => ParameterType::List(if v.is_empty() { None } else { Some(Box::new(v[0].ty())) }),
            Value::Record(fields) => ParameterType::Record(fields.iter().map(|(name, value)| (name.clone(), value.ty())).collect()),
            Value::Table(fields) => ParameterType::Table(fields.iter().map(|(name, value)| (name.clone(), value.ty())).collect()),
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


#[derive(Clone, Debug)]
struct Float(f64);

impl Deref for Float {
    type Target = f64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl arbitrary::Arbitrary<'_> for Float {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        union IntOrFloat{
            i: u64,
            f: f64,
        }
        let mut input = IntOrFloat { i: u.arbitrary::<u64>()? };
        const INFINITY: IntOrFloat = IntOrFloat { f: f64::INFINITY };
        
        unsafe {
            if (input.i & INFINITY.i) == INFINITY.i {
                input.i ^= INFINITY.i;
            }
            Ok(Float(input.f))
        }
    }
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        f64::size_hint(depth)
    }
}