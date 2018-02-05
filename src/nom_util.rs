// Copyright 2018 Torsten Weber
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::f64;
use std::ops::Mul;
use std::str;
use std::str::FromStr;

use nom;

named!(pub parse_f64<f64>,
    alt!(
        parse_finite_f64 => {|x| x} |
        tag!("Infinity") => {|_| f64::INFINITY} |
        tag!("-Infinity") => {|_| f64::NEG_INFINITY} |
        tag!("NaN") => {|_| f64::NAN}
    )
);

named!(
    parse_finite_f64<f64>,
    map_res!(
        map_res!(
            recognize!(do_parse!(
                opt!(alt!(tag!("+") | tag!("-"))) >> take_while1!(nom::is_digit)
                    >> opt!(preceded!(tag!("."), take_while!(nom::is_digit)))
                    >> opt!(preceded!(
                        alt!(tag!("e") | tag!("E")),
                        preceded!(
                            opt!(alt!(tag!("+") | tag!("-"))),
                            take_while1!(nom::is_digit)
                        )
                    )) >> (())
            )),
            str::from_utf8
        ),
        f64::from_str
    )
);

fn parse_int<T>(input: &[u8]) -> nom::IResult<&[u8], T>
where
    T: FromStr,
{
    map_res!(
        input,
        map_res!(
            recognize!(preceded!(
                opt!(alt!(tag!("+") | tag!("-"))),
                take_while1!(nom::is_digit)
            )),
            str::from_utf8
        ),
        T::from_str
    )
}

fn parse_uint<T>(input: &[u8]) -> nom::IResult<&[u8], T>
where
    T: FromStr + Mul<Output = T>,
{
    map_res!(
        input,
        map_res!(
            recognize!(preceded!(opt!(tag!("+")), take_while1!(nom::is_digit))),
            str::from_utf8
        ),
        T::from_str
    )
}

named!(pub parse_i8<i8>, do_parse!(n: parse_int >> (n)));
named!(pub parse_i64<i64>, do_parse!(n: parse_int >> (n)));

named!(pub parse_u8<u8>, do_parse!(n: parse_uint >> (n)));
named!(pub parse_u64<u64>, do_parse!(n: parse_uint >> (n)));

#[cfg(test)]
mod test {
    use nom::IResult;
    use quickcheck::TestResult;

    #[test]
    fn parse_f64() {
        use super::parse_f64;
        assert_eq!(parse_f64(b"1 "), IResult::Done(b" " as &[u8], 1.));
        assert_eq!(parse_f64(b"+1 "), IResult::Done(b" " as &[u8], 1.));
        assert_eq!(parse_f64(b"-1 "), IResult::Done(b" " as &[u8], -1.));
        assert_eq!(parse_f64(b"1. "), IResult::Done(b" " as &[u8], 1.));
        assert_eq!(parse_f64(b"+1. "), IResult::Done(b" " as &[u8], 1.));
        assert_eq!(parse_f64(b"-1. "), IResult::Done(b" " as &[u8], -1.));
        assert_eq!(parse_f64(b"1e0 "), IResult::Done(b" " as &[u8], 1.));
        assert_eq!(parse_f64(b"+1e0 "), IResult::Done(b" " as &[u8], 1.));
        assert_eq!(parse_f64(b"-1e0 "), IResult::Done(b" " as &[u8], -1.));
        assert_eq!(parse_f64(b"1e0 "), IResult::Done(b" " as &[u8], 1.));
        assert_eq!(parse_f64(b"+1e0 "), IResult::Done(b" " as &[u8], 1.));
        assert_eq!(parse_f64(b"-1e0 "), IResult::Done(b" " as &[u8], -1.));
        assert_eq!(parse_f64(b"1e0 "), IResult::Done(b" " as &[u8], 1.));
        assert_eq!(parse_f64(b"+1e0 "), IResult::Done(b" " as &[u8], 1.));
        assert_eq!(parse_f64(b"-1e0 "), IResult::Done(b" " as &[u8], -1.));
        assert_eq!(parse_f64(b"1.0e0 "), IResult::Done(b" " as &[u8], 1.));
        assert_eq!(parse_f64(b"1.0e+0 "), IResult::Done(b" " as &[u8], 1.));
        assert_eq!(parse_f64(b"1.0e-0 "), IResult::Done(b" " as &[u8], 1.));
        assert_eq!(
            parse_f64(b"4.705810011652687E+01"),
            IResult::Done(b"" as &[u8], 4.705810011652687E+01)
        );
        assert_eq!(
            parse_f64(b"-5.818122260105206E+00"),
            IResult::Done(b"" as &[u8], -5.818122260105206E+00)
        );
        assert_eq!(
            parse_f64(b"2.228997274254760E-08"),
            IResult::Done(b"" as &[u8], 2.228997274254760E-08)
        );
    }

    #[test]
    fn parse_f64_0() {
        use super::parse_f64;
        assert_eq!(parse_f64(b"0 "), IResult::Done(b" " as &[u8], 0.));
        assert_eq!(parse_f64(b"0. "), IResult::Done(b" " as &[u8], 0.));
        assert_eq!(parse_f64(b"0.0 "), IResult::Done(b" " as &[u8], 0.));
        assert_eq!(parse_f64(b"0.0E3 "), IResult::Done(b" " as &[u8], 0.));
        assert_eq!(parse_f64(b"0.0E3 "), IResult::Done(b" " as &[u8], 0.));
        assert_eq!(parse_f64(b"0.0000E+00"), IResult::Done(b"" as &[u8], 0.));
        assert_eq!(parse_f64(b"0.000000E+00"), IResult::Done(b"" as &[u8], 0.));
        assert_eq!(
            parse_f64(b"0.000000000000000E+00"),
            IResult::Done(b"" as &[u8], 0.)
        );
    }

