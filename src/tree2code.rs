use crate::setting::{
    node::{Call, Control, Expr, Function, Node, Program},
    token::{Arithmetic, Comparison, ExprSymbol, Value},
    *,
};

trait ToLLVMIR {
    fn to_llvmir(&self) -> &str;
}

const EVIL: &str = "evil";
const IGNORE: &str = "ignore";

impl ToLLVMIR for Arithmetic {
    fn to_llvmir(&self) -> &str {
        match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Div => "sdiv",
        }
    }
}
impl ToLLVMIR for Comparison {
    fn to_llvmir(&self) -> &str {
        match self {
            Self::Eq => "icmp eq",
            Self::Neq => "icmp ne",
            Self::Lt => "icmp slt",
            Self::Le => "icmp sle",
            Self::Gt => "icmp sgt",
            Self::Ge => "icmp sge",
        }
    }
}

fn i1toi32(name_i1: String, cgs: &mut CodeGenStatus) -> String {
    let name = cgs.name_gen.next();
    println!("%{} = zext i1 %{} to i32", name, name_i1);
    return name;
}

fn i32toi1(name_i32: String, cgs: &mut CodeGenStatus) -> String {
    let name = cgs.name_gen.next();
    println!("%{} = icmp ne i32 %{}, 0", name, name_i32);
    return name;
}

fn gen_expr(expr: Expr, cgs: &mut CodeGenStatus) -> String {
    match expr.op {
        ExprSymbol::Arithmetic(ari) => {
            let lhs = generate(expr.lhs, cgs);
            let rhs = generate(expr.rhs, cgs);
            let name = cgs.name_gen.next();

            println!("%{} = {} i32 %{}, %{}", name, ari.to_llvmir(), lhs, rhs);
            return name;
        }
        ExprSymbol::Comparison(com) => {
            let lhs = generate(expr.lhs, cgs);
            let rhs = generate(expr.rhs, cgs);
            let name = cgs.name_gen.next();

            println!("%{} = {} i32 %{}, %{}", name, com.to_llvmir(), lhs, rhs);
            i1toi32(name, cgs)
        }
        ExprSymbol::Assignment => {
            // lhs は ident なので、もう一度解析する必要あり
            if let Node::Value(Value::Ident(ref idn)) = *expr.lhs {
                let rhs = generate(expr.rhs, cgs);
                let ptr = cgs.variables.entry(idn.clone()).or_insert_with(|| {
                    let alloc = cgs.name_gen.next();
                    println!("%{} = alloca i32", alloc);
                    alloc
                });
                println!("store i32 %{}, i32* %{}", rhs, ptr);
                return ptr.clone();
            } else {
                panic!("The left side is not variable!");
            }
        }
        _ => panic!(),
    }
}

fn gen_control(control: Control, cgs: &mut CodeGenStatus) -> String {
    match control {
        node::Control::Return(be) => {
            let lhs = generate(be.value, cgs);
            println!("ret i32 %{}", lhs);
            return EVIL.to_string();
        }
        node::Control::If(be) => {
            let con = i32toi1(generate(be.condition, cgs), cgs);
            let if_name = cgs.name_gen.next();

            println!(
                "br i1 %{}, label %if{}_true, label %if{}_false",
                con, if_name, if_name
            );
            println!("if{}_true:", if_name);

            let if_result = generate(be.then_branch, cgs);
            if if_result != EVIL {
                println!("br label %if{}_end", if_name);
            }

            println!("if{}_false:", if_name);

            let else_result = be.else_branch.map(|b| generate(b, cgs)).unwrap_or_else(|| {
                println!("br label %if{}_end", if_name);
                EVIL.to_string()
            });

            if else_result != EVIL {
                println!("br label %if{}_end", if_name);
            }

            if if_result != EVIL || else_result != EVIL {
                println!("if{}_end:", if_name);
            }

            IGNORE.to_string()
        }
        node::Control::While(be) => {
            let while_name = cgs.name_gen.next();

            println!("br label %begin{}", while_name);
            println!("begin{}:", while_name);

            let con = i32toi1(generate(be.condition, cgs), cgs);

            println!(
                "br i1 %{}, label %while_true{}, label %end{}",
                con, while_name, while_name
            );

            println!("while_true{}:", while_name);

            generate(be.body, cgs);

            println!("br label %begin{}", while_name);
            println!("end{}:", while_name);

            IGNORE.to_string()
        }
        node::Control::For(be) => {
            let for_name = cgs.name_gen.next();

            be.initializer
                .map(|x| generate(x, cgs))
                .unwrap_or(IGNORE.to_string());

            println!("br label %begin{}", for_name);
            println!("begin{}:", for_name);

            let con = i32toi1(generate(be.condition, cgs), cgs);

            println!(
                "br i1 %{}, label %for_true{}, label %end{}",
                con, for_name, for_name
            );

            println!("for_true{}:", for_name);

            be.updater
                .map(|x| generate(x, cgs))
                .unwrap_or(IGNORE.to_string());
            generate(be.body, cgs);

            println!("br label %begin{}", for_name);
            println!("end{}:", for_name);

            IGNORE.to_string()
        }
    }
}

