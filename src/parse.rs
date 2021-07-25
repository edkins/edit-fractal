use nom::{
    Finish,
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    combinator::{all_consuming, map, opt, recognize, value},
    multi::fold_many0,
    sequence::{preceded, terminated, tuple},
};
use std::{cmp::Ordering, fmt};
use crate::ast::Expr;

pub fn parse(input: &str) -> Result<Expr, ParseErr> {
    Ok(all_consuming(preceded(whitespace, expr))(input)
        .finish()
        .map_err(|e| ParseErr::new(e, input))?
        .1)
}

fn expr(input: &str) -> IResult<&str, Expr, Err> {
    let (input, init) = expr_term(input)?;
    fold_many0(
        alt((
                tuple((symbol_return("+"), expr_term)),
                tuple((symbol_return("-"), expr_term)),
        )),
        init,
        |lhs, (op, rhs)| {
            Expr::Call(op.to_owned(), vec![lhs, rhs])
        }
    )(input)
}

fn expr_term(input: &str) -> IResult<&str, Expr, Err> {
    let (input, init) = expr_tight(input)?;
    fold_many0(
        alt((
                tuple((symbol_return("*"), expr_tight)),
                tuple((symbol_return("/"), expr_tight)),
        )),
        init,
        |lhs, (op, rhs)| {
            Expr::Call(op.to_owned(), vec![lhs, rhs])
        }
    )(input)
}

fn expr_tight(input: &str) -> IResult<&str, Expr, Err> {
    map(
        terminated(recognize(tuple((digit1, opt(tuple((tag("."), digit1)))))), whitespace),
        |s: &str| Expr::F32(s.parse().unwrap()),
    )(input)
}

fn symbol_return<'a, 'b: 'a>(sym: &'b str) -> impl Fn(&'a str) -> IResult<&'a str, &'a str, Err> {
    move |input| {
        terminated(tag(sym), whitespace)(input)
            .map_err(|e| decorate(e, format!("Expected: {:?}", sym)))
    }
}

fn symbol<'a, 'b: 'a>(sym: &'b str) -> impl Fn(&'a str) -> IResult<&'a str, (), Err> {
    move |input| {
        terminated(tagv(sym), whitespace)(input)
            .map_err(|e| decorate(e, format!("Expected: {:?}", sym)))
    }
}

fn tagv<'a, 'b: 'a>(t: &'b str) -> impl Fn(&'a str) -> IResult<&'a str, (), Err> {
    move |input| value((), tag(t))(input)
}

fn whitespace(input: &str) -> IResult<&str, (), Err> {
    value((), multispace0)(input)
}

//////////////
// My errors
//////////////

struct Err {
    remaining: usize,
    message: String,
}

#[derive(Debug)]
pub struct ParseErr {
    pub text: String,
    pub remaining: usize,
    pub message: String,
}

impl<'a> nom::error::ParseError<&'a str> for Err {
    fn from_error_kind(input: &'a str, kind: nom::error::ErrorKind) -> Self {
        Err {
            remaining: input.len(),
            message: format!("{:?}", kind),
        }
    }
    fn append(input: &'a str, kind: nom::error::ErrorKind, other: Self) -> Self {
        if other.remaining <= input.len() {
            other
        } else {
            Self::from_error_kind(input, kind)
        }
    }
    fn from_char(input: &'a str, x: char) -> Self {
        Err {
            remaining: input.len(),
            message: format!("Expected: {:?}", x),
        }
    }
    fn or(self, other: Self) -> Self {
        match other.remaining.cmp(&self.remaining) {
            Ordering::Equal => Err {
                remaining: self.remaining,
                message: format!("{} | {}", self.message, other.message),
            },
            Ordering::Less => other,
            Ordering::Greater => self,
        }
    }
}

impl Err {
    fn decorate(self, extra: impl fmt::Display) -> Self {
        Err {
            remaining: self.remaining,
            message: format!("{} {}", self.message, extra),
        }
    }
}

fn decorate(err: nom::Err<Err>, extra: impl fmt::Display) -> nom::Err<Err> {
    match err {
        nom::Err::Error(e) => nom::Err::Error(e.decorate(extra)),
        nom::Err::Failure(e) => nom::Err::Failure(e.decorate(extra)),
        e => e,
    }
}

impl ParseErr {
    fn new(e: Err, text: &str) -> Self {
        ParseErr {
            text: text.to_owned(),
            remaining: e.remaining,
            message: e.message,
        }
    }
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pos = self.text.len() - self.remaining;
        write!(
            f,
            "{}####{} {}",
            &self.text[..pos],
            &self.text[pos..],
            self.message
        )
    }
}

impl std::error::Error for ParseErr {}
