use std::io::BufRead;

use crate::{
    model::Var,
    rule::{self, Judgement},
};

#[derive(Clone, Debug, Default)]
pub struct Verifier {
    judgements: Vec<Judgement>,
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
        Self { judgements: vec![] }
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

        assert_eq!(lineno, self.judgements.len(), "wrong line number");

        let op_name = t.instruction();

        let j = match op_name {
            "sort" => rule::sort(),
            "var" => {
                let j = self.judgements[t.judgement()].clone();

                let var = t.variable();

                rule::var(j, var)
            }
            "weak" => {
                let a = self.judgements[t.judgement()].clone();
                let b = self.judgements[t.judgement()].clone();
                let var = t.variable();

                rule::weak(a, b, var)
            }
            "form" => {
                let a = self.judgements[t.judgement()].clone();
                let b = self.judgements[t.judgement()].clone();

                rule::form(a, b)
            }
            "appl" => {
                let a = self.judgements[t.judgement()].clone();
                let b = self.judgements[t.judgement()].clone();

                rule::appl(a, b)
            }
            "abst" => {
                let a = self.judgements[t.judgement()].clone();
                let b = self.judgements[t.judgement()].clone();

                rule::abst(a, b)
            }
            "conv" => {
                let a = self.judgements[t.judgement()].clone();
                let b = self.judgements[t.judgement()].clone();

                rule::conv(a, b)
            }
            "def" => {
                let a = self.judgements[t.judgement()].clone();
                let b = self.judgements[t.judgement()].clone();

                let name = t.constant();

                rule::def(a, b, name)
            }
            "defpr" => {
                let a = self.judgements[t.judgement()].clone();
                let b = self.judgements[t.judgement()].clone();

                let name = t.constant();

                rule::def_prim(a, b, name)
            }
            "inst" => {
                let m = self.judgements[t.judgement()].clone();
                let n = t.take_usize("arity");

                let mut args = Vec::with_capacity(n);

                for _ in 0..n {
                    args.push(self.judgements[t.judgement()].clone());
                }

                let definition = m.definitions[t.take_usize("definition index")].clone();

                rule::inst(m, &args, definition.name)
            }
            "cp" => {
                let a = self.judgements[t.judgement()].clone();

                rule::cp(a)
            }
            "sp" => {
                let a = self.judgements[t.judgement()].clone();

                let ix = t.take_usize("sp index");

                rule::sp(a, ix)
            }
            i => panic!("Unknown instruction {i}"),
        };

        t.end();

        self.judgements.push(j);

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
