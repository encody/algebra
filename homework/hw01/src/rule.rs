use crate::in_tree::{Entry, InTree};

use super::model::*;

#[derive(Debug)]
pub struct Resolver {
    pub judgements: Vec<Judgement>,
    pub context: InTree<(Var, Expr)>,
    pub definitions: InTree<DefinitionEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Book(Vec<Judgement>);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Judgement {
    pub definitions: usize,
    pub context: usize,
    pub m: Expr,
    pub n: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefinitionEntry {
    pub context: usize,
    pub name: String,
    pub m: Option<Expr>,
    pub n: Expr,
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            judgements: vec![],
            context: InTree::new(),
            definitions: InTree::new(),
        }
    }

    pub fn sort(&mut self) -> usize {
        self.judgements.push(Judgement {
            definitions: 0,
            context: 0,
            m: Expr::Asterisk,
            n: Expr::Square,
        });
        self.judgements.len() - 1
    }

    pub fn var(&mut self, j: usize, var: Var) -> usize {
        let Judgement {
            definitions,
            context,
            m,
            n,
        } = &self.judgements[j];
        assert!(n.is_sort());
        self.judgements.push(Judgement {
            definitions: *definitions,
            context: self.context.create(*context, (var, m.clone())),
            m: Expr::Var(var),
            n: m.clone(),
        });
        self.judgements.len() - 1
    }

    pub fn weak(&mut self, a: usize, b: usize, var: Var) -> usize {
        let a = &self.judgements[a];
        let b = &self.judgements[b];
        assert_eq!(a.definitions, b.definitions);
        assert_eq!(a.context, b.context);
        assert!(b.n.is_sort());

        self.judgements.push(Judgement {
            definitions: a.definitions,
            context: self.context.create(a.context, (var, b.m.clone())),
            m: a.m.clone(),
            n: a.n.clone(),
        });
        self.judgements.len() - 1
    }

    pub fn form(&mut self, a: usize, b: usize) -> usize {
        let a = &self.judgements[a];
        let b = &self.judgements[b];

        assert_eq!(a.definitions, b.definitions);
        assert!(a.n.is_sort());
        assert!(b.n.is_sort());

        let entry = self.context.get(b.context).unwrap();
        assert_eq!(a.context, entry.parent_index);
        let (ref var, ref ty) = entry.value;
        assert_eq!(ty, &a.m);

        self.judgements.push(Judgement {
            definitions: a.definitions,
            context: a.context,
            m: Expr::Pi(Box::new(Pi(*var, a.m.clone(), b.m.clone()))),
            n: b.n.clone(),
        });
        self.judgements.len() - 1
    }

    pub fn appl(&mut self, e1: usize, e2: usize) -> usize {
        let e1 = &self.judgements[e1];
        let e2 = &self.judgements[e2];

        assert_eq!(e1.definitions, e2.definitions);
        assert_eq!(e1.context, e2.context);

        let m = &e1.m;
        let Expr::Pi(pi) = &e1.n else {
            panic!("expected Pi");
        };
        let x = pi.0;
        let a1 = &pi.1;
        let b = &pi.2;

        let n = &e2.m;
        let a2 = &e2.n;
        // assert_eq!(a1, a2);

        self.judgements.push(Judgement {
            definitions: e1.definitions,
            context: e1.context,
            m: Expr::Application(Box::new(Application(m.clone(), n.clone()))),
            n: b.alpha_substitution(x, n.clone()),
        });
        self.judgements.len() - 1
    }

    pub fn abst(&mut self, e1: usize, e2: usize) -> usize {
        let e1 = &self.judgements[e1];
        let e2 = &self.judgements[e2];

        assert_eq!(e1.definitions, e2.definitions);
        let Entry {
            parent_index: e1_context_parent,
            value: (x1, a1),
            ..
        } = self.context.get(e1.context).unwrap();
        assert_eq!(*e1_context_parent, e2.context);

        let m = &e1.m;
        let b1 = &e1.n;

        let Expr::Pi(pi) = &e2.m else {
            panic!("Expected Pi");
        };
        let x2 = pi.0;
        let a2 = &pi.1;
        let b2 = &pi.2;

        assert_eq!(*x1, x2);
        assert_eq!(a1, a2);
        // assert_eq!(b1, b2);
        assert!(e2.n.is_sort());

        self.judgements.push(Judgement {
            definitions: e1.definitions,
            context: e2.context,
            m: Lambda(*x1, a1.clone(), m.clone()).into(),
            n: Pi(*x1, a2.clone(), b1.clone()).into(),
        });
        self.judgements.len() - 1
    }

    pub fn conv(&mut self, e1: usize, e2: usize) -> usize {
        let e1 = &self.judgements[e1];
        let e2 = &self.judgements[e2];

        assert_eq!(e1.definitions, e2.definitions);
        assert_eq!(e1.context, e2.context);
        let a = &e1.m;
        // let b1 = e1.n;

        let b2 = &e2.m;
        let s = &e2.n;
        assert!(s.is_sort());

        self.judgements.push(Judgement {
            definitions: e1.definitions,
            context: e1.context,
            m: a.clone(),
            n: b2.clone(),
        });
        self.judgements.len() - 1
    }

