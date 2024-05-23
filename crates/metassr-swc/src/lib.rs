// TODO:
// - Implement a converter for multiple files in directory
// - save output in out dir
// - redesign the compiler and know how it will works
// - make the compiler more flexible

use anyhow::{anyhow, Result};
use std::path::Path;

use swc_common::{comments::SingleThreadedComments, sync::Lrc, Globals, Mark, SourceMap, GLOBALS};
use swc_ecma_codegen::{text_writer::JsWriter, Emitter};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};
use swc_ecma_transforms_base::{fixer::fixer, hygiene::hygiene, resolver};
use swc_ecma_transforms_typescript::strip;
use swc_ecma_visit::FoldWith;

#[derive(PartialEq, Eq)]
pub enum Lang {
    TypeScript,
    JavaScript,
}

pub struct Compiler {
    filepath: String,
    lang: Lang,
    is_react: bool,
}

impl Compiler {
    pub fn new(filepath: &str) -> Self {
        let path = Path::new(filepath);
        let ext = path.extension().unwrap().to_str().unwrap();
        let is_typescript = ext.eq("tsx") || ext.eq("ts");
        let is_react = ext.eq("tsx") || ext.eq("jsx");

        Self {
            filepath: filepath.to_string(),
            is_react,
            lang: if is_typescript {
                Lang::TypeScript
            } else {
                Lang::JavaScript
            },
        }
    }

    /// Transforms typescript to javascript. Returns tuple (js string, source map)
    pub fn to_js(&self) -> Result<String> {
        if self.lang == Lang::JavaScript {
            return Err(anyhow!("your code is already in javascript"));
        }

        let cm: Lrc<SourceMap> = Default::default();

        let fm = cm
            .load_file(Path::new(&self.filepath))
            .expect("failed to load input typescript file");

        let comments = SingleThreadedComments::default();

        let lexer = Lexer::new(
            Syntax::Typescript(TsConfig {
                tsx: self.is_react,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(&*fm),
            Some(&comments),
        );

        let mut parser = Parser::new_from(lexer);

        for e in parser.take_errors() {
            eprintln!("{e:?}")
        }

        let program = parser
            .parse_program()
            .map_err(|e| eprintln!("{e:?}"))
            .expect("failed to parse program.");

        let globals = Globals::default();
        let code = GLOBALS.set(&globals, || {
            let unresolved_mark = Mark::new();
            let top_level_mark = Mark::new();

            // Optionally transforms decorators here before the resolver pass
            // as it might produce runtime declarations.

            // Conduct identifier scope analysis
            let program = program.fold_with(&mut resolver(unresolved_mark, top_level_mark, true));

            // Remove typescript types
            let program = program.fold_with(&mut strip(top_level_mark));

            // Fix up any identifiers with the same name, but different contexts
            let program = program.fold_with(&mut hygiene());

            // Ensure that we have enough parenthesis.
            let program = program.fold_with(&mut fixer(Some(&comments)));

            let mut buf = vec![];
            {
                let mut emitter = Emitter {
                    cfg: swc_ecma_codegen::Config::default(),
                    cm: cm.clone(),
                    comments: Some(&comments),
                    wr: JsWriter::new(cm.clone(), "\n", &mut buf, None),
                };

                emitter.emit_program(&program).unwrap();
            }

            return String::from_utf8(buf).expect("non-utf8?");
        });
        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiling_from_ts_to_js() {
        println!("{:?}", Compiler::new("../../tests/web-app/App.tsx").to_js());
    }
}
