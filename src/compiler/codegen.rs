use crate::compiler::parser::*;





#[derive(Default, Debug, Clone)]
pub struct CodeGenerator {

}
impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            
        }
    }

    pub fn run(&mut self, program: &Program) -> Result<String, String> {
        self.emit_program(program)
    }


    fn emit_program(&mut self, program: &Program) -> Result<String, String> {
        let mut ret = String::new();
        for item in &program.items {
            ret += &self.emit_top_level(item)?;
        }
        Ok(ret)
    }
    fn emit_top_level(&mut self, item: &TopLevel) -> Result<String, String> {
        Ok(
            
            match item {
                _ => "meow".to_string()
            }

        )
    }




    fn error<T>(&mut self, base: &str, err: &str) -> Result<T, String> {
        Err(format!("{base}:{err} \n"))
    }
}


