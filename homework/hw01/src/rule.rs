use super::model::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context(Vec<(Var, Expr)>);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Book(Vec<Judgement>);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Judgement {
    pub definitions: Vec<Definition>,
    pub context: Context,
    pub m: Expr,
    pub n: Expr,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definition {
    pub context: Context,
    pub name: String,
    pub m: Option<Expr>,
    pub n: Expr,
}

pub fn sort() -> Judgement {
    Judgement {
        definitions: vec![],
        context: Context(vec![]),
        m: Expr::Asterisk,
        n: Expr::Square,
    }
}

pub fn var(
    Judgement {
        definitions,
        mut context,
        m,
        n,
    }: Judgement,
    var: Var,
) -> Judgement {
    assert!(n.is_sort());
    context.0.push((var, m.clone()));
    Judgement {
        definitions,
        context,
        m: Expr::Var(var),
        n: m,
    }
}

pub fn weak(a: Judgement, b: Judgement, var: Var) -> Judgement {
    assert_eq!(a.definitions, b.definitions);
    assert_eq!(a.context, b.context);
    assert!(b.n.is_sort());

    let mut context = a.context;
    context.0.push((var, b.m));

    Judgement {
        definitions: a.definitions,
        context,
        m: a.m,
        n: a.n,
    }
}

pub fn form(a: Judgement, b: Judgement) -> Judgement {
    assert_eq!(a.definitions, b.definitions);
    assert!(a.n.is_sort());
    assert!(b.n.is_sort());

    let mut context = b.context;
    let (var, ty) = context.0.pop().unwrap();
    assert_eq!(a.context, context);
    assert_eq!(ty, a.m);

    Judgement {
        definitions: a.definitions,
        context: a.context,
        m: Expr::Pi(Box::new(Pi(var, a.m, b.m))),
        n: b.n,
    }
}

pub fn appl(e1: Judgement, e2: Judgement) -> Judgement {
    assert_eq!(e1.definitions, e2.definitions);
    assert_eq!(e1.context, e2.context);

    let m = e1.m;
    let Expr::Pi(pi) = e1.n else {
        panic!("expected Pi");
    };
    let Pi(x, a1, b) = *pi;

    let n = e2.m;
    let a2 = e2.n;
    assert_eq!(a1, a2);

    Judgement {
        definitions: e1.definitions,
        context: e1.context,
        m: Expr::Application(Box::new(Application(m, n.clone()))),
        n: b.alpha_substitution(x, n),
    }
}

pub fn abst(mut e1: Judgement, e2: Judgement) -> Judgement {
    assert_eq!(e1.definitions, e2.definitions);
    let (x1, a1) = e1.context.0.pop().unwrap();
    assert_eq!(e1.context, e2.context);

    let m = e1.m;
    let b1 = e1.n;

    let Expr::Pi(pi) = e2.m else {
        panic!("Expected Pi");
    };
    let Pi(x2, a2, b2) = *pi;

    assert_eq!(x1, x2);
    assert_eq!(a1, a2);
    assert_eq!(b1, b2);
    assert!(e2.n.is_sort());

    Judgement {
        definitions: e1.definitions,
        context: e1.context,
        m: Lambda(x1, a1, m).into(),
        n: Pi(x1, a2, b1).into(),
    }
}

pub fn conv(e1: Judgement, e2: Judgement) -> Judgement {
    assert_eq!(e1.definitions, e2.definitions);
    assert_eq!(e1.context, e2.context);
    let a = e1.m;
    // let b1 = e1.n;

    let b2 = e2.m;
    let s = e2.n;
    assert!(s.is_sort());

    Judgement {
        definitions: e1.definitions,
        context: e1.context,
        m: a,
        n: b2,
    }
}

pub fn def(e1: Judgement, e2: Judgement, name: String) -> Judgement {
    assert_eq!(e1.definitions, e2.definitions);
    let context = e1.context;

    let k = e1.m;
    let l = e1.n;
    let m = e2.m;
    let n = e2.n;

    let mut definitions = e1.definitions;
    assert_eq!(definitions.iter().find(|d| d.name == name), None);
    definitions.push(Definition {
        context: e2.context,
        name,
        m: Some(m),
        n,
    });

    Judgement {
        definitions,
        context,
        m: k,
        n: l,
    }
}

pub fn def_prim(e1: Judgement, e2: Judgement, name: String) -> Judgement {
    assert_eq!(e1.definitions, e2.definitions);

    let k = e1.m;
    let l = e1.n;
    let n = e2.m;
    let s = e2.n;

    assert!(s.is_sort());
    let mut definitions = e1.definitions;
    assert_eq!(definitions.iter().find(|d| d.name == name), None);
    definitions.push(Definition {
        context: e2.context,
        name,
        m: None,
        n,
    });

    Judgement {
        definitions,
        context: e1.context,
        m: k,
        n: l,
    }
}

pub fn inst(e1: Judgement, e2: &[Judgement], name: String) -> Judgement {
    let d = e1.definitions.iter().find(|d| d.name == name).unwrap();

    assert_eq!(d.context.0.len(), e2.len());

    let mut n = d.n.clone();

    let mut values = vec![];

    for (e, (v, a)) in e2.iter().zip(d.context.0.iter()) {
        assert_eq!(e1.definitions, e.definitions);
        assert_eq!(e1.context, e.context);

        let u = &e.m;
        let a_substituted = &e.n;
        assert_eq!(&a.alpha_substitution(*v, u.clone()), a_substituted);

        n = n.alpha_substitution(*v, u.clone());

        values.push(u.clone());
    }

    assert_eq!(e1.m, Expr::Asterisk);
    assert_eq!(e1.n, Expr::Square);

    Judgement {
        definitions: e1.definitions,
        context: e1.context,
        m: Expr::Definition(crate::model::Definition(name, values)),
        n,
    }
}

pub fn cp(j: Judgement) -> Judgement {
    j
}

pub fn sp(j: Judgement, ix: usize) -> Judgement {
    let m: Expr = j.context.0[ix].0.into();
    let n: Expr = j.context.0[ix].1.clone();
    Judgement {
        definitions: j.definitions,
        context: j.context,
        m,
        n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::vec_init_then_push)]
    #[test]
    fn homework() {
        let mut book = vec![];
        book.push(sort());
        book.push(var(book[0].clone(), Var('A')));
        book.push(weak(book[0].clone(), book[0].clone(), Var('A')));
        book.push(var(book[2].clone(), Var('B')));
        book.push(weak(book[2].clone(), book[2].clone(), Var('B')));
        book.push(weak(book[1].clone(), book[2].clone(), Var('B')));
        book.push(var(book[5].clone(), Var('a')));
        book.push(weak(book[4].clone(), book[5].clone(), Var('a')));
        book.push(weak(book[3].clone(), book[5].clone(), Var('a')));
        book.push(form(book[5].clone(), book[8].clone()));

        eprintln!("{book:?}");
    }
}
