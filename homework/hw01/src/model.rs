use colored::Colorize;
use std::{collections::HashSet, fmt::Display, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Var(pub char);

impl FromStr for Var {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 1
            && let Some(c) = s.chars().nth(0)
        {
            Ok(Self(c))
        } else {
            Err("must be a single character")
        }
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string().blue())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Asterisk;

impl Display for Asterisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "*".red())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Square;

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "@".red())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lambda(pub Var, pub Expr, pub Expr);

impl Display for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}:({}).({})", self.0, self.1, self.2)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pi(pub Var, pub Expr, pub Expr);

impl Display for Pi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "?{}:({}).({})", self.0, self.1, self.2)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Definition(pub String, pub Vec<Expr>);

impl Display for Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[", self.0.green())?;
        if !self.1.is_empty() {
            write!(f, "({})", self.1[0])?;
            for x in &self.1[1..] {
                write!(f, ",({})", x)?;
            }
        }
        write!(f, "]")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Application(pub Expr, pub Expr);

impl Display for Application {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%({})({})", self.0, self.1)
    }
}

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

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Var(var) => write!(f, "{var}"),
            Expr::Asterisk => write!(f, "{}", Asterisk),
            Expr::Square => write!(f, "{}", Square),
            Expr::Lambda(lambda) => write!(f, "{lambda}"),
            Expr::Pi(pi) => write!(f, "{pi}"),
            Expr::Definition(definition) => write!(f, "{definition}"),
            Expr::Application(application) => write!(f, "{application}"),
        }
    }
}

impl From<Var> for Expr {
    fn from(value: Var) -> Self {
        Self::Var(value)
    }
}

impl From<Asterisk> for Expr {
    fn from(_: Asterisk) -> Self {
        Expr::Asterisk
    }
}

impl From<Square> for Expr {
    fn from(_: Square) -> Self {
        Expr::Square
    }
}

impl From<Lambda> for Expr {
    fn from(value: Lambda) -> Self {
        Self::Lambda(Box::new(value))
    }
}

impl From<Pi> for Expr {
    fn from(value: Pi) -> Self {
        Expr::Pi(Box::new(value))
    }
}

impl From<Definition> for Expr {
    fn from(value: Definition) -> Self {
        Expr::Definition(value)
    }
}

impl From<Application> for Expr {
    fn from(value: Application) -> Self {
        Expr::Application(Box::new(value))
    }
}

impl FromStr for Expr {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let v = s.bytes().map(|b| b as char).collect::<Vec<_>>();
        crate::parse::take_expr(&mut v.as_slice())
    }
}

impl Expr {
    pub fn is_sort(&self) -> bool {
        matches!(self, Self::Asterisk | Self::Square)
    }

    pub fn de_bruijn(&self) -> crate::de_bruijn::Expr {
        crate::de_bruijn::de_bruijn(self, &crate::de_bruijn::Bindings::new(None))
    }

    pub fn alpha_substitution(&self, var: Var, expr: Expr) -> Expr {
        crate::de_bruijn::de_bruijn(self, &crate::de_bruijn::Bindings::new(Some((var, expr))))
            .into()
    }

    pub fn free_vars(&self) -> HashSet<Var> {
        match self {
            Expr::Asterisk | Expr::Square => HashSet::new(),
            Expr::Var(var) => HashSet::from([*var]),
            Expr::Lambda(lambda) => {
                let mut fv = lambda.2.free_vars();
                fv.remove(&lambda.0);
                fv.extend(lambda.1.free_vars());
                fv
            }
            Expr::Pi(pi) => {
                let mut fv = pi.2.free_vars();
                fv.remove(&pi.0);
                fv.extend(pi.1.free_vars());
                fv
            }
            Expr::Definition(definition) => {
                definition.1.iter().fold(HashSet::new(), |mut hs, e| {
                    hs.extend(e.free_vars());
                    hs
                })
            }
            Expr::Application(application) => {
                let mut fv = application.0.free_vars();
                fv.extend(application.1.free_vars());
                fv
            }
        }
    }
}

pub fn generate_free_var_gte(fv: &HashSet<Var>, mut v: Var) -> Var {
    #[allow(clippy::char_lit_as_u8)]
    while fv.contains(&v) {
        v.0 = ((v.0 as u8 + 1) % ('a' as u8)) as char;
    }
    v
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("x", ['x'])]
    #[case("%(x)(y)", ['x', 'y'])]
    #[case("$x:(*).(x)", [])]
    #[case("$x:(*).(%(x)(y))", ['y'])]
    fn free_vars(#[case] e: Expr, #[case] fv: impl IntoIterator<Item = char>) {
        assert_eq!(
            e.free_vars(),
            fv.into_iter().map(Var).collect::<HashSet<Var>>()
        );
    }
}
