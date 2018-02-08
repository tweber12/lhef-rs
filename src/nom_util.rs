// Copyright 2018 Torsten Weber
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Helper functions to implement parsers for lhe files using nom
//!
//! This module contains functions to parse numbers in decimal
//! represenetation from byte strings.
//!
//! `nom` itself has a `double` function to parse `f64`s, but this
//! function has many limitations, so `parse_f64` from this module is
//! preferable.

use std::f32;
use std::f64;
use std::ops::Mul;
use std::str;
use std::str::FromStr;

use nom;

macro_rules! consumed {
    ($i:expr, $parser:expr) => {
        match ws!($i, $parser) {
            nom::IResult::Done(rest,_) if rest.len() == $i.len()
                => nom::IResult::Error(nom::ErrorKind::Custom(0)),
            e => e,
        }
    }
}

macro_rules! perm_two {
    ($i:ident, $p1:ident, $p2:ident) => {
        alt!($i,
            do_parse!(v1: consumed!($p1) >> v2: ws!($p2) >> ((v1, v2))) |
            do_parse!(v2: $p2 >> v1: ws!($p1) >> ((v1, v2)))
        )
    }
}

macro_rules! perm_three {
    ($i:ident, $p1:ident, $p2:ident, $p3:ident) => {
        do_parse!($i,
            v1: consumed!($p1) >>
            v23: perm_two!($p2, $p3) >>
            ((v1, v23.0, v23.1))
        )
    }
}

macro_rules! permutation_opt {
    ($i:ident, $p1:ident, $p2:ident, $p3:ident) => {
        alt!($i,
             do_parse!(v: perm_three!($p1, $p2, $p3) >> ((v.0, v.1, v.2))) |
             do_parse!(v: perm_three!($p2, $p3, $p1) >> ((v.2, v.0, v.1))) |
             do_parse!(v: perm_three!($p3, $p1, $p2) >> ((v.1, v.2, v.0)))
           )
    }
}

named!(pub parse_f32<f32>,
    alt!(
        parse_finite_float |
        parse_finite_float_leading => {|x| x} |
        tag!("Infinity") => {|_| f32::INFINITY} |
        tag!("-Infinity") => {|_| f32::NEG_INFINITY} |
        tag!("NaN") => {|_| f32::NAN}
    )
);

named!(pub parse_f64<f64>,
    alt!(
        parse_finite_float |
        parse_finite_float_leading => {|x| x} |
        tag!("Infinity") => {|_| f64::INFINITY} |
        tag!("-Infinity") => {|_| f64::NEG_INFINITY} |
        tag!("NaN") => {|_| f64::NAN}
    )
);

fn parse_finite_float<T>(input: &[u8]) -> nom::IResult<&[u8], T>
where
    T: FromStr,
{
    map_res!(
        input,
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
        T::from_str
    )
}

