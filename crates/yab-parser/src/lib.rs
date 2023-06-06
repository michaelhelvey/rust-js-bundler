#![allow(dead_code)]
#![allow(unused_imports)]
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till, take_until, take_while, take_while1},
    character::complete::{alpha0, i32, multispace0, space0},
    error::ParseError,
    multi::separated_list0,
    sequence::{delimited, separated_pair, tuple},
    Compare, IResult, InputLength, InputTake, Parser,
};

// Like Rust, all I want to do is writer a higher order function that takes a
// string and returns a "tag" parser.  'have, 'you, 'considered, 'fucking, 'yourself.
fn ws<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

fn parse_paren(input: &str) -> IResult<&str, &str> {
    ws(tag("("))(input)
}

fn my_parser(input: &str) -> IResult<&str, &str> {
    take_while(|c| c != ';')(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Result;

    #[test]
    fn it_works() -> Result<()> {
        let source = ";here is my text; and here is more";
        let (_, parsed) = my_parser(source)?;

        assert_eq!(parsed, "here is my text");
        Ok(())
    }
}
