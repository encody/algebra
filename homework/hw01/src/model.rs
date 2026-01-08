use colored::Colorize;
use std::{fmt::Display, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Var(pub char);

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

#[derive(Clone, Debug)]
pub struct Lambda(pub Var, pub Expr, pub Expr);

impl Display for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}:({}).({})", self.0, self.1, self.2)
    }
}

#[derive(Clone, Debug)]
pub struct Pi(pub Var, pub Expr, pub Expr);

impl Display for Pi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "?{}:({}).({})", self.0, self.1, self.2)
    }
}

#[derive(Clone, Debug)]
pub struct Definition(pub Box<[char]>, pub Vec<Expr>);

impl Display for Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[", self.0.iter().collect::<String>().green())?;
        if !self.1.is_empty() {
            write!(f, "({})", self.1[0])?;
            for x in &self.1[1..] {
                write!(f, ",({})", x)?;
            }
        }
        write!(f, "]")
    }
}

#[derive(Clone, Debug)]
pub struct Application(pub Expr, pub Expr);

impl Display for Application {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%({})({})", self.0, self.1)
    }
}

#[derive(Clone, Debug)]
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
    pub fn de_bruijn(&self) -> crate::de_bruijn::Expr {
        crate::de_bruijn::de_bruijn(self, &crate::de_bruijn::BindingStack::Empty)
    }
}
