use colored::Colorize;

use std::fmt::Display;

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug)]
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

fn take_one(input: &mut &[char]) -> Result<char> {
    if input.is_empty() {
        return Err("Unexpected end of input".to_string());
    }
    let c = input[0];
    *input = &input[1..];
    Ok(c)
}

fn take_var(input: &mut &[char]) -> Result<Var> {
    let v = take_one(input)?;
    if v.is_ascii_alphabetic() {
        Ok(Var(v))
    } else {
        Err("Expecting variable".to_string())
    }
}

// fn take_asterisk(input: &mut &[char]) -> Result<Asterisk> {
//     take_exact(input, '*')?;
//     Ok(Asterisk)
// }

// fn take_square(input: &mut &[char]) -> Result<Square> {
//     take_exact(input, '@')?;
//     Ok(Square)
// }

fn take_exact(input: &mut &[char], e: char) -> Result<()> {
    if input.is_empty() {
        return Err("Unexpected end of input".to_string());
    }
    let v = input[0];
    if v == e {
        take_one(input)?;
        Ok(())
    } else {
        Err(format!("Expecting {e}"))
    }
}

fn take_pi(input: &mut &[char]) -> Result<Pi> {
    let x = take_var(input)?;
    take_exact(input, ':')?;
    take_exact(input, '(')?;
    let m = take_expr(input)?;
    take_exact(input, ')')?;
    take_exact(input, '.')?;
    take_exact(input, '(')?;
    let n = take_expr(input)?;
    take_exact(input, ')')?;
    Ok(Pi(x, m, n))
}

fn take_lambda(input: &mut &[char]) -> Result<Lambda> {
    let x = take_var(input)?;
    take_exact(input, ':')?;
    take_exact(input, '(')?;
    let m = take_expr(input)?;
    take_exact(input, ')')?;
    take_exact(input, '.')?;
    take_exact(input, '(')?;
    let n = take_expr(input)?;
    take_exact(input, ')')?;
    Ok(Lambda(x, m, n))
}

fn take_application(input: &mut &[char]) -> Result<Application> {
    take_exact(input, '(')?;
    let m = take_expr(input)?;
    take_exact(input, ')')?;
    take_exact(input, '(')?;
    let n = take_expr(input)?;
    take_exact(input, ')')?;
    Ok(Application(m, n))
}

fn take_definition(name: Box<[char]>, input: &mut &[char]) -> Result<Definition> {
    take_exact(input, '[')?;
    let mut d = vec![];
    if take_exact(input, '(').is_ok() {
        let m = take_expr(input)?;
        d.push(m);
        take_exact(input, ')')?;
        while take_exact(input, ',').is_ok() {
            take_exact(input, '(')?;
            let m = take_expr(input)?;
            d.push(m);
            take_exact(input, ')')?;
        }
    }
    take_exact(input, ']')?;
    Ok(Definition(name, d))
}

pub fn take_expr(input: &mut &[char]) -> Result<Expr> {
    let mut i = 0;
    while i < input.len() && input[i].is_ascii_alphabetic() {
        i += 1;
    }
    if i == 1 {
        return Ok(Var(take_one(input)?).into());
    } else if i > 1 {
        let name = {
            let mut buf = vec!['0'; i];
            buf.copy_from_slice(&input[0..i]);
            buf.into_boxed_slice()
        };
        *input = &input[i..];
        return take_definition(name, input).map(Into::into);
    }

    let c = take_one(input)?;
    match c {
        '*' => Ok(Expr::Asterisk),
        '@' => Ok(Expr::Square),
        '%' => take_application(input).map(Into::into),
        '$' => take_lambda(input).map(Into::into),
        '?' => take_pi(input).map(Into::into),
        _ => Err(format!("Unexpected: {:?}", c)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[test]
    #[case(b"a")]
    #[case(b"b")]
    #[case(b"C")]
    #[case(b"*")]
    #[case(b"@")]
    #[case(b"%(a)(b)")]
    #[case(b"%(a)(b)")]
    #[case(b"$x:(M).(N)")]
    #[case(b"?x:(M).(N)")]
    #[case(b"empty[]")]
    #[case(b"implies[(M),(N)]")]
    #[case(b"?x:(?x:(M).(%(a)(b))).(%(a)(b))")]
    fn valid(#[case] input: &'static [u8]) {
        let v = input.iter().map(|b| *b as char).collect::<Vec<_>>();
        println!("{:?}", take_expr(&mut v.as_slice()).unwrap());
    }

    #[rstest]
    #[test]
    #[case(b"1")]
    #[case(b"&")]
    #[case(b"%")]
    #[case(b"$")]
    #[case(b"#")]
    #[case(b"%:(a)(b)")]
    #[case(b"%(a).(b)")]
    #[case(b"$x:(M.(N)")]
    #[case(b"x:(M).(N)")]
    #[case(b"e[]")]
    #[case(b"implies[(M),]")]
    #[case(b"implies[()]")]
    #[case(b"?x:(?x:(M).(%(a)((b)))).(%(a)(b))")]
    fn invalid(#[case] input: &'static [u8]) {
        let v = input.iter().map(|b| *b as char).collect::<Vec<_>>();
        println!("{:?}", take_expr(&mut v.as_slice()).unwrap_err());
    }
}
