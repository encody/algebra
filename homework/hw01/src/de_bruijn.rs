#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Var(Var),
    Asterisk,
    Square,
    Lambda(Box<Lambda>),
    Pi(Box<Pi>),
    Definition(Definition),
    Application(Box<Application>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Var {
    Free(crate::model::Var),
    Bound(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lambda(pub Expr, pub Expr);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pi(pub Expr, pub Expr);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Definition(pub Box<[char]>, pub Vec<Expr>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Application(pub Expr, pub Expr);

#[derive(Debug, Clone, Default)]
pub enum BindingStack<'a> {
    #[default]
    Empty,
    Binding {
        var: crate::model::Var,
        previous: &'a BindingStack<'a>,
    },
}

impl<'a> BindingStack<'a> {
    fn with<'b>(&'b self, new: crate::model::Var) -> BindingStack<'b> {
        BindingStack::Binding {
            var: new,
            previous: self,
        }
    }

    fn index(&self, search: &crate::model::Var) -> Option<usize> {
        let mut stack = self;
        let mut index = 1usize;
        while let Self::Binding { var, previous } = stack {
            if var == search {
                return Some(index);
            }
            stack = previous;
            index += 1;
        }

        None
    }

    fn get_binding(&self, search: crate::model::Var) -> Var {
        match self.index(&search) {
            Some(index) => Var::Bound(index),
            None => Var::Free(search),
        }
    }
}

pub fn de_bruijn(e: &crate::model::Expr, bindings: &BindingStack<'_>) -> Expr {
    match e {
        crate::model::Expr::Var(var) => Expr::Var(bindings.get_binding(*var)),
        crate::model::Expr::Asterisk => Expr::Asterisk,
        crate::model::Expr::Square => Expr::Square,
        crate::model::Expr::Lambda(lambda) => {
            let crate::model::Lambda(var, m, n) = &**lambda;
            Expr::Lambda(Box::new(Lambda(
                de_bruijn(m, bindings),
                de_bruijn(n, &bindings.with(*var)),
            )))
        }
        crate::model::Expr::Pi(pi) => {
            let crate::model::Pi(var, m, n) = &**pi;
            Expr::Pi(Box::new(Pi(
                de_bruijn(m, bindings),
                de_bruijn(n, &bindings.with(*var)),
            )))
        }
        crate::model::Expr::Definition(crate::model::Definition(a, exprs)) => {
            Expr::Definition(Definition(
                a.clone(),
                exprs.iter().map(|e| de_bruijn(e, bindings)).collect(),
            ))
        }
        crate::model::Expr::Application(application) => {
            let crate::model::Application(f, a) = &**application;
            Expr::Application(Box::new(Application(
                de_bruijn(f, bindings),
                de_bruijn(a, bindings),
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    #[rstest]
    #[test]
    #[case(
        "%($x:(*).(%(x)($z:(*).(%(x)(y)))))(z)",
        "%($x:(*).(%(x)($z:(*).(%(x)(y)))))(z)"
    )]
    #[case(
        "%($x:(*).(%(x)($z:(*).(%(x)(y)))))(z)",
        "%($u:(*).(%(u)($z:(*).(%(u)(y)))))(z)"
    )]
    #[case(
        "%($x:(*).(%(x)($z:(*).(%(x)(y)))))(z)",
        "%($z:(*).(%(z)($x:(*).(%(z)(y)))))(z)"
    )]
    #[case("$x:(*).($y:(*).(%(%(x)(z))(y)))", "$v:(*).($y:(*).(%(%(v)(z))(y)))")]
    #[case("$x:(*).($y:(*).(%(%(x)(z))(y)))", "$v:(*).($u:(*).(%(%(v)(z))(u)))")]
    fn alpha_equivalence_true(#[case] a: crate::model::Expr, #[case] b: crate::model::Expr) {
        assert_eq!(a.de_bruijn(), b.de_bruijn());
    }

    #[rstest]
    #[test]
    #[case(
        "%($x:(*).(%(x)($z:(*).(%(x)(y)))))(z)",
        "%($y:(*).(%(y)($z:(*).(%(y)(y)))))(z)"
    )]
    #[case(
        "%($x:(*).(%(x)($z:(*).(%(x)(y)))))(z)",
        "%($z:(*).(%(z)($z:(*).(%(z)(y)))))(z)"
    )]
    #[case(
        "%($x:(*).(%(x)($z:(*).(%(x)(y)))))(z)",
        "%($u:(*).(%(u)($z:(*).(%(u)(y)))))(v)"
    )]
    #[case("$x:(*).($y:(*).(%(%(x)(z))(y)))", "$y:(*).($y:(*).(%(%(y)(z))(y)))")]
    #[case("$x:(*).($y:(*).(%(%(x)(z))(y)))", "$z:(*).($y:(*).(%(%(z)(z))(y)))")]
    fn alpha_equivalence_false(#[case] a: crate::model::Expr, #[case] b: crate::model::Expr) {
        assert_ne!(a.de_bruijn(), b.de_bruijn());
    }

    #[rstest]
    #[test]
    fn homework_cases() {
        let a: crate::model::Expr = "$x:(*).($x:(x).(x))".parse().unwrap();
        let b: crate::model::Expr = "$x:(*).($x:(y).(x))".parse().unwrap();
        let c: crate::model::Expr = "$x:(*).($y:(x).(y))".parse().unwrap();
        if a.de_bruijn() == b.de_bruijn() {
            eprintln!("a & b");
        }
        if a.de_bruijn() == c.de_bruijn() {
            eprintln!("a & c");
        }
        if b.de_bruijn() == c.de_bruijn() {
            eprintln!("b & c");
        }
    }
}
