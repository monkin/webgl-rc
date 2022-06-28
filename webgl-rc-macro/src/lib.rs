#![allow(dead_code)]

extern crate proc_macro;
extern crate regex;

use proc_macro::{TokenTree, TokenStream, LexError};
use std::fs::File;
use std::path::Path;
use std::io::Read;
use regex::Regex;
use std::env::VarError;

/*
TokenStream [
    Ident { ident: "struct", span: #0 bytes(517..523) },
    Ident { ident: "TriangleUniforms", span: #0 bytes(524..540) },
    Group {
        delimiter: Brace,
        stream: TokenStream [
            Ident { ident: "color", span: #0 bytes(547..552) },
            Punct { ch: ':', spacing: Alone, span: #0 bytes(552..553) },
            Group {
                delimiter: Parenthesis,
                stream: TokenStream [
                    Ident { ident: "f32", span: #0 bytes(555..558) },
                    Punct { ch: ',', spacing: Alone, span: #0 bytes(558..559) },
                    Ident { ident: "f32", span: #0 bytes(560..563) },
                    Punct { ch: ',', spacing: Alone, span: #0 bytes(563..564) },
                    Ident { ident: "f32", span: #0 bytes(565..568) }
                ],
                span: #0 bytes(554..569)
            },
            Punct { ch: ',', spacing: Alone, span: #0 bytes(569..570) }
        ],
        span: #0 bytes(541..572)
    }
]
*/

#[derive(Debug)]
enum Error {
    Unknown(String),
    StructNameNotFound,
    Syntax(LexError),
    InvalidArguments(String),
    RootDirNotFound(VarError),
    IOError {
        file: String,
        error: std::io::Error,
    },
    Multiple(Vec<Error>),
}

impl ToString for Error {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

impl From<LexError> for Error {
    fn from(error: LexError) -> Self {
        Error::Syntax(error)
    }
}

impl From<VarError> for Error {
    fn from(error: VarError) -> Self {
        Error::RootDirNotFound(error)
    }
}

impl From<Error> for TokenStream {
    fn from(error: Error) -> Self {
        format!("compile_error!(r####\"{:?}\"####);", error).parse().unwrap()
    }
}

#[derive(Clone, Debug)]
struct Field {
    name: String,
    type_name: String,
}

#[derive(Clone, Debug)]
struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

fn parse_field<I>(iter: &mut I) -> Option<Field>
where
    I : Iterator<Item = TokenTree> + Clone,
{
    let field_name = iter.take_while(|v| match v {
        TokenTree::Punct(punct) => punct.to_string() != ":",
        _ => true,
    }).last().map(|v| match v {
        TokenTree::Ident(ident) => Some(ident.to_string()),
        _ => None,
    })??;

    let type_name = iter.take_while(|v| match v {
        TokenTree::Punct(punct) => punct.to_string() != ",",
        _ => true,
    }).map(|v| v.to_string()).collect::<Vec<_>>().join("");

    if type_name != "" {
        Some(Field {
            name: field_name,
            type_name: type_name,
        })
    } else {
        None
    }
}

fn parse_struct<I>(iter: &mut I) -> Result<Struct, Error>
where
    I : Iterator<Item = TokenTree> + Clone,
{
    let name = match iter.skip_while(|item| {
        match item {
            TokenTree::Ident(ident) => ident.to_string() != "struct",
            _ => true
        }
    }).skip(1).next().ok_or(Error::StructNameNotFound)? {
        TokenTree::Ident(ident) => Ok(ident.to_string()),
        _ => Err(Error::StructNameNotFound)
    }?;

    let fields = iter.last().map(|group| {
        match group {
            TokenTree::Group(group) => Some(group.stream().into_iter()),
            _ => None,
        }
    }).flatten().map(|mut iter| {
        let mut fields: Vec<Field> = Default::default();

        while let Some(field) = parse_field(&mut iter) {
            fields.push(field)
        }

        fields
    }).unwrap_or_else(|| Default::default());

    Ok(Struct { name, fields })
}

fn uniforms_impl(tokens: TokenStream) -> Result<TokenStream, Error> {
    let parsed = parse_struct(&mut tokens.into_iter())?;
    let source = format!(
        r####"impl webgl_rc::uniforms::Uniforms for {struct_name} {{
            fn uniforms(&self) -> Vec<webgl_rc::uniforms::Field> {{
                use webgl_rc::uniforms::IntoUniform;
                vec![
                    {content}
                ]
            }}
        }}"####,
        struct_name = parsed.name,
        content = &parsed.fields.iter().map(|field| {
            format!(
                r###"webgl_rc::uniforms::Field {{ name: r#"u_{name}"#, value: self.{name}.into_uniform() }},"###,
                name = field.name,
            )
        }).collect::<Vec<_>>().join("")
    );
    Ok(source.parse()?)
}

#[proc_macro_derive(Uniforms)]
pub fn uniforms(tokens: TokenStream) -> TokenStream {
    uniforms_impl(tokens).unwrap_or_else(|error| error.into())
}

