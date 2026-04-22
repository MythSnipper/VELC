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
    fn can_cast(&self, target: &TypeName, source: &TypeName) -> bool {
        if target == source {
            return true;
        }
    
        if self.is_void_type(target) || self.is_void_type(source) {
            return false;
        }
    
        if self.is_numeric_type(target) && self.is_numeric_type(source) {
            return true;
        }
    
        if self.is_pointer_type(target) && self.is_pointer_type(source) {
            return true;
        }
    
        if self.is_pointer_type(target) && self.is_integer_type(source) {
            return true;
        }
    
        if self.is_integer_type(target) && self.is_pointer_type(source) {
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
                if self.is_void_type(&var.Type) {
                    return self.error(
                        ERROR_STRING_ROOT,
                        &format!("Variable '{}' cannot have type void", var.name),
                    );
                }
    
                if let Some(init) = &var.init {
                    let init_type = self.analyze_expr(init)?;
    
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
    
            Stmt::Expr(expr) => {
                self.analyze_expr(expr)?;
                Ok(())
            }
    
            Stmt::Empty => Ok(()),
    
            _ => self.error(ERROR_STRING_ROOT, &format!("NOT IMPLEMENTED! {:?}", stmt)),
        }
    }
    fn analyze_expr(&mut self, expr: &Expr) -> Result<TypeName, String> {
        let ERROR_STRING_ROOT = "velc:Analyzer:analyze_expr";
    
        match expr {
            Expr::BoolLiteral(_) => {
                Ok(TypeName::Builtin(BuiltinType::Bool))
            }
    
            Expr::IntLiteral(_) => {
                Ok(TypeName::Builtin(BuiltinType::Int64))
            }
    
            Expr::FloatLiteral(_) => {
                Ok(TypeName::Builtin(BuiltinType::Float64))
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
                        if !self.is_numeric_type(&inner_type) {
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
                        if !self.is_integer_type(&inner_type) {
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
    
                        if !self.is_numeric_type(&inner_type) && !self.is_pointer_type(&inner_type) {
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
    
                if !self.is_numeric_type(&inner_type) && !self.is_pointer_type(&inner_type) {
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
                        if !self.is_numeric_type(&left_type) || !self.is_numeric_type(&right_type) {
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
    
                        if self.is_float_type(&left_type) || self.is_float_type(&right_type) {
                            Ok(TypeName::Builtin(BuiltinType::Float64))
                        }
                        else {
                            Ok(TypeName::Builtin(BuiltinType::Int64))
                        }
                    }

                    BinaryOp::Mod => {
                        if !self.is_integer_type(&left_type) || !self.is_integer_type(&right_type) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!(
                                    "Modulo operator requires integer operands, got {:?} and {:?}",
                                    left_type,
                                    right_type
                                ),
                            );
                        }
                    
                        Ok(left_type)
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
                        if !self.is_numeric_type(&left_type) || !self.is_numeric_type(&right_type) {
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
    
                        Ok(TypeName::Builtin(BuiltinType::Bool))
                    }
    
                    BinaryOp::BitwiseAnd
                    | BinaryOp::BitwiseOr
                    | BinaryOp::BitwiseXor
                    | BinaryOp::Lshift
                    | BinaryOp::Rshift => {
                        if !self.is_integer_type(&left_type) || !self.is_integer_type(&right_type) {
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
    
                        Ok(left_type)
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
                let left_type = self.analyze_expr(left)?;
                let right_type = self.analyze_expr(right)?;
    
                if !self.is_assignable_expr(left) {
                    return self.error(
                        ERROR_STRING_ROOT,
                        "Invalid assignment target",
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
                        if !self.is_numeric_type(&left_type) || !self.is_numeric_type(&right_type) {
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
    
                        Ok(left_type)
                    }

                    AssignOp::ModAssign => {
                        if !self.is_integer_type(&left_type) || !self.is_integer_type(&right_type) {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!(
                                    "Compound modulo assignment requires integer operands, got {:?} and {:?}",
                                    left_type,
                                    right_type
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
                        if !self.is_integer_type(&left_type) || !self.is_integer_type(&right_type) {
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
                match &**callee {
                    Expr::Identifier(name) => {
                        let sig = match self.lookup_function_signature(name) {
                            Some(sig) => sig.clone(),
                            None => {
                                return self.error(
                                    ERROR_STRING_ROOT,
                                    &format!("Attempt to call undefined function '{}'", name),
                                );
                            }
                        };
    
                        if args.len() != sig.params.len() {
                            return self.error(
                                ERROR_STRING_ROOT,
                                &format!(
                                    "Function '{}' expects {} arguments, got {}",
                                    name,
                                    sig.params.len(),
                                    args.len()
                                ),
                            );
                        }
    
                        for (arg, param_type) in args.iter().zip(sig.params.iter()) {
                            let arg_type = self.analyze_expr(arg)?;
    
                            if !self.can_assign(param_type, &arg_type) {
                                return self.error(
                                    ERROR_STRING_ROOT,
                                    &format!(
                                        "Function '{}' argument type mismatch: expected {:?}, got {:?}",
                                        name,
                                        param_type,
                                        arg_type
                                    ),
                                );
                            }
                        }
    
                        Ok(sig.ret)
                    }
    
                    _ => {
                        self.error(
                            ERROR_STRING_ROOT,
                            "Only direct function calls by identifier are supported right now",
                        )
                    }
                }
            }
    
            Expr::Index { base, index } => {
                let base_type = self.analyze_expr(base)?;
                let index_type = self.analyze_expr(index)?;
    
                if !self.is_integer_type(&index_type) {
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


    fn error<T>(&mut self, base: &str, err: &str) -> Result<T, String> {
        Err(format!("{base}:{err} \n"))
    }

}