    pub fn def(&mut self, e1: usize, e2: usize, name: String) -> usize {
        let e1 = &self.judgements[e1];
        let e2 = &self.judgements[e2];

        assert_eq!(e1.definitions, e2.definitions);
        let context = e1.context;

        let k = &e1.m;
        let l = &e1.n;
        let m = &e2.m;
        let n = &e2.n;

        assert_eq!(
            self.definitions.resolve(e1.definitions, |d| d.name == name),
            None,
        );

        self.judgements.push(Judgement {
            definitions: self.definitions.create(
                e1.definitions,
                DefinitionEntry {
                    context: e2.context,
                    name,
                    m: Some(m.clone()),
                    n: n.clone(),
                },
            ),
            context,
            m: k.clone(),
            n: l.clone(),
        });
        self.judgements.len() - 1
    }

    pub fn def_prim(&mut self, e1: usize, e2: usize, name: String) -> usize {
        let e1 = &self.judgements[e1];
        let e2 = &self.judgements[e2];

        assert_eq!(e1.definitions, e2.definitions);

        let k = &e1.m;
        let l = &e1.n;
        let n = &e2.m;
        let s = &e2.n;

        assert!(s.is_sort());
        assert_eq!(
            self.definitions.resolve(e1.definitions, |d| d.name == name),
            None,
        );

        self.judgements.push(Judgement {
            definitions: self.definitions.create(
                e1.definitions,
                DefinitionEntry {
                    context: e2.context,
                    name,
                    m: None,
                    n: n.clone(),
                },
            ),
            context: e1.context,
            m: k.clone(),
            n: l.clone(),
        });
        self.judgements.len() - 1
    }

    pub fn inst_ix(&mut self, e1: usize, e2: &[usize], d: usize) -> usize {
        let e1 = &self.judgements[e1];

        let entry = self.definitions.traverse(e1.definitions, d).unwrap();
        let d = &entry.value;

        assert_eq!(self.context.len(d.context), e2.len());

        let mut n = d.n.clone();

        let mut values = vec![];

        let mut c = d.context;

        for e in e2.iter().rev() {
            let e = &self.judgements[*e];
            let Entry {
                parent_index,
                value: (v, a),
                ..
            } = self.context.get(c).unwrap();
            c = *parent_index;

            assert_eq!(e1.definitions, e.definitions);
            assert_eq!(e1.context, e.context);

            let u = &e.m;
            let a_substituted = &e.n;
            // assert_eq!(
            //     a.alpha_substitution(*v, u.clone()).de_bruijn(),
            //     a_substituted.de_bruijn()
            // );

            n = n.alpha_substitution(*v, u.clone());

            values.push(u.clone());
        }

        values.reverse();

        assert_eq!(e1.m, Expr::Asterisk);
        assert_eq!(e1.n, Expr::Square);

        self.judgements.push(Judgement {
            definitions: e1.definitions,
            context: e1.context,
            m: Expr::Definition(crate::model::Definition(d.name.clone(), values)),
            n,
        });
        self.judgements.len() - 1
    }

    pub fn inst(&mut self, e1: usize, e2: &[usize], name: String) -> usize {
        let e1 = &self.judgements[e1];

        let d = self
            .definitions
            .resolve(e1.definitions, |d| d.name == name)
            .unwrap();

        assert_eq!(self.context.get(d.context).unwrap().len, e2.len());

        let mut n = d.n.clone();

        let mut values = vec![];

        let mut c = d.context;

        for e in e2.iter().rev() {
            let e = &self.judgements[*e];
            let Entry {
                parent_index,
                value: (v, a),
                ..
            } = self.context.get(c).unwrap();
            c = *parent_index;

            assert_eq!(e1.definitions, e.definitions);
            assert_eq!(e1.context, e.context);

            let u = &e.m;
            let a_substituted = &e.n;
            // assert_eq!(
            //     a.alpha_substitution(*v, u.clone()).de_bruijn(),
            //     a_substituted.de_bruijn()
            // );

            n = n.alpha_substitution(*v, u.clone());

            values.push(u.clone());
        }

        values.reverse();

        assert_eq!(e1.m, Expr::Asterisk);
        assert_eq!(e1.n, Expr::Square);

        self.judgements.push(Judgement {
            definitions: e1.definitions,
            context: e1.context,
            m: Expr::Definition(crate::model::Definition(name, values)),
            n,
        });
        self.judgements.len() - 1
    }

    pub fn cp(&mut self, j: usize) -> usize {
        let j = &self.judgements[j];
        self.judgements.push(j.clone());
        self.judgements.len() - 1
    }

    pub fn sp(&mut self, j: usize, ix: usize) -> usize {
        let j = &self.judgements[j];

        let context = self.context.traverse(j.context, ix).unwrap();

        let (m, n) = &context.value;

        self.judgements.push(Judgement {
            definitions: j.definitions,
            context: j.context,
            m: (*m).into(),
            n: n.clone(),
        });
        self.judgements.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::vec_init_then_push)]
    #[test]
    fn homework() {
        let mut book = Resolver::new();

        book.sort();
        book.var(0, Var('A'));
        book.weak(0, 0, Var('A'));
        book.var(2, Var('B'));
        book.weak(2, 2, Var('B'));
        book.weak(1, 2, Var('B'));
        book.var(5, Var('a'));
        book.weak(4, 5, Var('a'));
        book.weak(3, 5, Var('a'));
        book.form(5, 8);

        eprintln!("{book:?}");
    }
}
