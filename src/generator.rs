use crate::{
    parser::Node,
    symbols::{Symbol, SymbolDefinition, SymbolTable, TypeTable},
    syntax::{Declaration, Expression, FunctionDefinition, LiteralKind, PostfixOp, Type, TypeId},
};

// #[derive(Error)]
// enum Error {}

pub enum ConstantResolveError<'a> {
    NoDefinition(String),
    InvalidSymbol(String, SymbolDefinition<'a>),
}

#[derive(Default)]
struct RoDataSection {
    entries: Vec<String>,
}

#[derive(Default)]
struct TextSection {
    entries: Vec<String>,
}

#[derive(Default)]
pub struct Sections {
    ro_data: RoDataSection,
    text: TextSection,
}

#[derive(Debug)]
pub struct Generator<'a> {
    ast: &'a [Node],

    types: TypeTable, // index via type id
    global_symbols: SymbolTable<'a>,
}

impl<'a> Generator<'a> {
    pub fn new(ast: &'a [Node]) -> Self {
        let mut types = TypeTable::new();
        let mut global_symbols = SymbolTable::new();

        // TODO: primitive types

        for node in ast {
            match node {
                Node::Function(function) => global_symbols.push(Symbol {
                    name: function.name.clone(),
                    definition: SymbolDefinition::Function(function),
                }),
                Node::ConstDecl(decl) => {
                    global_symbols.push(Symbol {
                        name: decl.name.clone(),
                        definition: SymbolDefinition::Variable(decl),
                    });
                }
                Node::StructDef(struct_def) => types.push(Type::Struct(struct_def.clone())),
            }
        }

        Self {
            ast,
            types,
            global_symbols,
        }
    }

    pub fn resolve_constant(&self, declaration: &'a Declaration) -> Vec<String> {
        // TODO: name mangling (use IDs)
        match &declaration.value {
            Expression::Literal { kind, value } => match kind {
                LiteralKind::Char => todo!(),
                LiteralKind::Integer => todo!(),
                LiteralKind::Float => todo!(),
                LiteralKind::String => {
                    let mut lines = vec![];
                    lines.push(format!("{} db \"{value}\"", declaration.name));
                    lines.push(format!(
                        "{}_len equ $ - {}",
                        declaration.name, declaration.name,
                    ));
                    lines
                }
            },
            _ => todo!("Implement constant abstract types"),
        }
    }

    pub fn generate_call(&self, _args: &Vec<Expression>) -> Vec<String> {
        todo!("Call generation...")
    }

    pub fn generate_function(&self, definition: &FunctionDefinition) -> Vec<String> {
        let mut lines = vec![];

        lines.push(String::new());

        if !definition.params.is_empty() {
            lines.push(format!("; {:?}", definition.params));
        }

        lines.push(format!("{}:", definition.name));

        for statement in &definition.statements {
            match statement {
                Expression::PrefixOperation { op, rhs } => todo!(),
                Expression::InfixOperation { lhs, op, rhs } => todo!(),
                Expression::PostfixOperation { lhs, op } => match op {
                    PostfixOp::Call { args } => lines.extend(self.generate_call(args)),
                    PostfixOp::Index(expression) => todo!(),
                    PostfixOp::Dot => todo!(),
                    PostfixOp::Try => todo!(),
                },
                Expression::Declaration { name, value } => todo!(),
                Expression::Identifier(_) => todo!(),
                Expression::Index { lhs, idx } => todo!(),
                Expression::Literal { kind, value } => todo!(),
                Expression::Return(expression) => todo!(),
            }
        }

        lines.push("ret".to_string());
        lines
    }

    pub fn generate_asm(&self) -> String {
        // TODO: create intermediary section structures to be assembled together

        let mut sections = Sections::default();

        for symbol in &self.global_symbols.symbols {
            match symbol.definition {
                SymbolDefinition::Variable(declaration) => sections
                    .ro_data
                    .entries
                    .extend(self.resolve_constant(declaration)),
                SymbolDefinition::Function(function_definition) => {
                    sections
                        .text
                        .entries
                        .extend(self.generate_function(function_definition));
                }
            }
        }

        let mut asm = String::new();

        asm.push_str("section .rodata\n\n");
        for entry in sections.ro_data.entries {
            asm.push_str(&entry);
            asm.push('\n');
        }

        asm.push_str(
            "section .text\n\n\
            global _start\n\
            _start:\n\n\
            call main\n\n\
            mov rdi, rax\n\
            mov rax, 60\n\
            syscall\n\
            ",
        );

        for entry in sections.text.entries {
            asm.push_str(&entry);
            asm.push('\n')
        }

        asm
    }

    fn _const_eval(_expr: Expression, _value_type: TypeId) {
        // TODO: implement constant evaluations

        todo!()
    }
}
