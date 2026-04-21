
use std::{fs, process::exit, path::Path};



mod compiler;
use compiler::*;

#[derive(Default, Debug, Clone)]
struct Compiler {
    input_filename: String,
    output_filename: String,
    temp_filename_root: String,

    time_execution: bool,
    save_intermediate_files: bool,

    do_preprocess: bool,
    do_compile: bool,
    do_assemble: bool,
    do_link: bool,
    do_execute: bool,

    //structures
    lexer: lexer::Lexer,
    parser: parser::Parser,
    analyzer: analyzer::Analyzer,


    source_code: String,
    source_preprocessed: String,
    tokens: Vec<lexer::Token>,
    program: parser::Program,
}
impl Compiler {
    pub fn new() -> Self {
        Self {
            input_filename: String::new(),
            output_filename: String::from("vel.out"),
            temp_filename_root: String::new(),

            time_execution: false,
            save_intermediate_files: false,

            do_preprocess: false,
            do_compile: false,
            do_assemble: false,
            do_link: false,
            do_execute: false,

            lexer: lexer::Lexer::new(),
            parser: parser::Parser::new(),
            analyzer: analyzer::Analyzer::new(),

            source_code: String::new(),
            source_preprocessed: String::new(),
            tokens: Vec::new(),
            program: parser::Program::new(),
        }
    }
    pub fn parse_args(&mut self) -> Result<(), String> {
        let mut args: Vec<String> = std::env::args().collect();
        let help_message: String = format!("Usage: velc [options] file
Options:
  --help        Display this information
  --version     Display compiler version information
  -t,--time     Time the execution of each subprocess
  -save-temps   Save intermediate files
  -E            Preprocess only
  -S            Compile only
  -c            Compile and assemble only
  -o <file>     Place the output into <file>
  --run         Run the output file");
        let version_message: String = format!("velc version 1.0.0 
20260409");

        //remove program name from args list
        args.remove(0);

        let mut input_filename: Option<String> = None;
        let mut output_filename_flag: bool = false;
        let mut preprocess_only_flag: bool = false;
        let mut compile_only_flag: bool = false;
        let mut compile_assemble_only_flag: bool = false;

        for arg in &args {
            if output_filename_flag {
                self.output_filename = arg.to_string();
                output_filename_flag = false;
            }
            else{
                match arg.as_str() {
                    "--help" => {
                        println!("{}", help_message);
                        exit(0);
                    }
                    "--version" => {
                        println!("{}", version_message);
                        exit(0);
                    }
                    "-t" | "--time" => {
                        self.time_execution = true;
                    }
                    "-save-temps" => {
                        self.save_intermediate_files = true;
                    }
                    "-E" => {
                        preprocess_only_flag = true;
                    }
                    "-S" => {
                        compile_only_flag = true;
                    }
                    "-c" => {
                        compile_assemble_only_flag = true;
                    }
                    "-o" => {
                        output_filename_flag = true;
                    }
                    "--run" => {
                        self.do_execute = true;
                    }
                    _ => {
                        input_filename = Some(arg.to_string());
                    }
                }
            }
        }
        if preprocess_only_flag {
            self.do_preprocess = true;
            self.do_compile = false;
            self.do_assemble =  false;
            self.do_link = false;
            self.do_execute = false;
        }
        else if compile_only_flag {
            self.do_preprocess = true;
            self.do_compile = true;
            self.do_assemble =  false;
            self.do_link = false;
            self.do_execute = false;
        }
        else if compile_assemble_only_flag {
            self.do_preprocess = true;
            self.do_compile = true;
            self.do_assemble =  true;
            self.do_link = false;
            self.do_execute = false;
        }
        else {
            self.do_preprocess = true;
            self.do_compile = true;
            self.do_assemble =  true;
            self.do_link = true;
        }

        if let Some(filename) = input_filename {
            //validate input file is there
            let path = Path::new(&filename);
            self.input_filename = filename.clone();
            //get filename root
            self.temp_filename_root = match path.file_stem() {
                Some(stem) => stem.to_string_lossy().into_owned(),
                None => filename.to_string()
            };
        }
        else {
            return Err(String::from("No input filename"));
        }

        Ok(())
    }
    pub fn run(&mut self) -> Result<(), String> {

        //read source code from input file
        self.source_code = fs::read_to_string(&self.input_filename)
        .map_err(|err| format!("failed to read {}: {}", self.input_filename, err))? ;
        
        //preprocess
        if self.do_preprocess {
            self.source_preprocessed = preprocessor::run(&self.source_code)?;
        }

        //compile
        if self.do_compile {
            //lexer
            self.lexer = lexer::Lexer::from_source(&self.source_preprocessed);
            self.tokens = self.lexer.run()?;

            //parser
            self.parser = parser::Parser::from_tokens(&self.tokens);
            self.program = self.parser.run()?;
            println!("{:#?}", self.program);

            //semantic analyzer
            self.analyzer = analyzer::Analyzer::new();
            self.analyzer.run(&self.program)?;
            


            //code generator





        }



        Ok(())
    }
}

fn main() {
    let mut compiler: Compiler = Compiler::new();


    if let Err(e) = compiler.parse_args() {
        println!("Error: {e}");
    }
    else if let Err(e) = compiler.run() {
        println!("{e}");
    }

}


