fn attributes_impl(prefix: &str, tokens: TokenStream) -> Result<TokenStream, Error> {
    let parsed = parse_struct(&mut tokens.into_iter())?;
    let source = format!(
        r####"
            impl webgl_rc::data_buffer::Item for {struct_name} {{
                fn layout() -> Vec<webgl_rc::data_buffer::Layout> {{
                    use webgl_rc::types::TypeMark;
                    vec![
                        {layout_items}
                    ]
                }}
            }}
            impl webgl_rc::data_buffer::Writable for {struct_name} {{
                fn write(&self, output: &mut Vec<f32>) {{
                    use webgl_rc::data_buffer::Writable;
                    {write_items}
                }}
                fn stride() -> usize {{
                    use webgl_rc::data_buffer::Writable;
                    {stride_items}
                }}
            }}
        "####,
        struct_name = parsed.name,
        layout_items = &parsed.fields.iter().map(|field| {
            format!(
                r###"webgl_rc::data_buffer::Layout {{ name: r#"{prefix}_{name}"#, data_type: <{type_name} as TypeMark>::data_type() }},"###,
                prefix = prefix,
                name = field.name,
                type_name = field.type_name,
            )
        }).collect::<Vec<_>>().join(""),
        write_items = &parsed.fields.iter().map(|field| {
            format!(
                r###"self.{name}.write(output);"###,
                name = field.name,
            )
        }).collect::<Vec<_>>().join(""),
        stride_items = &parsed.fields.iter().map(|field| {
            format!(
                r###"<{type_name} as Writable>::stride()"###,
                type_name = field.type_name,
            )
        }).collect::<Vec<_>>().join(" + "),
    );
    source.parse().map_err(|error: LexError| error.into())
}

#[proc_macro_derive(Attributes)]
pub fn attributes(tokens: TokenStream) -> TokenStream {
    attributes_impl("a", tokens).unwrap_or_else(|error| error.into())
}

#[proc_macro_derive(Instances)]
pub fn instances(tokens: TokenStream) -> TokenStream {
    attributes_impl("i", tokens).unwrap_or_else(|error| error.into())
}

struct Content {
    content: String,
    dependencies: Vec<String>,
}

fn load_glsl_file(root: &Path, file: &Path) -> Result<Content, Error> {
    let mut handle = File::open(file).map_err(|error| {
        Error::IOError {
            file: file.to_str().unwrap().into(),
            error,
        }
    })?;
    let mut source: String = Default::default();
    handle.read_to_string(&mut source).map_err(|error| {
        Error::IOError {
            file: file.to_str().unwrap().into(),
            error,
        }
    })?;

    let mut dependencies = vec![file.to_str().unwrap().into()];
    let mut errors = Vec::new();

    let source_with_includes = Regex::new(r#"#include\s*(<.+?>|".+?")"#)
        .unwrap()
        .replace_all(&source, &mut |captures: &regex::Captures<'_>| {
            let capture = captures.get(1).unwrap().as_str();
            let file_name = if capture.starts_with("<") {
                root.join(capture.get(1..(capture.len() - 1)).unwrap())
            } else {
                file.parent().unwrap().join(capture.get(1..(capture.len() - 1)).unwrap())
            };

            match load_glsl_file(root, &file_name) {
                Ok(content) => {
                    for file in content.dependencies {
                        dependencies.push(file);
                    }
                    content.content
                }
                Err(error) => {
                    errors.push(error);
                    format!("#error Failed to include file {:?}\n", file_name)
                }
            }
        });

    if errors.is_empty() {
        Ok(Content {
            content: source_with_includes.into(),
            dependencies,
        })
    } else {
        Err(Error::Multiple(errors))
    }
}

fn load_glsl_impl(stream: TokenStream) -> Result<TokenStream, Error> {
    let tokens = stream.into_iter().collect::<Vec<_>>();
    return if tokens.is_empty() {
        Err(Error::InvalidArguments("File name not provided".into()))
    } else if tokens.len() > 1 {
        Err(Error::InvalidArguments("Too many arguments".into()))
    } else {
        let name = match tokens.first().unwrap() {
            TokenTree::Literal(value) => {
                Ok(
                    value.to_string().chars().into_iter()
                        .skip_while(|c| *c != '"')
                        .skip(1)
                        .take_while(|c| *c != '"')
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join("")
                )
            },
            other => Err(Error::Unknown(format!("File name should be a string but {:?} provided", other)))
        }?;

        let root = Path::new(
            &std::env::var("CARGO_MANIFEST_DIR")?
        ).join("glsl");

        let content = load_glsl_file(root.as_path(), root.join(name).as_path())?;

        Ok(format!(
            r#####"{{ {dependencies}; r####"{content}"#### }}"#####,
            dependencies = content.dependencies.into_iter().map(|file| {
                format!(r##"const _: &[u8] = include_bytes!(r#"{file}"#);"##, file = file)
            }).collect::<Vec<_>>().join(""),
            content = content.content
        ).parse()?)
    }
}

#[proc_macro]
pub fn load_glsl(tokens: TokenStream) -> TokenStream {
    load_glsl_impl(tokens).unwrap_or_else(|error| error.into())
}