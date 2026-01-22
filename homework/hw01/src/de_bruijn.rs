use std::collections::HashSet;

use crate::model::generate_free_var_gte;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Var(Var),
    Asterisk,
    Square,
    Lambda(Box<Lambda>),
    Pi(Box<Pi>),
    Definition(Definition),
    Application(Box<Application>),
}

impl From<Expr> for crate::model::Expr {
    fn from(value: Expr) -> Self {
        match value {
            Expr::Var(Var::Bound(_, v) | Var::Free(v)) => crate::model::Expr::Var(v),
            Expr::Asterisk => crate::model::Expr::Asterisk,
            Expr::Square => crate::model::Expr::Square,
            Expr::Lambda(l) => crate::model::Lambda(l.0, l.1.into(), l.2.into()).into(),
            Expr::Pi(pi) => crate::model::Pi(pi.0, pi.1.into(), pi.2.into()).into(),
            Expr::Definition(d) => {
                crate::model::Definition(d.0, d.1.into_iter().map(Into::into).collect()).into()
            }
            Expr::Application(a) => crate::model::Application(a.0.into(), a.1.into()).into(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Var {
    Free(crate::model::Var),
    Bound(usize, crate::model::Var),
}

impl PartialEq for Var {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Free(v1), Self::Free(v2)) => v1 == v2,
            (Self::Bound(i1, _), Self::Bound(i2, _)) => i1 == i2,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Lambda(pub crate::model::Var, pub Expr, pub Expr);

impl PartialEq for Lambda {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1 && self.2 == other.2
    }
}

#[derive(Clone, Debug)]
pub struct Pi(pub crate::model::Var, pub Expr, pub Expr);

impl PartialEq for Pi {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1 && self.2 == other.2
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Definition(pub String, pub Vec<Expr>);

#[derive(Clone, Debug, PartialEq)]
pub struct Application(pub Expr, pub Expr);

#[derive(Clone, Debug)]
pub struct Bindings {
    stack: Vec<Binding>,
    substitution: Option<(crate::model::Var, crate::model::Expr)>,
}

#[derive(Clone, Debug)]
struct Binding {
    original: crate::model::Var,
    rename: crate::model::Var,
}

impl Bindings {
    pub fn new(substitution: Option<(crate::model::Var, crate::model::Expr)>) -> Self {
        Self {
            stack: vec![],
            substitution,
        }
    }

    pub fn substitution_free_vars(&self) -> HashSet<crate::model::Var> {
        self.substitution
            .as_ref()
            .map_or(HashSet::new(), |(_, body)| body.free_vars())
    }

    fn with(&self, original: crate::model::Var, rename: crate::model::Var) -> Self {
        let mut s = self.clone();
        s.stack.push(Binding { original, rename });
        s
    }

    fn index(&self, search: &crate::model::Var) -> IndexResult {
        if let Some((ix, v)) = self
            .stack
            .iter()
            .rev()
            .enumerate()
            .find(|(_, v)| &v.original == search)
        {
            return IndexResult::Index {
                index: ix + 1,
                rename: v.rename,
            };
        }

        if let Some((sub_v, sub_e)) = &self.substitution
            && sub_v == search
        {
            return IndexResult::Substitution(sub_e.clone());
        }

        IndexResult::Free
    }

    fn get_binding_or_substitution(&self, search: crate::model::Var) -> Expr {
        match self.index(&search) {
            IndexResult::Index { index, rename } => Expr::Var(Var::Bound(index, rename)),
            IndexResult::Free => Expr::Var(Var::Free(search)),
            IndexResult::Substitution(e) => e.de_bruijn(),
        }
    }
}

enum IndexResult {
    Free,
    Index {
        index: usize,
        rename: crate::model::Var,
    },
    Substitution(crate::model::Expr),
}

pub fn de_bruijn(e: &crate::model::Expr, bindings: &Bindings) -> Expr {
    match e {
        crate::model::Expr::Var(var) => bindings.get_binding_or_substitution(*var),
        crate::model::Expr::Asterisk => Expr::Asterisk,
        crate::model::Expr::Square => Expr::Square,
        crate::model::Expr::Lambda(lambda) => {
            let crate::model::Lambda(var, m, n) = &**lambda;
            let mut fv = bindings.substitution_free_vars();
            fv.extend(n.free_vars());
            let rename = generate_free_var_gte(&fv, *var);
            Expr::Lambda(Box::new(Lambda(
                rename,
                de_bruijn(m, bindings),
                de_bruijn(n, &bindings.with(*var, rename)),
            )))
        }
        crate::model::Expr::Pi(pi) => {
            let crate::model::Pi(var, m, n) = &**pi;
            let mut fv = bindings.substitution_free_vars();
            fv.extend(n.free_vars());
            let rename = generate_free_var_gte(&fv, *var);
            Expr::Pi(Box::new(Pi(
                rename,
                de_bruijn(m, bindings),
                de_bruijn(n, &bindings.with(*var, rename)),
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
    #[case("x", "x", "z", "z")]
    #[case("%(x)(x)", "x", "z", "%(z)(z)")]
    #[case("%($x:(*).(x))(x)", "x", "z", "%($x:(*).(x))(z)")]
    #[case("$y:(*).(%(y)(x))", "x", "%(x)(y)", "$z:(*).(%(z)(%(x)(y)))")]
    #[case("$x:(*).(%(y)(x))", "x", "%(x)(y)", "$z:(*).(%(y)(z))")]
    #[case(
        "$x:(*).($y:(*).(%(z)(%(z)(x))))",
        "z",
        "y",
        "$x:(*).($v:(*).(%(y)(%(y)(x))))"
    )]
    fn substitution(
        #[case] e: crate::model::Expr,
        #[case] v: crate::model::Var,
        #[case] sub: crate::model::Expr,
        #[case] expected: crate::model::Expr,
    ) {
        assert_eq!(
            e.alpha_substitution(v, sub).de_bruijn(),
            expected.de_bruijn(),
        );
    }

    #[rstest]
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
