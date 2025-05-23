use std::collections::HashMap;

pub mod token {
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum Arithmetic {
        Add, // +
        Sub, // -
        Mul, // *
        Div, // /
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum Parentheses {
        L, // (
        R, // )
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum BlockBrace {
        L, // {
        R, // }
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum Comparison {
        Eq,  // ==
        Neq, // !=
        Lt,  // <
        Le,  // <=
        Gt,  // >
        Ge,  // >=
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum ExprSymbol {
        Arithmetic(Arithmetic),
        Parentheses(Parentheses),
        BlockDelimiter(BlockBrace),
        Comparison(Comparison),
        Assignment,
        Stop,  //;
        Comma, //,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub enum Value {
        Number(usize),
        Ident(String),
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum ControlStructure {
        If,
        Else,
        For,
        While,
        Return,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub enum Token {
        ControlStructure(ControlStructure),
        ExprSymbol(ExprSymbol),
        Value(Value),
    }

    impl Token {
        pub const fn arith(a: Arithmetic) -> Self {
            Self::ExprSymbol(ExprSymbol::Arithmetic(a))
        }
        pub const fn comp(c: Comparison) -> Self {
            Self::ExprSymbol(ExprSymbol::Comparison(c))
        }
        pub const fn paren(p: Parentheses) -> Self {
            Self::ExprSymbol(ExprSymbol::Parentheses(p))
        }
        pub const fn block(b: BlockBrace) -> Self {
            Self::ExprSymbol(ExprSymbol::BlockDelimiter(b))
        }
        pub const fn assign() -> Self {
            Self::ExprSymbol(ExprSymbol::Assignment)
        }
        pub const fn stop() -> Self {
            Self::ExprSymbol(ExprSymbol::Stop)
        }
        pub const fn comma() -> Self {
            Self::ExprSymbol(ExprSymbol::Comma)
        }
        pub const fn ctrl(c: ControlStructure) -> Self {
            Self::ControlStructure(c)
        }
        pub const fn number(n: usize) -> Self {
            Self::Value(Value::Number(n))
        }
        pub fn ident(name: impl Into<String>) -> Self {
            Self::Value(Value::Ident(name.into()))
        }
    }

    impl Token {
        pub const SYMBOLS: [(&str, Self); 22] = [
            ("+", Self::arith(Arithmetic::Add)),
            ("-", Self::arith(Arithmetic::Sub)),
            ("*", Self::arith(Arithmetic::Mul)),
            ("/", Self::arith(Arithmetic::Div)),
            ("(", Self::paren(Parentheses::L)),
            (")", Self::paren(Parentheses::R)),
            ("{", Self::block(BlockBrace::L)),
            ("}", Self::block(BlockBrace::R)),
            ("==", Self::comp(Comparison::Eq)),
            ("!=", Self::comp(Comparison::Neq)),
            ("<", Self::comp(Comparison::Lt)),
            ("<=", Self::comp(Comparison::Le)),
            (">", Self::comp(Comparison::Gt)),
            (">=", Self::comp(Comparison::Ge)),
            ("=", Self::assign()),
            (";", Self::stop()),
            (",", Self::comma()),
            ("if", Self::ctrl(ControlStructure::If)),
            ("else", Self::ctrl(ControlStructure::Else)),
            ("while", Self::ctrl(ControlStructure::While)),
            ("for", Self::ctrl(ControlStructure::For)),
            ("return", Self::ctrl(ControlStructure::Return)),
        ];

        pub fn classify(input: &str) -> Option<Self> {
            for (symbol, token) in Self::SYMBOLS.iter() {
                if *symbol == input {
                    return Some(token.clone());
                }
            }
            None
        }
    }
}

pub mod node {
    use crate::setting::token::{ExprSymbol, Value};
    #[derive(Debug, PartialEq, Clone)]
    pub struct Expr {
        pub op: ExprSymbol,
        pub lhs: Box<Node>,
        pub rhs: Box<Node>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Call {
        pub callee: Box<Node>,         // 関数名（識別子）など
        pub arguments: Vec<Box<Node>>, // 引数
    }
    #[derive(Debug, PartialEq, Clone)]
    pub struct If {
        pub condition: Box<Node>,
        pub then_branch: Box<Node>,
        pub else_branch: Option<Box<Node>>,
    }
    #[derive(Debug, PartialEq, Clone)]
    pub struct While {
        pub condition: Box<Node>,
        pub body: Box<Node>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct For {
        pub initializer: Option<Box<Node>>,
        pub condition: Box<Node>,
        pub updater: Option<Box<Node>>,
        pub body: Box<Node>,
    }
    #[derive(Debug, PartialEq, Clone)]
    pub struct Return {
        pub value: Box<Node>,
    }
    #[derive(Debug, PartialEq, Clone)]
    pub enum Control {
        If(If),
        While(While),
        For(For),
        Return(Return),
    }
    #[derive(Debug, PartialEq, Clone)]
    pub struct Function {
        pub name: Value, //本当はIdentが良いのだが
        pub arguments: Vec<Value>,
        pub body: Box<Node>,
    }
    #[derive(Debug, PartialEq, Clone)]
    pub struct Program {
        pub statements: Vec<Box<Node>>,
    }

    // 抽象構文木のノードの型
    #[derive(Debug, PartialEq, Clone)]
    pub enum Node {
        Value(Value),
        Call(Call),
        Expr(Expr),
        Control(Control),
        Function(Function),
        Program(Program),
    }
    impl Node {
        pub fn value(val: Value) -> Box<Self> {
            Box::new(Node::Value(val))
        }

        pub fn expr(op: ExprSymbol, lhs: Box<Node>, rhs: Box<Node>) -> Box<Self> {
            Box::new(Node::Expr(Expr { op, lhs, rhs }))
        }

        pub fn call(callee: Box<Node>, arguments: Vec<Box<Node>>) -> Box<Self> {
            Box::new(Node::Call(Call { callee, arguments }))
        }

        pub fn r#return(val: Box<Node>) -> Box<Self> {
            Box::new(Node::Control(Control::Return(Return { value: val })))
        }

        pub fn r#if(
            cond: Box<Node>,
            then_branch: Box<Node>,
            else_branch: Option<Box<Node>>,
        ) -> Box<Self> {
            Box::new(Node::Control(Control::If(If {
                condition: cond,
                then_branch,
                else_branch,
            })))
        }
        pub fn r#while(cond: Box<Node>, body: Box<Node>) -> Box<Self> {
            Box::new(Node::Control(Control::While(While {
                condition: cond,
                body: body,
            })))
        }

        pub fn r#for(
            init: Option<Box<Node>>,
            cond: Box<Node>,
            update: Option<Box<Node>>,
            body: Box<Node>,
        ) -> Box<Self> {
            Box::new(Node::Control(Control::For(For {
                initializer: init,
                condition: cond,
                updater: update,
                body: body,
            })))
        }
        pub fn function(name: Value, arguments: Vec<Value>, body: Box<Node>) -> Box<Self> {
            Box::new(Node::Function(Function {
                name,
                arguments,
                body,
            }))
        }

        pub fn program(statements: Vec<Box<Node>>) -> Box<Node> {
            Box::new(Node::Program(Program {
                statements: statements,
            }))
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TmpNameGen {
    counter: usize,
}

impl TmpNameGen {
    pub fn new() -> Self {
        TmpNameGen { counter: 0 }
    }

    pub fn next(&mut self) -> String {
        let name = format!("tmp{}", self.counter);
        self.counter += 1;
        name
    }
}

#[derive(Debug, PartialEq)]
pub struct CodeGenStatus {
    pub name_gen: TmpNameGen,
    pub variables: HashMap<String, String>,
}

impl CodeGenStatus {
    pub fn new() -> Self {
        Self {
            name_gen: TmpNameGen::new(),
            variables: HashMap::new(),
        }
    }
}
