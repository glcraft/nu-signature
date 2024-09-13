use nu_protocol::{ast::Expr, Signature};

mod keyword {
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
            "Mock def command."
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
}

struct SpannedError {
    span: (u32, u32),
    msg: String,
}

fn extract_declaration(content: &str) -> Result<(String, Signature), String> { 
    let engine = nu_protocol::engine::EngineState::new();
    let mut working_set = nu_protocol::engine::StateWorkingSet::new(&engine);
    working_set.add_decl(Box::new(keyword::Extern));
    let ext_call = nu_parser::parse(&mut working_set, None, content.as_bytes(), false);
    if !working_set.parse_errors.is_empty() {
        println!("Parse errors: {:#?}", working_set.parse_errors);
        for err in &working_set.parse_errors {
            match err {
                nu_protocol::ParseError::UnknownType(span) => println!("Unknown type: '{}'", std::str::from_utf8(working_set.get_span_contents(*span)).unwrap()),
                e => println!("{:#?}", e),
            }
        }
    }
    
    if ext_call.pipelines.len() != 1 || ext_call.pipelines[0].elements.len() != 1 {
        return Err("only def expression expected".to_string());
    }
    let Expr::Call(call) = &ext_call.pipelines[0].elements[0].expr.expr else {
        return Err("def expression expected".to_string());
    };
    let Expr::String(ref name) = call.positional_nth(0).ok_or_else(|| "missing function name".to_string())?.expr else {
        return Err("missing function name".to_string());
    };
    let decl = working_set.get_decl(working_set.find_decl(name.as_bytes()).expect("failed to find func as decl"));
    // println!("description: {:#?}", decl.description());
    // println!("extra_description: {:#?}", decl.extra_description());
    // println!("name: {:#?}", decl.name());
    // println!("command type: {:#?}", decl.command_type());

    
    Ok((name.clone(), decl.signature()))
}


fn main() {
    let test = r#"
    # description hello world
    extern hello_world [--test1 test2:int]: nothing -> string;"#;
    // let (tokens, errors) = nu_parser::lex(test.as_bytes(), 0, &[], &[], false);
    // println!("tokens: {tokens:?}, errors: {errors:?}");
    let (name, signature) = extract_declaration(test).unwrap();
    // assert_eq!(, Expr::Nothing);
    print!("func name: {}\nsignature: {:#?}", name, signature);
}