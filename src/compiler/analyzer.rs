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


    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }
    fn declare_local(&mut self, name: &str, Type: TypeName) -> Result<(), String> {
        let ERROR_STRING_ROOT = "velc:Analyzer:declare_local";

        let scope = match self.scopes.last_mut() {
            Some(scope) => scope,
            None => {
                return self.error(ERROR_STRING_ROOT, "No active local scope");
            }
        };

        if scope.contains_key(name) {
            return self.error(
                ERROR_STRING_ROOT,
                &format!("Local symbol '{}' already declared in this scope", name),
            );
        }

        scope.insert(
            name.to_string(),
            LocalSymbol {
                Type,
            },
        );

        Ok(())
    }

    fn lookup_local(&self, name: &str) -> Option<&LocalSymbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }

        None
    }
    fn lookup_global(&self, name: &str) -> Option<&GlobalSymbol> {
        self.globals.get(name)
    }
    fn lookup_variable_type(&self, name: &str) -> Option<TypeName> {
        if let Some(local) = self.lookup_local(name) {
            return Some(local.Type.clone());
        }

        match self.lookup_global(name) {
            Some(GlobalSymbol::GlobalVar(Type)) => Some(Type.clone()),
            _ => None,
        }
    }
    fn lookup_function_signature(&self, name: &str) -> Option<&FunctionSignature> {
        match self.lookup_global(name) {
            Some(GlobalSymbol::Function(sig)) => Some(sig),
            _ => None,
        }
    }

    fn is_void_type(&self, Type: &TypeName) -> bool {
        matches!(Type, TypeName::Builtin(BuiltinType::Void))
    }
    fn is_bool_type(&self, Type: &TypeName) -> bool {
        matches!(Type, TypeName::Builtin(BuiltinType::Bool))
    }
    fn is_char_type(&self, Type: &TypeName) -> bool {
        matches!(Type, TypeName::Builtin(BuiltinType::Char))
    }
    fn is_integer_type(&self, Type: &TypeName) -> bool {
        matches!(
            Type,
            TypeName::Builtin(BuiltinType::Int8)
                | TypeName::Builtin(BuiltinType::Int16)
                | TypeName::Builtin(BuiltinType::Int32)
                | TypeName::Builtin(BuiltinType::Int64)
                | TypeName::Builtin(BuiltinType::Uint8)
                | TypeName::Builtin(BuiltinType::Uint16)
                | TypeName::Builtin(BuiltinType::Uint32)
                | TypeName::Builtin(BuiltinType::Uint64)
                | TypeName::Builtin(BuiltinType::Char)
        )
    }
    fn is_float_type(&self, Type: &TypeName) -> bool {
        matches!(
            Type,
            TypeName::Builtin(BuiltinType::Float32)
                | TypeName::Builtin(BuiltinType::Float64)
        )
    }
    fn is_numeric_type(&self, Type: &TypeName) -> bool {
        self.is_integer_type(Type) || self.is_float_type(Type)
    }
    fn is_pointer_type(&self, Type: &TypeName) -> bool {
        matches!(Type, TypeName::Pointer(_))
    }
    fn get_pointee_type(&self, Type: &TypeName) -> Option<TypeName> {
        match Type {
            TypeName::Pointer(inner) => Some((**inner).clone()),
            _ => None,
        }
    }
    
    fn is_assignable_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Identifier(_) => true,
            Expr::Index { .. } => true,
            Expr::Member { .. } => true,
            Expr::Prefix { op: PrefixOp::Deref, .. } => true,
            _ => false,
        }
    }
    fn is_addressable_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Identifier(_) => true,
            Expr::Index { .. } => true,
            Expr::Member { .. } => true,
            Expr::Prefix { op: PrefixOp::Deref, .. } => true,
            _ => false,
        }
    }
    
    fn can_assign(&self, target: &TypeName, value: &TypeName) -> bool {
        if self.is_void_type(target) || self.is_void_type(value) {
            return false;
        }

        if target == value {
            return true;
        }

        if self.is_numeric_type(target) && self.is_numeric_type(value) {
            return true;
        }

        if self.is_pointer_type(target) && self.is_pointer_type(value) {
            return target == value;
        }

        false
    }

    fn enter_loop(&mut self) {
        self.loop_depth += 1;
    }
    fn exit_loop(&mut self) {
        if self.loop_depth > 0 {
            self.loop_depth -= 1;
        }
    }
    fn is_inside_loop(&self) -> bool {
        self.loop_depth > 0
    }
    
    fn enter_function(&mut self, ret: TypeName) {
        self.current_function_ret = Some(ret);
    }
    fn exit_function(&mut self) {
        self.current_function_ret = None;
    }
    fn current_return_type(&self) -> Option<&TypeName> {
        self.current_function_ret.as_ref()
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
    fn register_function(&mut self, func: &FunctionDecl) -> Result<(), String> {
        let ERROR_STRING_ROOT = "velc:Analyzer:register_function";

        if self.lookup_global(&func.name).is_some() {
            return self.error(
                ERROR_STRING_ROOT,
                &format!("Top level symbol '{}' is already declared", func.name),
            );
        }

        /*
        void returns allowed for main
        
        if self.is_void_type(&func.ret) && func.name != "main" {
            
        }
        */

        let mut params: Vec<TypeName> = Vec::new();

        for param in &func.params {
            if self.is_void_type(&param.Type) {
                return self.error(
                    ERROR_STRING_ROOT,
                    &format!("Parameter '{}' in function '{}' cannot have void type", param.name, func.name),
                );
            }

            params.push(param.Type.clone());
        }

        self.globals.insert(
            func.name.clone(),
            GlobalSymbol::Function(FunctionSignature {
                ret: func.ret.clone(),
                params,
            }),
        );

        Ok(())
    }
    fn register_global_var(&mut self, var: &VarDecl) -> Result<(), String> {
        let ERROR_STRING_ROOT = "velc:Analyzer:register_global_var";

        if self.lookup_global(&var.name).is_some() {
            return self.error(
                ERROR_STRING_ROOT,
                &format!("Top level symbol '{}' is already declared", var.name),
            );
        }

        if self.is_void_type(&var.Type) {
            return self.error(
                ERROR_STRING_ROOT,
                &format!("Global variable '{}' cannot have void type", var.name),
            );
        }

        self.globals.insert(
            var.name.clone(),
            GlobalSymbol::GlobalVar(var.Type.clone()),
        );

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
            TopLevel::Function(func) => self.analyze_function(func),
            TopLevel::GlobalVar(var) => self.analyze_global_var(var),
            TopLevel::Assembly(_) => Ok(())
        }
    }
    fn analyze_function(&mut self, func: &FunctionDecl) -> Result<(), String> {
        let ERROR_STRING_ROOT = "velc:Analyzer:analyze_function";

        self.enter_function(func.ret.clone());
        self.push_scope();

        for param in &func.params {
            if self.is_void_type(&param.Type) {
                self.pop_scope();
                self.exit_function();

                return self.error(
                    ERROR_STRING_ROOT,
                    &format!("Parameter '{}' in function '{}' cannot have type void", param.name, func.name),
                );
            }

            self.declare_local(&param.name, param.Type.clone())?;
        }

        let result = self.analyze_stmt(&func.body);

        self.pop_scope();
        self.exit_function();

        result
    }
    fn analyze_global_var(&mut self, var: &VarDecl) -> Result<(), String> {
        let ERROR_STRING_ROOT = "velc:Analyzer:analyze_global_var";

        if self.is_void_type(&var.Type) {
            return self.error(
                ERROR_STRING_ROOT,
                &format!("Global variable '{}' cannot have type void", var.name),
            );
        }

        if let Some(init) = &var.init {
            let init_type = self.analyze_expr(init)?;

            if !self.can_assign(&var.Type, &init_type) {
                return self.error(
                    ERROR_STRING_ROOT,
                    &format!(
                        "Cannot initialize global variable '{}' of type {:?} with value of type {:?}",
                        var.name,
                        var.Type,
                        init_type
                    ),
                );
            }
        }

        Ok(())
    }
    fn analyze_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        todo!("Fix")
    }
    fn analyze_expr(&mut self, expr: &Expr) -> Result<Typename, String> {
        todo!("Fix")
    }



    fn error<T>(&mut self, base: &str, err: &str) -> Result<T, String> {
        Err(format!("{base}:{err} \n"))
    }

}