fn gen_value(value: Value, cgs: &mut CodeGenStatus) -> String {
    match value {
        Value::Number(num) => {
            let name1 = cgs.name_gen.next();
            println!("%{} = add i32 0, {}", name1, num);
            return name1;
        }
        Value::Ident(idn) => {
            if let Some(ptr) = cgs.variables.get(&idn) {
                // 既にallcoされた変数
                let tmp = cgs.name_gen.next();
                println!("%{} = load i32, i32* %{}", tmp, ptr);
                return tmp;
            } else {
                // 初めて出てきた変数
                let ptr = cgs.name_gen.next();
                println!("%{} = alloca i32", ptr);
                cgs.variables.insert(idn.clone(), ptr.clone());
                return ptr;
            }
        }
    }
}

fn gen_program(program: Program, cgs: &mut CodeGenStatus) -> String {
    for statement in program.statements {
        generate(statement, cgs);
    }
    IGNORE.to_string()
}

fn gen_call(call: Call, cgs: &mut CodeGenStatus) -> String {
    let name = cgs.name_gen.next();
    let args: Vec<String> = call
        .arguments
        .iter()
        .map(|arg| format!("i32 noundef %{}", generate(arg.clone(), cgs)))
        .collect();

    let fn_name = match *call.callee {
        Node::Value(Value::Ident(ref idn)) => idn.clone(),
        _ => panic!("The callee is not a function!"),
    };
    println!("%{} = call i32 @{}({})", name, fn_name, args.join(", "));
    return name;
}

fn gen_function(function: Function, cgs: &mut CodeGenStatus) -> String {
    let get_name = |x: &Value| match x {
        Value::Ident(idn) => idn.clone(),
        _ => panic!("The callee is not a function!"),
    };

    let name = get_name(&function.name);
    let args: Vec<String> = function
        .arguments
        .iter()
        .map(|_| cgs.name_gen.next())
        .collect();

    println!(
        "define i32 @{}({}) {{",
        name,
        args.iter()
            .map(|x| format!("i32 noundef %{}", x))
            .collect::<Vec<_>>()
            .join(", ")
    );

    (0..args.len()).for_each(|i| {
        let ptr = cgs.name_gen.next();
        println!("%{} = alloca i32", ptr);
        println!("store i32 %{}, i32* %{}", args[i], ptr);

        cgs.variables
            .insert(get_name(&function.arguments[i]), ptr.clone());
    });
    generate(function.body, cgs);
    println!("}}");

    //名前空間のクリア
    cgs.variables.clear();

    return IGNORE.to_string();
}

pub fn generate(node: Box<Node>, cgs: &mut CodeGenStatus) -> String {
    match *node {
        Node::Expr(expr) => gen_expr(expr, cgs),
        Node::Control(control) => gen_control(control, cgs),
        Node::Value(value) => gen_value(value, cgs),
        Node::Program(program) => gen_program(program, cgs),
        Node::Call(call) => gen_call(call, cgs),
        Node::Function(function) => gen_function(function, cgs),
    }
}

#[test]
fn test() {
    use crate::string2tree::program;
    use crate::tokenize::tokenize;
    let a = "return a(1,2);";
    let mut b = tokenize(&a.to_string());
    let ast = program(&mut b);
    println!("{:#?}", ast);
    let mut cgs = CodeGenStatus::new();
    generate(ast, &mut cgs);
}