    #[test]
    fn parse_f64_special() {
        use std::f64;
        use super::parse_f64;
        assert_eq!(
            parse_f64(b"Infinity"),
            IResult::Done(b"" as &[u8], f64::INFINITY)
        );
        assert_eq!(
            parse_f64(b"-Infinity"),
            IResult::Done(b"" as &[u8], f64::NEG_INFINITY)
        );
        match parse_f64(b"NaN") {
            IResult::Done(r, v) => {
                assert_eq!(r, b"" as &[u8]);
                assert!(v.is_nan());
            }
            err => panic!("Failed to parse NaN, {:?}", err),
        }
    }

    quickcheck! {
        fn parse_f64_qc(n: f64) -> bool {
            println!("{}", n);
            let string = format!("{} ", n);
            let r = super::parse_f64(string.as_bytes()).to_full_result().map(|p| { println!("{} == {}", p, n); p == n }).unwrap_or(false);
            println!("{}", r);
            r
        }
    }

    #[test]
    fn parse_i8() {
        use super::parse_i8;
        assert_eq!(parse_i8(b"8 "), IResult::Done(b" " as &[u8], 8));
        assert_eq!(parse_i8(b"+8 "), IResult::Done(b" " as &[u8], 8));
        assert_eq!(parse_i8(b"-8 "), IResult::Done(b" " as &[u8], -8));
        assert_eq!(parse_i8(b"0 "), IResult::Done(b" " as &[u8], 0));
        assert_eq!(parse_i8(b"127 "), IResult::Done(b" " as &[u8], 127));
        assert_eq!(parse_i8(b"-128 "), IResult::Done(b" " as &[u8], -128));
    }

    quickcheck! {
        fn parse_i8_qc(n: i8) -> bool {
            let string = n.to_string();
            super::parse_i8(string.as_bytes()).to_full_result().map(|p| p == n).unwrap_or(false)
        }
    }

    #[test]
    fn parse_i8_overflow() {
        use super::parse_i8;
        assert!(parse_i8(b"128 ").is_err());
        assert!(parse_i8(b"-129 ").is_err());
        assert!(parse_i8(b"31290 ").is_err());
        assert!(parse_i8(b"-9242 ").is_err());
    }

    quickcheck! {
        fn parse_i8_overflow_qc(n: i64) -> TestResult {
            use std::i8;
            if n <= i8::MAX as i64 && n >= i8::MIN as i64 {
                return TestResult::discard();
            }
            let string = n.to_string();
            TestResult::from_bool(super::parse_i8(string.as_bytes()).to_full_result().is_err())
        }
    }

    #[test]
    fn parse_i64() {
        use super::parse_i64;
        use std::i64;
        assert_eq!(parse_i64(b"8 "), IResult::Done(b" " as &[u8], 8));
        assert_eq!(parse_i64(b"+8 "), IResult::Done(b" " as &[u8], 8));
        assert_eq!(parse_i64(b"-8 "), IResult::Done(b" " as &[u8], -8));
        assert_eq!(parse_i64(b"0 "), IResult::Done(b" " as &[u8], 0));
        assert_eq!(parse_i64(b"127 "), IResult::Done(b" " as &[u8], 127));
        assert_eq!(parse_i64(b"-128 "), IResult::Done(b" " as &[u8], -128));
        assert_eq!(
            parse_i64(b"-9223372036854775808 "),
            IResult::Done(b" " as &[u8], i64::MIN)
        );
        assert_eq!(
            parse_i64(b"9223372036854775807 "),
            IResult::Done(b" " as &[u8], i64::MAX)
        );
    }

    quickcheck! {
        fn parse_i64_qc(n: i64) -> bool {
            let string = n.to_string();
            super::parse_i64(string.as_bytes()).to_full_result().map(|p| p == n).unwrap_or(false)
        }
    }

    #[test]
    fn parse_i64_overflow() {
        use super::parse_i64;
        assert!(parse_i64(b"9223372036854775808 ").is_err());
        assert!(parse_i64(b"-9223372036854775809 ").is_err());
        assert!(parse_i64(b"3129058230958202589025 ").is_err());
        assert!(parse_i64(b"-9242920589023509252992 ").is_err());
    }

    #[test]
    fn parse_u8() {
        use super::parse_u8;
        assert_eq!(parse_u8(b"8 "), IResult::Done(b" " as &[u8], 8));
        assert_eq!(parse_u8(b"+8 "), IResult::Done(b" " as &[u8], 8));
        assert_eq!(parse_u8(b"0"), IResult::Done(b"" as &[u8], 0));
        assert_eq!(parse_u8(b"127 "), IResult::Done(b" " as &[u8], 127));
        assert_eq!(parse_u8(b"255 "), IResult::Done(b" " as &[u8], 255));
    }

    quickcheck! {
        fn parse_u8_qc(n: u8) -> bool {
            let string = n.to_string();
            super::parse_u8(string.as_bytes()).to_full_result().map(|p| p == n).unwrap_or(false)
        }
    }

    #[test]
    fn parse_u8_overflow() {
        use super::parse_u8;
        assert!(parse_u8(b"256 ").is_err());
        assert!(parse_u8(b"-129 ").is_err());
        assert!(parse_u8(b"31290 ").is_err());
        assert!(parse_u8(b"-9242 ").is_err());
    }

    quickcheck! {
        fn parse_u8_overflow_qc(n: i64) -> TestResult {
            use std::u8;
            if n <= u8::MAX as i64 && n >= u8::MIN as i64 {
                return TestResult::discard();
            }
            let string = n.to_string();
            TestResult::from_bool(super::parse_u8(string.as_bytes()).to_full_result().is_err())
        }
    }
}
