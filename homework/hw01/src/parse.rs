use crate::model::{Application, Definition, Expr, Lambda, Pi, Var};

type Result<T> = std::result::Result<T, String>;
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

fn take_definition(name: String, input: &mut &[char]) -> Result<Definition> {
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
            buf.into_iter().collect::<String>()
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
