# nu-signature

A Rust crate to generate a nu_protocol::Signature based on nu definition.

## How does it work

Let's build a fake command in a plugin. Here is the base : 

```rs
use nu_plugin::*;

pub struct HelloWorld;

impl PluginCommand for HelloWorld {
    type Plugin = crate::commands::Matrix;

    fn name(&self) -> &str {
        "hello_world"
    }

    fn signature(&self) -> nu_protocol::Signature {
        /* Watch below */
    }

    fn description(&self) -> &str {
        "Simple hello world command, to be used for testing"
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        _call: &EvaluatedCall,
        _input: nu_protocol::PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::LabeledError> {
        todo!()
    }
}
```

So in `signature` function, we would like to write a `nu_protocol::Signature`. 
Instead of writing it by hand, we will use the macro `nu_signature::signature` by writing a `extern` definition :

```rs
use nu_signature::signature;

//...

    fn signature(&self) -> nu_protocol::Signature {
        signature!{r#"
            extern hello_world [
                pos_arg: int    # pos_arg description
                --switch        # switch description
                --flag: string  # flag description
            ]: nothing -> string;
        "#}
    }
```
This sample will create the whole signature using the nu syntax.

Let's see what it generates using [`cargo expand`](https://crates.io/crates/cargo-expand) :
```rs
    fn signature(&self) -> nu_protocol::Signature {
        {
            let mut sig = nu_protocol::Signature::build("hello_world")
                .category(nu_protocol::Category::Experimental)
                .input_output_types(
                    <[_]>::into_vec(
                        #[rustc_box]
                        ::alloc::boxed::Box::new([
                            (nu_protocol::Type::Nothing, nu_protocol::Type::String),
                        ]),
                    ),
                );
            sig.named = <[_]>::into_vec(
                #[rustc_box]
                ::alloc::boxed::Box::new([
                    nu_protocol::Flag {
                        long: String::from("switch"),
                        arg: None,
                        desc: String::from("switch description"),
                        short: None,
                        required: false,
                        default_value: None,
                        var_id: None,
                    },
                    nu_protocol::Flag {
                        long: String::from("flag"),
                        arg: Some(nu_protocol::SyntaxShape::String),
                        desc: String::from("flag description"),
                        short: None,
                        required: false,
                        default_value: None,
                        var_id: None,
                    },
                ]),
            );
            sig.required_positional = <[_]>::into_vec(
                #[rustc_box]
                ::alloc::boxed::Box::new([
                    nu_protocol::PositionalArg {
                        name: String::from("pos_arg"),
                        shape: nu_protocol::SyntaxShape::Int,
                        desc: String::from("pos_arg description"),
                        var_id: None,
                        default_value: None,
                    },
                ]),
            );
            sig
        }
    }
```
As you can see, each single parameter, description, type and input-output have been taken into account. 
You can also specify rest positional arguments, optional positional arguments, several input-output types...
You can use all types of nu (including generic types like `list<string>` or `record<a:int b:string>`), like you 
would do in nu 👍

## How to use the crate ?

Simply add the following line to your Cargo.toml dependencies
```toml
[dependencies]
nu-signature = { git = "https://github.com/glcraft/nu-signature.git" }
```

Note : the project is **not** on crates.io.

## Note about the implementation

You need to use the `extern` keyword to write the signature. This is a workaround considering the nu parser
because the `signature` macro does only need, only a function declaration, so its name, parameters and input-output 
types. By using `def` keyword, you HAVE to define a body, and that body needs to output something of the same type as 
you declared in the function signature.
*Moreover, a plugin command is kinda an external command, isn't ?* 😄

The nu syntax has to be string quoted, either by single quotes `"..."` or by raw string quotes `r#""#`. 
This is because what's inside a Rust macro has to be Rust-like code (At least something similar to Rust syntax). 
The macro's input, the token stream, has some limitations incompatible with Nu's syntax, like new lines are discarded 
and `#` nu's comments doesn't read right.
So to keep the raw form of nu syntax, the signature can be passed by string. The string is recovered as is in the 
token stream.

The project uses the nu-parser crate, but doesn't implement all commands from nu. This mean you cannot use anything 
than `extern` keyword.

There are two crates in the projects. Everything is inside nu-signature-core. The "main" crate is here to make a 
proc-macro Rust library. This is due to test the library with fuzz tests, which use the "core" crate as a runtime 
library, not a proc-macro library