#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

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
    current_global_init: Option<String>,
    loop_depth: usize,
}
impl Analyzer {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            scopes: Vec::new(),
            current_function_ret: None,
            current_global_init: None,
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
    
        if let Some(current) = &self.current_global_init {
            if current == name {
                return None;
            }
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
    fn is_string_type(&self, Type: &TypeName) -> bool {
        matches!(Type, TypeName::Builtin(BuiltinType::String))
    }
    fn is_int_literal_type(&self, Type: &TypeName) -> bool {
        matches!(Type, TypeName::IntLiteral)
    }
    fn is_float_literal_type(&self, Type: &TypeName) -> bool {
        matches!(Type, TypeName::FloatLiteral)
    }
    fn is_integer_like_type(&self, Type: &TypeName) -> bool {
        self.is_integer_type(Type) || self.is_int_literal_type(Type)
    }
    fn is_float_like_type(&self, Type: &TypeName) -> bool {
        self.is_float_type(Type) || self.is_float_literal_type(Type)
    }
    fn is_numeric_like_type(&self, Type: &TypeName) -> bool {
        self.is_integer_like_type(Type) || self.is_float_like_type(Type)
    }
    fn get_pointee_type(&self, Type: &TypeName) -> Option<TypeName> {
        match Type {
            TypeName::Pointer(inner) => Some((**inner).clone()),
            _ => None,
        }
    }
    fn resolve_arithmetic_type(&self, left: &TypeName, right: &TypeName) -> Result<TypeName, String> {
        let ERROR_STRING_ROOT = "velc:Analyzer:resolve_arithmetic_type";
    
        // int literal + int literal => int literal
        if self.is_int_literal_type(left) && self.is_int_literal_type(right) {
            return Ok(TypeName::IntLiteral);
        }
    
        // float literal + float literal => float literal
        if self.is_float_literal_type(left) && self.is_float_literal_type(right) {
            return Ok(TypeName::FloatLiteral);
        }
    
        // concrete integer + int literal => concrete integer
        if self.is_integer_type(left) && self.is_int_literal_type(right) {
            return Ok(left.clone());
        }
    
        if self.is_int_literal_type(left) && self.is_integer_type(right) {
            return Ok(right.clone());
        }
    
        // concrete float + float literal => concrete float
        if self.is_float_type(left) && self.is_float_literal_type(right) {
            return Ok(left.clone());
        }
    
        if self.is_float_literal_type(left) && self.is_float_type(right) {
            return Ok(right.clone());
        }
    
        // integer + integer => choose wider concrete integer
        if self.is_integer_type(left) && self.is_integer_type(right) {
            return self.common_integer_type(left, right);
        }
    
        // float + float => choose wider concrete float
        if self.is_float_type(left) && self.is_float_type(right) {
            return self.common_float_type(left, right);
        }
    
        // int literal with concrete float => require explicit cast
        if (self.is_int_literal_type(left) && self.is_float_type(right))
            || (self.is_float_type(left) && self.is_int_literal_type(right))
            || (self.is_integer_type(left) && self.is_float_type(right))
            || (self.is_float_type(left) && self.is_integer_type(right))
            || (self.is_int_literal_type(left) && self.is_float_literal_type(right))
            || (self.is_float_literal_type(left) && self.is_int_literal_type(right))
            || (self.is_integer_type(left) && self.is_float_literal_type(right))
            || (self.is_float_literal_type(left) && self.is_integer_type(right))
        {
            return self.error(
                ERROR_STRING_ROOT,
                &format!(
                    "Mixed integer/float arithmetic requires explicit cast, got {:?} and {:?}",
                    left,
                    right
                ),
            );
        }
    
        self.error(
            ERROR_STRING_ROOT,
            &format!(
                "Could not resolve arithmetic type for {:?} and {:?}",
                left,
                right
            ),
        )
    }
    fn common_integer_type(&self, left: &TypeName, right: &TypeName) -> Result<TypeName, String> {
        let ERROR_STRING_ROOT = "velc:Analyzer:common_integer_type";
    
        let lw = match self.integer_width(left) {
            Some(w) => w,
            None => return self.error(ERROR_STRING_ROOT, &format!("Left type {:?} is not integer", left)),
        };
    
        let rw = match self.integer_width(right) {
            Some(w) => w,
            None => return self.error(ERROR_STRING_ROOT, &format!("Right type {:?} is not integer", right)),
        };
    
        if lw > rw {
            Ok(left.clone())
        }
        else if rw > lw {
            Ok(right.clone())
        }
        else {
            // same width: preserve left for now
            Ok(left.clone())
        }
    }
    fn common_float_type(&self, left: &TypeName, right: &TypeName) -> Result<TypeName, String> {
        let ERROR_STRING_ROOT = "velc:Analyzer:common_float_type";
    
        let lw = match self.float_width(left) {
            Some(w) => w,
            None => return self.error(ERROR_STRING_ROOT, &format!("Left type {:?} is not float", left)),
        };
    
        let rw = match self.float_width(right) {
            Some(w) => w,
            None => return self.error(ERROR_STRING_ROOT, &format!("Right type {:?} is not float", right)),
        };
    
        if lw >= rw {
            Ok(left.clone())
        }
        else {
            Ok(right.clone())
        }
    }

    fn is_array_type(&self, t: &TypeName) -> bool {
        matches!(t, TypeName::Array(_, _))
    }    
    fn array_element_type(&self, t: &TypeName) -> Option<TypeName> {
        match t {
            TypeName::Array(elem, _) => Some((**elem).clone()),
            _ => None,
        }
    }
    fn contains_array_type(&self, t: &TypeName) -> bool {
        match t {
            TypeName::Array(_, _) => true,
    
            TypeName::Pointer(inner) => {
                self.contains_array_type(inner)
            }
    
            TypeName::Function { ret, params } => {
                self.contains_array_type(ret)
                    || params.iter().any(|p| self.contains_array_type(p))
            }
    
            _ => false,
        }
    }

    fn validate_storage_type(&self, t: &TypeName, context: &str) -> Result<(), String> {
        let ERROR_STRING_ROOT = "velc:Analyzer:validate_storage_type";
    
        match t {
            TypeName::Builtin(BuiltinType::Void) => {
                self.error(
                    ERROR_STRING_ROOT,
                    &format!("{} cannot have type void", context),
                )
            }
    
            TypeName::Array(elem, size) => {
                if *size == 0 {
                    return self.error(
                        ERROR_STRING_ROOT,
                        &format!("{} cannot have zero-sized array type", context),
                    );
                }
    
                match elem.as_ref() {
                    TypeName::Builtin(BuiltinType::Void) => {
                        return self.error(
                            ERROR_STRING_ROOT,
                            &format!("{} cannot be an array of void", context),
                        );
                    }
    
                    TypeName::Function { .. } => {
                        return self.error(
                            ERROR_STRING_ROOT,
                            &format!("{} cannot be an array of function values", context),
                        );
                    }
    
                    _ => {}
                }
    
                self.validate_storage_type(elem, context)
            }
    
            TypeName::Function { .. } => {
                self.error(
                    ERROR_STRING_ROOT,
                    &format!("{} cannot have bare function type", context),
                )
            }
    
            TypeName::IntLiteral | TypeName::FloatLiteral => {
                self.error(
                    ERROR_STRING_ROOT,
                    &format!("{} cannot have literal pseudo-type", context),
                )
            }
    
            _ => Ok(()),
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
    fn is_condition_type(&self, Type: &TypeName) -> bool {

        if self.is_array_type(Type) {
            return false;
        }


        self.is_bool_type(Type)
            || self.is_integer_type(Type)
            || self.is_float_type(Type)
            || self.is_pointer_type(Type)
    }
    fn is_static_global_initializer(&self, expr: &Expr) -> bool {
        match expr {
            Expr::BoolLiteral(_) => true,
            Expr::IntLiteral(_) => true,
            Expr::FloatLiteral(_) => true,
            Expr::CharLiteral(_) => true,
            Expr::StringLiteral(_) => true,
    
            Expr::Cast { expr, .. } => self.is_static_global_initializer(expr),
    
            _ => false,
        }
    }
    fn is_valid_asm_register(&self, reg: &str) -> bool {
        matches!(
            reg,
            "rax" | "rbx" | "rcx" | "rdx" |
            "rsi" | "rdi" |
            "rbp" | "rsp" |
            "r8"  | "r9"  | "r10" | "r11" |
            "r12" | "r13" | "r14" | "r15"
        )
    }

    fn integer_width(&self, Type: &TypeName) -> Option<u8> {
        match Type {
            TypeName::Builtin(BuiltinType::Int8) => Some(8),
            TypeName::Builtin(BuiltinType::Uint8) => Some(8),
            TypeName::Builtin(BuiltinType::Int16) => Some(16),
            TypeName::Builtin(BuiltinType::Uint16) => Some(16),
            TypeName::Builtin(BuiltinType::Int32) => Some(32),
            TypeName::Builtin(BuiltinType::Uint32) => Some(32),
            TypeName::Builtin(BuiltinType::Int64) => Some(64),
            TypeName::Builtin(BuiltinType::Uint64) => Some(64),
            TypeName::Builtin(BuiltinType::Char) => Some(8),
            TypeName::Pointer(_) => Some(64),
            _ => None,
        }
    }
    fn float_width(&self, Type: &TypeName) -> Option<u8> {
        match Type {
            TypeName::Builtin(BuiltinType::Float32) => Some(32),
            TypeName::Builtin(BuiltinType::Float64) => Some(64),
            _ => None,
        }
    }
    
    fn can_assign(&self, target: &TypeName, value: &TypeName) -> bool {

        if self.is_array_type(target) || self.is_array_type(value) {
            return false;
        }

        if target == value {
            return true;
        }
    
        if self.is_void_type(target) || self.is_void_type(value) {
            return false;
        }
    
        // function -> pointer-to-function
        if let TypeName::Pointer(inner) = target {
            if **inner == *value {
                if matches!(**inner, TypeName::Function { .. }) {
                    return true;
                }
            }
        }
    
        match value {
            TypeName::IntLiteral => {
                return self.is_integer_type(target);
            }
    
            TypeName::FloatLiteral => {
                return self.is_float_type(target);
            }
    
            _ => {}
        }
    
        if let (Some(target_w), Some(value_w)) = (self.integer_width(target), self.integer_width(value)) {
            return value_w <= target_w;
        }
    
        if let (Some(target_w), Some(value_w)) = (self.float_width(target), self.float_width(value)) {
            return value_w <= target_w;
        }
    
        false
    }
    fn can_cast(&self, target: &TypeName, source: &TypeName) -> bool {

        if self.is_array_type(target) || self.is_array_type(source) {
            return false;
        }

        if target == source {
            return true;
        }

        if self.is_void_type(target) || self.is_void_type(source) {
            return false;
        }

        // literal -> integer
        if self.is_integer_type(target) && self.is_int_literal_type(source) {
            return true;
        }

        if self.is_integer_type(target) && self.is_float_literal_type(source) {
            return true;
        }

        // literal -> float
        if self.is_float_type(target) && self.is_int_literal_type(source) {
            return true;
        }

        if self.is_float_type(target) && self.is_float_literal_type(source) {
            return true;
        }

        // concrete integer <-> concrete integer
        if self.integer_width(target).is_some() && self.integer_width(source).is_some() {
            return true;
        }

        // concrete float <-> concrete float
        if self.float_width(target).is_some() && self.float_width(source).is_some() {
            return true;
        }

        // concrete integer <-> concrete float
        if self.integer_width(target).is_some() && self.float_width(source).is_some() {
            return true;
        }

        if self.float_width(target).is_some() && self.integer_width(source).is_some() {
            return true;
        }

        // pointer <-> pointer
        if self.is_pointer_type(target) && self.is_pointer_type(source) {
            return true;
        }

        // string <-> pointer
        //
        // string is backend-lowered as a pointer-sized value, but semantically it is
        // still its own builtin type. So allow this only as an explicit cast.
        if self.is_string_type(target) && self.is_pointer_type(source) {
            return true;
        }

        if self.is_pointer_type(target) && self.is_string_type(source) {
            return true;
        }

        // pointer <-> integer
        if self.is_pointer_type(target) && self.integer_width(source).is_some() {
            return true;
        }

        if self.integer_width(target).is_some() && self.is_pointer_type(source) {
            return true;
        }

        // string <-> pointer
        if self.is_string_type(target) && self.is_pointer_type(source) {
            return true;
        }

        if self.is_pointer_type(target) && self.is_string_type(source) {
            return true;
        }

        // string <-> integer
        if self.is_string_type(target) && self.integer_width(source).is_some() {
            return true;
        }

        if self.integer_width(target).is_some() && self.is_string_type(source) {
            return true;
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
        match item {
            TopLevel::Function(func) => {
                self.register_function(func)?;
            }
            TopLevel::GlobalVar(var) => {
                self.register_global_var(var)?;
            }
            TopLevel::Assembly(..) => {}
        }
        Ok(())
    }
    fn register_function(&mut self, func: &FunctionDecl) -> Result<(), String> {
        let ERROR_STRING_ROOT = "velc:Analyzer:register_function";
    
        if self.lookup_global(&func.name).is_some() {
            return self.error(
                ERROR_STRING_ROOT,
                &format!("Top level symbol '{}' already declared", func.name),
            );
        }
    
        if self.is_array_type(&func.ret) {
            return self.error(
                ERROR_STRING_ROOT,
                "Function cannot return array type",
            );
        }

        if matches!(func.ret, TypeName::Function { .. }) {
            return self.error(
                ERROR_STRING_ROOT,
                "Function cannot return bare function type",
            );
        }


        let mut params = Vec::new();
    
        for param in &func.params {
            if self.is_void_type(&param.Type) {
                return self.error(
                    ERROR_STRING_ROOT,
                    &format!(
                        "Parameter '{}' in function '{}' cannot have type void",
                        param.name,
                        func.name
                    ),
                );
            }

            if self.is_array_type(&param.Type) {
                return self.error(
                    ERROR_STRING_ROOT,
                    "Array parameters are not implemented yet; use a pointer parameter",
                );
            }

            self.validate_storage_type(&param.Type, "Function parameter")?;
    
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

        self.validate_storage_type(
            &var.Type,
            &format!("Global variable '{}'", var.name),
        )?;

        self.globals.insert(
            var.name.clone(),
            GlobalSymbol::GlobalVar(var.Type.clone()),
        );

        Ok(())
    }

    fn analyze_program(&mut self, program: &Program) -> Result<(), String> {
        for item in &program.items {
            self.register_top_level(item)?;
        }

        for item in &program.items {
            self.analyze_top_level(item)?;
        }

        Ok(())
    }
    fn analyze_top_level(&mut self, item: &TopLevel) -> Result<(), String> {
        match item {
            TopLevel::Function(func) => self.analyze_function(func),
            TopLevel::GlobalVar(var) => self.analyze_global_var(var),
            TopLevel::Assembly(..) => Ok(())
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
                    &format!(
                        "Parameter '{}' in function '{}' cannot have type void",
                        param.name,
                        func.name
                    ),
                );
            }
            if matches!(param.Type, TypeName::Array(_, _)) {
                return self.error(
                    ERROR_STRING_ROOT,
                    "Array parameters are not implemented yet; use a pointer parameter",
                );
            }
    
            if let Err(err) = self.declare_local(&param.name, param.Type.clone()) {
                self.pop_scope();
                self.exit_function();
                return Err(err);
            }
        }
    
        let result = self.analyze_stmt(&func.body);
    
        self.pop_scope();
        self.exit_function();
    
        result
    }
    fn analyze_global_var(&mut self, var: &VarDecl) -> Result<(), String> {
        let ERROR_STRING_ROOT = "velc:Analyzer:analyze_global_var";
    
        self.validate_storage_type(&var.Type, "Global variable")?;

        if self.is_array_type(&var.Type) {
            if var.init.is_some() {
                return self.error(
                    ERROR_STRING_ROOT,
                    "Global array initializers are not implemented yet",
                );
            }
    
            return Ok(());
        }


    
        if let Some(init) = &var.init {
            if !self.is_static_global_initializer(init) {
                return self.error(
                    ERROR_STRING_ROOT,
                    &format!(
                        "Global variable '{}' initializer must be static",
                        var.name
                    ),
                );
            }
    
            let init_type = self.analyze_expr(init)?;
    

            if self.is_array_type(&init_type) {
                return self.error(
                    ERROR_STRING_ROOT,
                    "Cannot initialize global variable from array value",
                );
            }

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

    fn analyze_assembly_decl(&mut self, decl: &AssemblyDecl) -> Result<(), String> {
        let ERROR_STRING_ROOT = "velc:Analyzer:analyze_assembly_decl";
    
        for input in &decl.inputs {
            if self.lookup_variable_type(&input.name).is_none() {
                return self.error(
                    ERROR_STRING_ROOT,
                    &format!("Inline assembly input '{}' is not a declared variable", input.name),
                );
            }
    
            if !self.is_valid_asm_register(&input.reg) {
                return self.error(
                    ERROR_STRING_ROOT,
                    &format!("Inline assembly input register '{}' is invalid", input.reg),
                );
            }
        }
    
        for output in &decl.outputs {
            if self.lookup_variable_type(&output.name).is_none() {
                return self.error(
                    ERROR_STRING_ROOT,
                    &format!("Inline assembly output '{}' is not a declared variable", output.name),
                );
            }
    
            if !self.is_valid_asm_register(&output.reg) {
                return self.error(
                    ERROR_STRING_ROOT,
                    &format!("Inline assembly output register '{}' is invalid", output.reg),
                );
            }
        }
    
        Ok(())
    }


    fn analyze_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        let ERROR_STRING_ROOT = "velc:Analyzer:analyze_stmt";
    
        match stmt {
            Stmt::Block(stmts) => {
                self.push_scope();
    
                for stmt in stmts {
                    if let Err(err) = self.analyze_stmt(stmt) {
                        self.pop_scope();
                        return self.error(ERROR_STRING_ROOT, &err);
                    }
                }
    
                self.pop_scope();
                Ok(())
            }

            Stmt::VarDecl(var) => {
                self.validate_storage_type(&var.Type, "Local variable")?;

                if self.is_array_type(&var.Type) {
                    if var.init.is_some() {
                        return self.error(
                            ERROR_STRING_ROOT,
                            "Array initializers are not implemented yet",
                        );
                    }
            
                    self.declare_local(&var.name, var.Type.clone())?;
                    return Ok(());
                }
    
                if let Some(init) = &var.init {
                    let init_type = self.analyze_expr(init)?;
    
                    if self.is_array_type(&init_type) {
                        return self.error(
                            ERROR_STRING_ROOT,
                            "Cannot initialize variable from array value",
                        );
                    }

                    if !self.can_assign(&var.Type, &init_type) {
                        return self.error(
                            ERROR_STRING_ROOT,
                            &format!(
                                "Cannot initialize variable '{}' of type {:?} with value of type {:?}",
                                var.name,
                                var.Type,
                                init_type
                            ),
                        );
                    }
                }
    
                self.declare_local(&var.name, var.Type.clone())
            }
    
            Stmt::Return(expr) => {
                let expected = match self.current_return_type() {
                    Some(t) => t.clone(),
                    None => {
                        return self.error(
                            ERROR_STRING_ROOT,
                            "Return statement outside of function",
                        );
                    }
                };
    
                match expr {
                    Some(expr) => {
                        if self.is_void_type(&expected) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                "Void function cannot return a value",
                            );
                        }
    
                        let got = self.analyze_expr(expr)?;
    
                        if !self.can_assign(&expected, &got) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!(
                                    "Return type mismatch: expected {:?}, got {:?}",
                                    expected,
                                    got
                                ),
                            );
                        }
    
                        Ok(())
                    }
                    None => {
                        if !self.is_void_type(&expected) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!(
                                    "Non-void function must return a value of type {:?}",
                                    expected,
                                ),
                            );
                        }
    
                        Ok(())
                    }
                }
            }

