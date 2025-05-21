pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Add => write!(f, "+"),
            Op::Sub => write!(f, "-"),
            Op::Mul => write!(f, "*"),
            Op::Div => write!(f, "/"),
        }
    }
}

pub enum Expr {
    Number(i64),
    BinaryOp {
        lhs: Box<Expr>,
        op: Op,
        rhs: Box<Expr>,
    },
}

pub fn num(value: i64) -> Expr {
    Expr::Number(value)
}

impl Expr {
    pub fn add(self, rhs: Expr) -> Expr {
        Expr::BinaryOp {
            lhs: Box::new(self),
            op: Op::Add,
            rhs: Box::new(rhs),
        }
    }

    pub fn sub(self, rhs: Expr) -> Expr {
        Expr::BinaryOp {
            lhs: Box::new(self),
            op: Op::Sub,
            rhs: Box::new(rhs),
        }
    }

    pub fn mul(self, rhs: Expr) -> Expr {
        Expr::BinaryOp {
            lhs: Box::new(self),
            op: Op::Mul,
            rhs: Box::new(rhs),
        }
    }

    pub fn div(self, rhs: Expr) -> Expr {
        Expr::BinaryOp {
            lhs: Box::new(self),
            op: Op::Div,
            rhs: Box::new(rhs),
        }
    }
}

pub struct RPN(Expr);

impl std::fmt::Display for RPN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn fmt(f: &mut std::fmt::Formatter<'_>, expr: &Expr) -> std::fmt::Result {
            match expr {
                Expr::Number(a) => write!(f, "{a}"),
                Expr::BinaryOp { lhs, op, rhs } => match (lhs.as_ref(), rhs.as_ref()) {
                    (Expr::Number(a), Expr::Number(b)) => write!(f, "{a} {b} {op}"),
                    (Expr::Number(a), b @ Expr::BinaryOp { .. }) => {
                        write!(f, "{a} ")?;
                        fmt(f, b)?;
                        write!(f, " {op}")
                    }
                    (a @ Expr::BinaryOp { .. }, Expr::Number(b)) => {
                        fmt(f, a)?;
                        write!(f, " {b} {op}")
                    }
                    (a, b) => {
                        fmt(f, a)?;
                        fmt(f, b)?;
                        write!(f, " {op}")
                    }
                },
            }
        }

        fmt(f, &self.0)
    }
}

#[cfg(test)]
mod test {
    use super::{num, RPN};

    #[test]
    fn test_rpn() {
        for (expr, want) in [
            (num(5), "5"),
            (num(5).add(num(9)), "5 9 +"),
            (num(1).add(num(1)).mul(num(2)), "1 1 + 2 *"),
            (num(2).mul(num(1).add(num(1))), "2 1 1 + *"),
        ] {
            let have = RPN(expr).to_string();
            assert_eq!(want, have);
        }
    }
}