fn parse_finite_float_leading<T>(input: &[u8]) -> nom::IResult<&[u8], T>
where
    T: FromStr,
{
    map_res!(
        input,
        map_res!(
            recognize!(do_parse!(
                opt!(alt!(tag!("+") | tag!("-")))
                    >> preceded!(tag!("."), take_while!(nom::is_digit))
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
        T::from_str
    )
}

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
named!(pub parse_i16<i16>, do_parse!(n: parse_int >> (n)));
named!(pub parse_i32<i32>, do_parse!(n: parse_int >> (n)));
named!(pub parse_i64<i64>, do_parse!(n: parse_int >> (n)));

named!(pub parse_u8<u8>, do_parse!(n: parse_uint >> (n)));
named!(pub parse_u16<u16>, do_parse!(n: parse_uint >> (n)));
named!(pub parse_u32<u32>, do_parse!(n: parse_uint >> (n)));
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
        assert_eq!(
            parse_f64(b"-.20889051E+01"),
            IResult::Done(b"" as &[u8], -0.20889051E+01)
        );
        assert_eq!(
            parse_f64(b".20889051E+01"),
            IResult::Done(b"" as &[u8], 0.20889051E+01)
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

    mod permutation_opt {
        use nom;

        fn a(input: &[u8]) -> nom::IResult<&[u8], &[u8]> {
            ws!(input, tag!("a"))
        }
        fn b(input: &[u8]) -> nom::IResult<&[u8], &[u8]> {
            ws!(input, tag!("b"))
        }
        fn c(input: &[u8]) -> nom::IResult<&[u8], &[u8]> {
            ws!(input, tag!("c"))
        }

        fn oa(input: &[u8]) -> nom::IResult<&[u8], Option<&[u8]>> {
            opt!(input, ws!(tag!("a")))
        }
        fn ob(input: &[u8]) -> nom::IResult<&[u8], Option<&[u8]>> {
            opt!(input, ws!(tag!("b")))
        }
        fn oc(input: &[u8]) -> nom::IResult<&[u8], Option<&[u8]>> {
            opt!(input, ws!(tag!("c")))
        }

        macro_rules! perm_3 {
            ($name:ident, $bytes:expr) => {
                #[test]
                fn $name() {
                    let bytes = $bytes;
                    let bs = bytes as &[u8];
                    let (ra, rb, rc) = permutation_opt!(bs, a, b, c).to_full_result().unwrap();
                    assert_eq!(ra, b"a");
                    assert_eq!(rb, b"b");
                    assert_eq!(rc, b"c");
                }
            }
        }
        perm_3!(abc, b"a b c");
        perm_3!(acb, b"a c b");
        perm_3!(bac, b"b a c");
        perm_3!(bca, b"b c a");
        perm_3!(cab, b"c a b");
        perm_3!(cba, b"c b a");

        macro_rules! perm_o_3 {
            ($name:ident, $bytes:expr) => {
                #[test]
                fn $name() {
                    let bytes = $bytes;
                    let bs = bytes as &[u8];
                    let (ra, rb, rc) = permutation_opt!(bs, oa, ob, oc).to_full_result().unwrap();
                    assert_eq!(ra, Some(b"a" as &[u8]));
                    assert_eq!(rb, Some(b"b" as &[u8]));
                    assert_eq!(rc, Some(b"c" as &[u8]));
                }
            }
        }
        perm_o_3!(abc_o, b"a b c");
        perm_o_3!(acb_o, b"a c b");
        perm_o_3!(bac_o, b"b a c");
        perm_o_3!(bca_o, b"b c a");
        perm_o_3!(cab_o, b"c a b");
        perm_o_3!(cba_o, b"c b a");

        macro_rules! perm_r_3 {
            ($name:ident, $bytes:expr) => {
                #[test]
                fn $name() {
                    let bytes = $bytes;
                    let bs = bytes as &[u8];
                    let (ra, rb, rc) = permutation_opt!(bs, oa, ob, c).to_full_result().unwrap();
                    assert_eq!(ra, Some(b"a" as &[u8]));
                    assert_eq!(rb, Some(b"b" as &[u8]));
                    assert_eq!(rc, b"c");
                }
            }
        }
        perm_r_3!(abc_r, b"a b c");
        perm_r_3!(acb_r, b"a c b");
        perm_r_3!(bac_r, b"b a c");
        perm_r_3!(bca_r, b"b c a");
        perm_r_3!(cab_r, b"c a b");
        perm_r_3!(cba_r, b"c b a");

        macro_rules! perm_o {
            ($name:ident, $bytes:expr, $ra:expr, $rb:expr, $rc:expr) => {
                #[test]
                fn $name() {
                    let bytes = $bytes;
                    let bs = bytes as &[u8];
                    let (ra, rb, rc) = permutation_opt!(bs, oa, ob, oc).to_full_result().unwrap();
                    assert_eq!(ra, $ra);
                    assert_eq!(rb, $rb);
                    assert_eq!(rc, $rc);
                }
            }
        }
        perm_o!(ab_o, b"abx", Some(b"a" as &[u8]), Some(b"b" as &[u8]), None);
        perm_o!(ba_o, b"bax", Some(b"a" as &[u8]), Some(b"b" as &[u8]), None);
        perm_o!(ac_o, b"acx", Some(b"a" as &[u8]), None, Some(b"c" as &[u8]));
        perm_o!(ca_o, b"cax", Some(b"a" as &[u8]), None, Some(b"c" as &[u8]));
        perm_o!(bc_o, b"bcx", None, Some(b"b" as &[u8]), Some(b"c" as &[u8]));
        perm_o!(cb_o, b"cbx", None, Some(b"b" as &[u8]), Some(b"c" as &[u8]));
        perm_o!(a_o, b"ax", Some(b"a" as &[u8]), None, None);
        perm_o!(b_o, b"bx", None, Some(b"b" as &[u8]), None);
        perm_o!(c_o, b"cx", None, None, Some(b"c" as &[u8]));

        macro_rules! perm_r {
            ($name:ident, $bytes:expr, $ra:expr, $rb:expr) => {
                #[test]
                fn $name() {
                    let bytes = $bytes;
                    let bs = bytes as &[u8];
                    let (ra, rb, rc) = permutation_opt!(bs, oa, ob, c).to_full_result().unwrap();
                    assert_eq!(ra, $ra);
                    assert_eq!(rb, $rb);
                    assert_eq!(rc, b"c");
                }
            }
        }
        perm_r!(ac_r, b"acx", Some(b"a" as &[u8]), None);
        perm_r!(ca_r, b"cax", Some(b"a" as &[u8]), None);
        perm_r!(bc_r, b"bcx", None, Some(b"b" as &[u8]));
        perm_r!(cb_r, b"cbx", None, Some(b"b" as &[u8]));
        perm_r!(c_r, b"cx", None, None);

        perm_r!(c_r_lead, b"    cx", None, None);
    }
}