            Stmt::Break => {
                if !self.is_inside_loop() {
                    return self.error(
                        ERROR_STRING_ROOT,
                        "Break statement outside of loop",
                    );
                }
    
                Ok(())
            }
    
            Stmt::Continue => {
                if !self.is_inside_loop() {
                    return self.error(
                        ERROR_STRING_ROOT,
                        "Continue statement outside of loop",
                    );
                }
    
                Ok(())
            }

            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_type = self.analyze_expr(cond)?;
    
                if !self.is_condition_type(&cond_type) {
                    return self.error(
                        ERROR_STRING_ROOT,
                        &format!(
                            "If condition must be a bool, numeric, or pointer type, got {:?}",
                            cond_type
                        ),
                    );
                }
    
                self.analyze_stmt(then_branch)?;
    
                if let Some(else_branch) = else_branch {
                    self.analyze_stmt(else_branch)?;
                }
    
                Ok(())
            }

            Stmt::While { cond, body } => {
                let cond_type = self.analyze_expr(cond)?;
    
                if !self.is_condition_type(&cond_type) {
                    return self.error(
                        ERROR_STRING_ROOT,
                        &format!(
                            "While condition must be a bool, numeric, or pointer type, got {:?}",
                            cond_type
                        ),
                    );
                }
    
                self.enter_loop();
                let result = self.analyze_stmt(body);
                self.exit_loop();
    
                result
            }
    
            Stmt::For {
                init,
                cond,
                step,
                body,
            } => {
                self.push_scope();
    
                if let Some(init) = init {
                    match init {
                        ForInit::VarDecl(var) => {
                            if self.is_void_type(&var.Type) {
                                self.pop_scope();
                                return self.error(
                                    ERROR_STRING_ROOT,
                                    &format!("Variable '{}' cannot have type void", var.name),
                                );
                            }
    
                            if let Some(init_expr) = &var.init {
                                let init_type = match self.analyze_expr(init_expr) {
                                    Ok(t) => t,
                                    Err(err) => {
                                        self.pop_scope();
                                        return Err(err);
                                    }
                                };
    
                                if !self.can_assign(&var.Type, &init_type) {
                                    self.pop_scope();
                                    return self.error(
                                        ERROR_STRING_ROOT,
                                        &format!(
                                            "Cannot initialize variable '{}' of type {:?} with value of type {:?}",
                                            var.name,
                                            var.Type,
                                            init_type
                                        ),
                                    );
                                }
                            }
    
                            if let Err(err) = self.declare_local(&var.name, var.Type.clone()) {
                                self.pop_scope();
                                return Err(err);
                            }
                        }
    
                        ForInit::Expr(expr) => {
                            if let Err(err) = self.analyze_expr(expr) {
                                self.pop_scope();
                                return Err(err);
                            }
                        }
                    }
                }
    
                if let Some(cond) = cond {
                    let cond_type = match self.analyze_expr(cond) {
                        Ok(t) => t,
                        Err(err) => {
                            self.pop_scope();
                            return Err(err);
                        }
                    };
    
                    if !self.is_condition_type(&cond_type) {
                        self.pop_scope();
                        return self.error(
                            ERROR_STRING_ROOT,
                            &format!(
                                "For condition must be a bool, numeric, or pointer type, got {:?}",
                                cond_type
                            ),
                        );
                    }
                }
    
                if let Some(step) = step {
                    if let Err(err) = self.analyze_expr(step) {
                        self.pop_scope();
                        return Err(err);
                    }
                }
    
                self.enter_loop();
                let body_result = self.analyze_stmt(body);
                self.exit_loop();
    
                self.pop_scope();
    
                body_result
            }
            
    
            Stmt::Expr(expr) => {
                self.analyze_expr(expr)?;
                Ok(())
            }
    
            Stmt::Assembly(decl) => {
                self.analyze_assembly_decl(decl)
            }

            Stmt::Empty => Ok(()),
        }
    }
    fn analyze_expr(&mut self, expr: &Expr) -> Result<TypeName, String> {
        let ERROR_STRING_ROOT = "velc:Analyzer:analyze_expr";
    
        match expr {
            Expr::BoolLiteral(_) => {
                Ok(TypeName::Builtin(BuiltinType::Bool))
            }
    
            Expr::IntLiteral(_) => {
                Ok(TypeName::IntLiteral)
            }
    
            Expr::FloatLiteral(_) => {
                Ok(TypeName::FloatLiteral)
            }
    
            Expr::CharLiteral(_) => {
                Ok(TypeName::Builtin(BuiltinType::Char))
            }
    
            Expr::StringLiteral(_) => {
                Ok(TypeName::Builtin(BuiltinType::String))
            }
    
            Expr::Identifier(name) => {
                if let Some(Type) = self.lookup_variable_type(name) {
                    Ok(Type)
                }
                else if let Some(sig) = self.lookup_function_signature(name) {
                    Ok(TypeName::Function {
                        ret: Box::new(sig.ret.clone()),
                        params: sig.params.clone(),
                    })
                }
                else {
                    self.error(
                        ERROR_STRING_ROOT,
                        &format!("Use of undeclared identifier '{}'", name),
                    )
                }
            }
    
            Expr::Grouping(inner) => {
                self.analyze_expr(inner)
            }
    
            Expr::Prefix { op, expr } => {
                let inner_type = self.analyze_expr(expr)?;
    
                match op {
                    PrefixOp::Plus | PrefixOp::Neg => {
                        if !self.is_numeric_like_type(&inner_type) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!("Unary operator {:?} requires numeric operand, got {:?}", op, inner_type),
                            );
                        }
    
                        Ok(inner_type)
                    }
    
                    PrefixOp::LogicalNot => {
                        if !self.is_bool_type(&inner_type) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!("Logical not requires bool operand, got {:?}", inner_type),
                            );
                        }
    
                        Ok(TypeName::Builtin(BuiltinType::Bool))
                    }
    
                    PrefixOp::BitwiseNot => {
                        if !self.is_integer_like_type(&inner_type) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!("Bitwise not requires integer operand, got {:?}", inner_type),
                            );
                        }
    
                        Ok(inner_type)
                    }
    
                    PrefixOp::Ref => {
                        if !self.is_addressable_expr(expr) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                "Cannot take address of non-addressable expression",
                            );
                        }
    
                        Ok(TypeName::Pointer(Box::new(inner_type)))
                    }
    
                    PrefixOp::Deref => {
                        match self.get_pointee_type(&inner_type) {
                            Some(Type) => Ok(Type),
                            None => {
                                self.error(
                                    ERROR_STRING_ROOT,
                                    &format!("Cannot dereference non-pointer type {:?}", inner_type),
                                )
                            }
                        }
                    }
    
                    PrefixOp::Inc | PrefixOp::Dec => {
                        if !self.is_assignable_expr(expr) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                "Increment/decrement requires assignable expression",
                            );
                        }
    
                        if !self.is_numeric_like_type(&inner_type) && !self.is_pointer_type(&inner_type) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!(
                                    "Increment/decrement requires numeric or pointer operand, got {:?}",
                                    inner_type
                                ),
                            );
                        }
    
                        Ok(inner_type)
                    }
                }
            }
    
            Expr::Postfix { op: _, expr } => {
                let inner_type = self.analyze_expr(expr)?;
    
                if !self.is_assignable_expr(expr) {
                    return self.error(
                        ERROR_STRING_ROOT,
                        "Postfix increment/decrement requires assignable expression",
                    );
                }
    
                if !self.is_numeric_like_type(&inner_type) && !self.is_pointer_type(&inner_type) {
                    return self.error(
                        ERROR_STRING_ROOT,
                        &format!(
                            "Postfix increment/decrement requires numeric or pointer operand, got {:?}",
                            inner_type
                        ),
                    );
                }
    
                Ok(inner_type)
            }
    
            Expr::Binary { left, op, right } => {
                let left_type = self.analyze_expr(left)?;
                let right_type = self.analyze_expr(right)?;
    
                match op {
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div => {
                        if !self.is_numeric_like_type(&left_type) || !self.is_numeric_like_type(&right_type) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!(
                                    "Arithmetic operator {:?} requires numeric operands, got {:?} and {:?}",
                                    op,
                                    left_type,
                                    right_type
                                ),
                            );
                        }
    
                        self.resolve_arithmetic_type(&left_type, &right_type)
                    }
    
                    BinaryOp::Mod => {
                        if !self.is_integer_like_type(&left_type) || !self.is_integer_like_type(&right_type) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!(
                                    "Modulo operator requires integer operands, got {:?} and {:?}",
                                    left_type,
                                    right_type
                                ),
                            );
                        }
    
                        if self.is_int_literal_type(&left_type) && self.is_int_literal_type(&right_type) {
                            return Ok(TypeName::IntLiteral);
                        }
    
                        if self.is_integer_type(&left_type) && self.is_int_literal_type(&right_type) {
                            return Ok(left_type);
                        }
    
                        if self.is_int_literal_type(&left_type) && self.is_integer_type(&right_type) {
                            return Ok(right_type);
                        }
    
                        self.common_integer_type(&left_type, &right_type)
                    }
    
                    BinaryOp::Eq
                    | BinaryOp::Neq => {
                        if !self.can_assign(&left_type, &right_type) && !self.can_assign(&right_type, &left_type) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!(
                                    "Equality operator {:?} requires compatible operands, got {:?} and {:?}",
                                    op,
                                    left_type,
                                    right_type
                                ),
                            );
                        }
    
                        Ok(TypeName::Builtin(BuiltinType::Bool))
                    }
    
                    BinaryOp::Lt
                    | BinaryOp::Gt
                    | BinaryOp::Lte
                    | BinaryOp::Gte => {
                        if !self.is_numeric_like_type(&left_type) || !self.is_numeric_like_type(&right_type) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!(
                                    "Relational operator {:?} requires numeric operands, got {:?} and {:?}",
                                    op,
                                    left_type,
                                    right_type
                                ),
                            );
                        }
    
                        self.resolve_arithmetic_type(&left_type, &right_type)?;
    
                        Ok(TypeName::Builtin(BuiltinType::Bool))
                    }
    
                    BinaryOp::BitwiseAnd
                    | BinaryOp::BitwiseOr
                    | BinaryOp::BitwiseXor
                    | BinaryOp::Lshift
                    | BinaryOp::Rshift => {
                        if !self.is_integer_like_type(&left_type) || !self.is_integer_like_type(&right_type) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!(
                                    "Bitwise operator {:?} requires integer operands, got {:?} and {:?}",
                                    op,
                                    left_type,
                                    right_type
                                ),
                            );
                        }
    
                        if self.is_int_literal_type(&left_type) && self.is_int_literal_type(&right_type) {
                            Ok(TypeName::IntLiteral)
                        }
                        else if self.is_integer_type(&left_type) && self.is_int_literal_type(&right_type) {
                            Ok(left_type)
                        }
                        else if self.is_int_literal_type(&left_type) && self.is_integer_type(&right_type) {
                            Ok(right_type)
                        }
                        else {
                            self.common_integer_type(&left_type, &right_type)
                        }
                    }
    
                    BinaryOp::LogicalAnd
                    | BinaryOp::LogicalOr
                    | BinaryOp::LogicalXor => {
                        if !self.is_bool_type(&left_type) || !self.is_bool_type(&right_type) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!(
                                    "Logical operator {:?} requires bool operands, got {:?} and {:?}",
                                    op,
                                    left_type,
                                    right_type
                                ),
                            );
                        }
    
                        Ok(TypeName::Builtin(BuiltinType::Bool))
                    }
                }
            }
    
            Expr::Assign { left, op, right } => {
                if !self.is_assignable_expr(left) {
                    return self.error(
                        ERROR_STRING_ROOT,
                        "Invalid assignment target",
                    );
                }

                let left_type = self.analyze_expr(left)?;

                if self.is_array_type(&left_type) {
                    return self.error(
                        ERROR_STRING_ROOT,
                        "Cannot assign to whole array value",
                    );
                }

                let right_type = self.analyze_expr(right)?;

                if self.is_array_type(&right_type) {
                    return self.error(
                        ERROR_STRING_ROOT,
                        "Cannot assign array value",
                    );
                }
    
                match op {
                    AssignOp::Assign => {
                        if !self.can_assign(&left_type, &right_type) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!(
                                    "Cannot assign value of type {:?} to target of type {:?}",
                                    right_type,
                                    left_type
                                ),
                            );
                        }
    
                        Ok(left_type)
                    }
    
                    AssignOp::AddAssign
                    | AssignOp::SubAssign
                    | AssignOp::MulAssign
                    | AssignOp::DivAssign => {
                        if !self.is_numeric_like_type(&left_type) || !self.is_numeric_like_type(&right_type) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!(
                                    "Compound arithmetic assignment {:?} requires numeric operands, got {:?} and {:?}",
                                    op,
                                    left_type,
                                    right_type
                                ),
                            );
                        }
    
                        let result_type = self.resolve_arithmetic_type(&left_type, &right_type)?;
    
                        if !self.can_assign(&left_type, &result_type) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!(
                                    "Result of compound assignment {:?} has type {:?}, which cannot be assigned to {:?}",
                                    op,
                                    result_type,
                                    left_type
                                ),
                            );
                        }
    
                        Ok(left_type)
                    }
    
                    AssignOp::ModAssign => {
                        if !self.is_integer_like_type(&left_type) || !self.is_integer_like_type(&right_type) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!(
                                    "Compound modulo assignment requires integer operands, got {:?} and {:?}",
                                    left_type,
                                    right_type
                                ),
                            );
                        }
    
                        let result_type = if self.is_int_literal_type(&right_type) {
                            left_type.clone()
                        }
                        else {
                            self.common_integer_type(&left_type, &right_type)?
                        };
    
                        if !self.can_assign(&left_type, &result_type) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!(
                                    "Result of compound modulo assignment has type {:?}, which cannot be assigned to {:?}",
                                    result_type,
                                    left_type
                                ),
                            );
                        }
    
                        Ok(left_type)
                    }
    
                    AssignOp::AndAssign
                    | AssignOp::OrAssign
                    | AssignOp::XorAssign
                    | AssignOp::LshiftAssign
                    | AssignOp::RshiftAssign
                    | AssignOp::NotAssign => {
                        if !self.is_integer_like_type(&left_type) || !self.is_integer_like_type(&right_type) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!(
                                    "Compound bitwise assignment {:?} requires integer operands, got {:?} and {:?}",
                                    op,
                                    left_type,
                                    right_type
                                ),
                            );
                        }
    
                        Ok(left_type)
                    }
                }
            }
    
            Expr::Call { callee, args } => {
                let callee_type = self.analyze_expr(callee)?;
            
                let (ret_type, param_types) = match callee_type {
                    TypeName::Function { ret, params } => ((*ret).clone(), params),
            
                    TypeName::Pointer(inner) => {
                        match *inner {
                            TypeName::Function { ret, params } => ((*ret).clone(), params),
                            other => {
                                return self.error(
                                    ERROR_STRING_ROOT,
                                    &format!("Attempt to call non-function pointer type {:?}", other),
                                );
                            }
                        }
                    }
            
                    other => {
                        return self.error(
                            ERROR_STRING_ROOT,
                            &format!("Attempt to call non-function type {:?}", other),
                        );
                    }
                };
            
                if args.len() != param_types.len() {
                    return self.error(
                        ERROR_STRING_ROOT,
                        &format!(
                            "Function expects {} arguments, got {}",
                            param_types.len(),
                            args.len()
                        ),
                    );
                }
            
                for (arg, param_type) in args.iter().zip(param_types.iter()) {
                    let arg_type = self.analyze_expr(arg)?;
            
                    if !self.can_assign(param_type, &arg_type) {
                        return self.error(
                            ERROR_STRING_ROOT,
                            &format!(
                                "Function argument type mismatch: expected {:?}, got {:?}",
                                param_type,
                                arg_type
                            ),
                        );
                    }
                }
            
                Ok(ret_type)
            }
    
            Expr::Index { base, index } => {
                let base_type = self.analyze_expr(base)?;
                let index_type = self.analyze_expr(index)?;
    
                if !self.is_integer_like_type(&index_type) {
                    return self.error(
                        ERROR_STRING_ROOT,
                        &format!("Index expression must have integer type, got {:?}", index_type),
                    );
                }
    
                match base_type {
                    TypeName::Pointer(inner) => Ok(*inner),
                    TypeName::Array(inner, _) => Ok(*inner),
                    _ => {
                        self.error(
                            ERROR_STRING_ROOT,
                            &format!("Cannot index non-pointer/non-array type {:?}", base_type),
                        )
                    }
                }
            }
    
            Expr::Member { .. } => {
                self.error(
                    ERROR_STRING_ROOT,
                    "Member access is not supported yet",
                )
            }
    
            Expr::Cast { Type, expr } => {
                let source_type = self.analyze_expr(expr)?;
    
                if !self.can_cast(Type, &source_type) {
                    return self.error(
                        ERROR_STRING_ROOT,
                        &format!(
                            "Cannot cast expression of type {:?} to type {:?}",
                            source_type,
                            Type
                        ),
                    );
                }
    
                Ok(Type.clone())
            }
        }
    }


    fn error<T>(&self, base: &str, err: &str) -> Result<T, String> {
        Err(format!("{base}:{err} \n"))
    }

}

