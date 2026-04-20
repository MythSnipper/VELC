use crate::compiler::parser::*;

use std::collections::HashMap;


#[derive(Debug, Clone)]
pub enum GlobalSymbol {
    Function(FunctionSignature),
    GlobalVar(TypeName),
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub ret: TypeName,
    pub params: Vec<TypeName>,
}

#[derive(Debug, Clone)]
pub struct LocalSymbol {
    pub Type: TypeName,
}





#[derive(Default, Debug, Clone)]
pub struct Analyzer {
    globals: HashMap<String, GlobalSymbol>,
    scopes: Vec<HashMap<String, LocalSymbol>>,
    current_function_ret: Option<TypeName>,
    loop_depth: usize,
}
impl Analyzer {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            scopes: Vec::new(),
            current_function_ret: None,
            loop_depth: 0,
        }
    }

    pub fn run(&mut self, program: &Program) -> Result<(), String> {
        self.analyze_program(program)
    }


    fn register_top_level(&mut self, item: &TopLevel) -> Result<(), String> {
        let ERROR_STRING_ROOT = "velc:Analyzer:register_top_level";
        match item {
            TopLevel::Function(func) => {
                self.register_function(func)?;
            }
            TopLevel::GlobalVar(var) => {
                self.register_global_var(var)?;
            }
            TopLevel::Assembly(_) => {}
        }
        Ok(())
    }













    fn analyze_program(&mut self, program: &Program) -> Result<(), String> {
        let ERROR_STRING_ROOT = "velc:Analyzer:analyze_program";
        for item in &program.items {
            self.register_top_level(item)?;
        }

        for item in &program.items {
            self.analyze_top_level(item)?;
        }

        Ok(())
    }
    fn analyze_top_level(&mut self, item: &TopLevel) -> Result<(), String> {
        let ERROR_STRING_ROOT = "velc:Analyzer:analyze_top_level";
        match item {
            TopLevel::Function(_func) => {
                // function body analysis
                Ok(())
            }
            TopLevel::GlobalVar(var) => {
                if let TypeName::Builtin(BuiltinType::Void) = &var.Type {
                    return self.error(ERROR_STRING_ROOT, "Global variable cannot have type void");
                }

                Ok(())
            }
            TopLevel::Assembly(_) => Ok(())
        }
    }

    




    fn error<T>(&mut self, base: &str, err: &str) -> Result<T, String> {
        Err(format!("{base}:{err} \n"))
    }

}

