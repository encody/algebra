use crate::{model::Var, rule::Resolver};

#[derive(Debug, Default)]
pub struct Verifier {
    resolver: Resolver,
}

struct Tokenizer<T>(T);

impl<'a, T: Iterator<Item = &'a str>> Tokenizer<T> {
    pub fn line_number(&mut self) -> usize {
        self.0.next().expect("line number").parse().unwrap()
    }

    pub fn judgement(&mut self) -> usize {
        self.0.next().expect("judgement index").parse().unwrap()
    }

    pub fn take_usize(&mut self, reason: &str) -> usize {
        self.0.next().expect(reason).parse().unwrap()
    }

    pub fn instruction(&mut self) -> &'a str {
        self.0.next().expect("instruction")
    }

    pub fn variable(&mut self) -> Var {
        Var(self.0.next().expect("variable name").parse().unwrap())
    }

    pub fn constant(&mut self) -> String {
        self.0.next().expect("constant name").to_string()
    }

    pub fn end(mut self) {
        assert_eq!(self.0.next(), None);
    }
}

impl Verifier {
    pub fn new() -> Self {
        Self {
            resolver: Resolver::new(),
        }
    }

    pub fn run(input: &str) {
        let mut v = Self::new();

        for line in input.lines() {
            if line == "-1" {
                break;
            }
            v.run_line(line);
        }
    }

    pub fn run_line(&mut self, line: &str) {
        eprint!("verifying `{line}`...");
        let mut t = Tokenizer(line.split(' '));
        let lineno = t.line_number();

        assert_eq!(lineno, self.resolver.judgements.len(), "wrong line number");

        let op_name = t.instruction();

        match op_name {
            "sort" => self.resolver.sort(),
            "var" => {
                let j = t.judgement();

                let var = t.variable();

                self.resolver.var(j, var)
            }
            "weak" => {
                let a = t.judgement();
                let b = t.judgement();
                let var = t.variable();

                self.resolver.weak(a, b, var)
            }
            "form" => {
                let a = t.judgement();
                let b = t.judgement();

                self.resolver.form(a, b)
            }
            "appl" => {
                let a = t.judgement();
                let b = t.judgement();

                self.resolver.appl(a, b)
            }
            "abst" => {
                let a = t.judgement();
                let b = t.judgement();

                self.resolver.abst(a, b)
            }
            "conv" => {
                let a = t.judgement();
                let b = t.judgement();

                self.resolver.conv(a, b)
            }
            "def" => {
                let a = t.judgement();
                let b = t.judgement();

                let name = t.constant();

                self.resolver.def(a, b, name)
            }
            "defpr" => {
                let a = t.judgement();
                let b = t.judgement();

                let name = t.constant();

                self.resolver.def_prim(a, b, name)
            }
            "inst" => {
                let m = t.judgement();
                let n = t.take_usize("arity");

                let mut args = Vec::with_capacity(n);

                for _ in 0..n {
                    args.push(t.judgement());
                }

                let definition = t.take_usize("definition index");

                self.resolver.inst_ix(m, &args, definition)
            }
            "cp" => {
                let a = t.judgement();

                self.resolver.cp(a)
            }
            "sp" => {
                let a = t.judgement();

                let ix = t.take_usize("sp index");

                self.resolver.sp(a, ix)
            }
            i => panic!("Unknown instruction {i}"),
        };

        t.end();

        eprintln!("ok");
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader};

    use super::*;

    #[test]
    pub fn check() {
        let input = include_str!("../check/log");

        Verifier::run(input);
    }

    #[test]
    pub fn check2() {
        let input_path = "../hw01/check/bez_rules";
        let input = BufReader::new(std::fs::File::open(input_path).unwrap());

        let mut v = Verifier::new();

        for line in input.lines() {
            let line = line.unwrap();
            if line == "-1" {
                break;
            }
            v.run_line(&line);
        }
    }
}
