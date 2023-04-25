use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        alpha1, alphanumeric1, digit1, hex_digit1, multispace0, oct_digit1, one_of,
    },
    combinator::{map, opt, recognize},
    multi::{many0, many1},
    sequence::{pair, tuple},
    IResult,
};

/// Parse source code to get an ident.
pub fn ident(code: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s: &str| s.to_string(),
    )(code)
}

/// Wrap parser `f`, so we can ignore the multispaces around the content we want to parse.
pub fn in_multispace<F, I, O>(f: F) -> impl FnMut(I) -> IResult<I, O>
where
    I: nom::InputTakeAtPosition + Clone,
    <I as nom::InputTakeAtPosition>::Item: nom::AsChar + Clone,
    F: FnMut(I) -> IResult<I, O>,
{
    map(tuple((multispace0, f, multispace0)), |(_, x, _)| x)
}

/// Parse source code to get an integer literal.
/// Supports hex (startes with 0x) or dec.
pub fn integer<T: TryFrom<i64>>(code: &str) -> IResult<&str, T>
where
    <T as TryFrom<i64>>::Error: std::fmt::Debug,
{
    map(
        pair(
            opt(tag("-")),
            alt((
                map(pair(tag("0x"), hex_digit1), |(_, hexadecimal)| {
                    i64::from_str_radix(hexadecimal, 16).unwrap()
                }),
                map(pair(tag("0o"), oct_digit1), |(_, octal)| {
                    i64::from_str_radix(octal, 8).unwrap()
                }),
                map(pair(tag("0b"), many1(one_of("01"))), |(_, binary)| {
                    binary
                        .iter()
                        .fold(0, |acc, &b| acc * 2 + b as i64 - '0' as i64)
                }),
                map(digit1, |decimal: &str| decimal.parse::<i64>().unwrap()),
            )),
        ),
        |(neg, n)| {
            if neg.is_some() {
                T::try_from(-n).unwrap()
            } else {
                T::try_from(n).unwrap()
            }
        },
    )(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_ident() {
        let result = ident("a").unwrap().1;
        assert_eq!(result, "a".to_string());
        let result = ident("a_b_c").unwrap().1;
        assert_eq!(result, "a_b_c".to_string());
        let result = ident("WHILE_0_JUDGE").unwrap().1;
        assert_eq!(result, "WHILE_0_JUDGE".to_string());
        let result = ident("_").unwrap().1;
        assert_eq!(result, "_".to_string());

        let result = ident("1a").is_err();
        assert!(result);
    }

    #[test]
    fn can_parse_integer() {
        let result: i32 = integer("0").unwrap().1;
        assert_eq!(result, 0);
        let result: i32 = integer("99").unwrap().1;
        assert_eq!(result, 99);
        let result: i32 = integer("-99").unwrap().1;
        assert_eq!(result, -99);
        let result: i32 = integer("0x40000000").unwrap().1;
        assert_eq!(result, 0x40000000);
        let result: i32 = integer("-0x40000000").unwrap().1;
        assert_eq!(result, -0x40000000);
        let result: i32 = integer("0b01101").unwrap().1;
        assert_eq!(result, 0b01101);
        let result: i32 = integer("-0b01101").unwrap().1;
        assert_eq!(result, -0b01101);
        let result: i32 = integer("0o777").unwrap().1;
        assert_eq!(result, 0o777);
        let result: i32 = integer("-0o777").unwrap().1;
        assert_eq!(result, -0o777);
    }
}
