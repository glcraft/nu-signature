use core::str;

use ast::Expr;
use engine::CommandType;
use nu_protocol::*;
use nu_protocol::{
    engine::{
        Command, EngineState, Stack, Call
    }, 
    Signature, SyntaxShape
};

#[derive(Clone)]
pub struct Extern;

impl Command for Extern {
    fn name(&self) -> &str {
        "extern"
    }

    fn description(&self) -> &str {
        "Declare an external function."
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build("extern")
            .input_output_types(vec![(Type::Nothing, Type::Nothing)])
            .required("def_name", SyntaxShape::String, "Definition name.")
            .required("params", SyntaxShape::Signature, "Parameters.")
            .category(Category::Core)
    }
    
    fn command_type(&self) -> CommandType {
        CommandType::Keyword
    }

    fn run(
        &self,
        _engine_state: &EngineState,
        _stack: &mut Stack,
        _call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        Ok(PipelineData::empty())
    }
}
pub fn extract_declaration(content: &[u8]) -> Result<(String, Signature), String> { 
    let engine = nu_protocol::engine::EngineState::new();
    let mut working_set = nu_protocol::engine::StateWorkingSet::new(&engine);
    working_set.add_decl(Box::new(Extern));
    let ext_call = nu_parser::parse(&mut working_set, None, content, false);
    if !working_set.parse_errors.is_empty() {
        return Err(format!("Error while parsing the expression: {:#?}\nContent: {}", working_set.parse_errors, str::from_utf8(content).unwrap()));
    }
    
    if ext_call.pipelines.len() != 1 || ext_call.pipelines[0].elements.len() != 1 {
        return Err("only extern expression expected".to_string());
    }
    let Expr::Call(call) = &ext_call.pipelines[0].elements[0].expr.expr else {
        return Err(format!("extern expression expected, expr: {:?}", ext_call.pipelines));
    };
    let Expr::String(ref name) = call.positional_nth(0).ok_or_else(|| "missing function name".to_string())?.expr else {
        return Err("missing function name".to_string());
    };
    let decl = working_set.get_decl(working_set.find_decl(name.as_bytes()).ok_or_else(|| "failed to find func as decl".to_string())?);
    
    Ok((name.clone(), decl.signature()))
}