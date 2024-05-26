use std::rc::Rc;
use std::process;

pub enum AST {
    Int(i32),
    Float(f64),
    Variable(String),
    BinaryOp(BinaryOpAst),
}

pub enum CBinOps {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Xor,
    LAnd,
    LOr,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Shl,
    Shr,
}

pub struct BinaryOpAst {
    pub lhs: Rc<AST>,
    pub rhs: Rc<AST>,
    pub op: CBinOps,
}

impl BinaryOpAst {
    pub fn new(lhs: Rc<AST>, rhs: Rc<AST>, op: String) -> BinaryOpAst {
        let cop = match op.as_str() {
            "+" => CBinOps::Add,
            "-" => CBinOps::Sub,
            "*" => CBinOps::Mul,
            "/" => CBinOps::Div,
            "%" => CBinOps::Rem,
            "&" => CBinOps::And,
            "|" => CBinOps::Or,
            "^" => CBinOps::Xor,
            "&&" => CBinOps::LAnd,
            "||" => CBinOps::LOr,
            "==" => CBinOps::Eq,
            "!=" => CBinOps::Ne,
            "<" => CBinOps::Lt,
            ">" => CBinOps::Gt,
            "<=" => CBinOps::Le,
            ">=" => CBinOps::Ge,
            "<<" => CBinOps::Shl,
            ">>" => CBinOps::Shr,
            _ => {
                eprintln!("Unknown operator: {}", op);
                process::exit(1);
            }
        };

        BinaryOpAst {
            lhs,
            rhs,
            op: cop,
        }
    }

    pub fn eval_constexpr(&self) -> i32 {
        let lhs = self.lhs.eval_constexpr();
        let rhs = self.rhs.eval_constexpr();
        match self.op {
            CBinOps::Add => lhs + rhs,
            CBinOps::Sub => lhs - rhs,
            CBinOps::Mul => lhs * rhs,
            CBinOps::Div => lhs / rhs,
            CBinOps::Rem => lhs % rhs,
            CBinOps::And => lhs & rhs,
            CBinOps::Or => lhs | rhs,
            CBinOps::Xor => lhs ^ rhs,
            CBinOps::LAnd => (lhs != 0 && rhs != 0) as i32,
            CBinOps::LOr => (lhs != 0 || rhs != 0) as i32,
            CBinOps::Eq => (lhs == rhs) as i32,
            CBinOps::Ne => (lhs != rhs) as i32,
            CBinOps::Lt => (lhs < rhs) as i32,
            CBinOps::Gt => (lhs > rhs) as i32,
            CBinOps::Le => (lhs <= rhs) as i32,
            CBinOps::Ge => (lhs >= rhs) as i32,
            CBinOps::Shl => lhs << rhs,
            CBinOps::Shr => lhs >> rhs,
        }
    }
}

impl AST {
    pub fn eval_constexpr(&self) -> i32 {
        match self {
            AST::Int(n) => *n,
            AST::BinaryOp(ref bin) => bin.eval_constexpr(),
            _ => panic!("Invalid AST node for constant expression evaluation"),
        }
    }
}

