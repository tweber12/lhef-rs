// Copyright 2018 Torsten Weber
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A module to read and write lhe files, keeping extra information as strings
//!
//! This module contains types that can be used in `LheFileGeneric` to
//! read lhe files and extract all additional information contained in
//! the file as strings.
//!
//! # Examples
//!
//! ## Reading from a file
//!
//! ```rust,ignore
//! use lhef::ReadLhe;
//! use lhef::string::{LheFile, EventExtra};
//!
//! let lhe = LheFile::read_lhe_from_file(&"events.lhe").unwrap();
//!
//! // extra information of the 5th event
//! let EventExtra(ref extra) = lhe.events[4].extra;
//! ```
//!
//! ## Reading from a byte string
//!
//! ```rust
//! use lhef::{Particle, ReadLhe};
//! use lhef::string::{LheFile, Comment, Header, InitExtra, EventExtra};
//!
//! let bytes = b"\
//! <LesHouchesEvents version=\"1.0\">
//! <!-- Process: e+ e- > mu+ mu- -->
//! <header>
//! <tag> Important header information </tag>
//! </header>
//! <init>
//! 2212 2212 6500 6500 0 0 13100 13100 3 1
//! 2.1 3.2E-03 1.0E+00 1
//! ## Additional initialization information
//! </init>
//! <event>
//! 4 1 +1.04e-01 1.00e+03 7.54e-03 8.68e-02
//! -11 -1 0 0 0 0 +0.00e+00 +0.00e+00 +5.00e+02 5.00e+02 0.00e+00 0.00e+00 -1.00e+00
//!  11 -1 0 0 0 0 -0.00e+00 -0.00e+00 -5.00e+02 5.00e+02 0.00e+00 0.00e+00  1.00e+00
//! -13  1 1 2 0 0 -1.97e+02 -4.52e+02 -7.94e+01 5.00e+02 0.00e+00 0.00e+00 -1.00e+00
//!  13  1 1 2 0 0 +1.97e+02 +4.52e+02 +7.94e+01 5.00e+02 0.00e+00 0.00e+00  1.00e+00
//! ## Additional event information
//! </event>
//! </LesHouchesEvents>";
//!
//! let lhe = LheFile::read_lhe(bytes).to_full_result().unwrap();
//!
//! assert_eq!(lhe.version, "1.0".to_string());
//!
//! let Comment { ref comment } = lhe.comment;
//! assert_eq!(comment, &Some("Process: e+ e- > mu+ mu-".to_string()));
//!
//! let Header { ref header } = lhe.header;
//! assert_eq!(header, &Some("<tag> Important header information </tag>".to_string()));
//!
//! let init = &lhe.init;
//! assert_eq!(init.beam_1_id, 2212);
//! assert_eq!(init.process_info[0].process_id, 1);
//! let InitExtra(ref extra) = init.extra;
//! assert_eq!(extra, &"# Additional initialization information".to_string());
//!
//! let event = &lhe.events[0];
//! assert_eq!(event.process_id, 1);
//! assert_eq!(event.particles[0].pdg_id, -11);
//! let EventExtra(ref extra) = event.extra;
//! assert_eq!(extra, &"# Additional event information".to_string());
//! ```

use {ReadLhe, WriteLhe};
use generic::LheFileGeneric;

use nom;
use std::io;
use std::str;

#[cfg(test)]
use quickcheck::Arbitrary;
#[cfg(test)]
use quickcheck::Gen;

/// A file to read and write lhe files, keeping extra information as strings
///
/// Any additional information stored in the file is kept as strings,
/// with leading and trailing whitespace removed.
/// Each type of additional information is optional, i.e. files
/// containing only the mandatory information can also be parsed.
///
/// # Examples
///
/// ```rust
/// use lhef::{Particle, ReadLhe};
/// use lhef::string::{LheFile, Comment, Header, InitExtra, EventExtra};
///
/// let bytes = b"\
/// <LesHouchesEvents version=\"1.0\">
/// <!-- Process: e+ e- > mu+ mu- -->
/// <header>
/// <tag> Important header information </tag>
/// </header>
/// <init>
/// 2212 2212 6500 6500 0 0 13100 13100 3 1
/// 2.1 3.2E-03 1.0E+00 1
/// ## Additional initialization information
/// </init>
/// <event>
/// 4 1 +1.04e-01 1.00e+03 7.54e-03 8.68e-02
/// -11 -1 0 0 0 0 +0.00e+00 +0.00e+00 +5.00e+02 5.00e+02 0.00e+00 0.00e+00 -1.00e+00
///  11 -1 0 0 0 0 -0.00e+00 -0.00e+00 -5.00e+02 5.00e+02 0.00e+00 0.00e+00  1.00e+00
/// -13  1 1 2 0 0 -1.97e+02 -4.52e+02 -7.94e+01 5.00e+02 0.00e+00 0.00e+00 -1.00e+00
///  13  1 1 2 0 0 +1.97e+02 +4.52e+02 +7.94e+01 5.00e+02 0.00e+00 0.00e+00  1.00e+00
/// ## Additional event information
/// </event>
/// </LesHouchesEvents>";
///
/// let lhe = LheFile::read_lhe(bytes).to_full_result().unwrap();
///
/// assert_eq!(lhe.version, "1.0".to_string());
///
/// let Comment { ref comment } = lhe.comment;
/// assert_eq!(comment, &Some("Process: e+ e- > mu+ mu-".to_string()));
///
/// let Header { ref header } = lhe.header;
/// assert_eq!(header, &Some("<tag> Important header information </tag>".to_string()));
///
/// let init = &lhe.init;
/// assert_eq!(init.beam_1_id, 2212);
/// assert_eq!(init.process_info[0].process_id, 1);
/// let InitExtra(ref extra) = init.extra;
/// assert_eq!(extra, &"# Additional initialization information".to_string());
///
/// let event = &lhe.events[0];
/// assert_eq!(event.process_id, 1);
/// assert_eq!(event.particles[0].pdg_id, -11);
/// let EventExtra(ref extra) = event.extra;
/// assert_eq!(extra, &"# Additional event information".to_string());
/// ```
pub type LheFile = LheFileGeneric<Comment, Header, InitExtra, EventExtra>;

/// A optional comment in an lhe file, as a string
///
/// The contents of the comment are contained in `Comment` as a string
/// if a comment was present in the file, with leading and trailing
/// whitespace removed.
///
/// # Examples
///
/// If a comment is there...
///
/// ```rust
/// use lhef::{Particle, ReadLhe};
/// use lhef::string::{LheFile, Comment};
///
/// let bytes = b"\
/// <LesHouchesEvents version=\"1.0\">
/// <!-- Process: e+ e- > mu+ mu- -->
/// <init>
/// 2212 2212 6500 6500 0 0 13100 13100 3 1
/// 2.1 3.2E-03 1.0E+00 1
/// </init>
/// </LesHouchesEvents>";
///
/// let lhe = LheFile::read_lhe(bytes).to_full_result().unwrap();
///
/// let Comment { ref comment } = lhe.comment;
/// assert_eq!(comment, &Some("Process: e+ e- > mu+ mu-".to_string()));
/// ```
///
/// ...and if it isn't.
///
/// ```rust
/// # use lhef::{Particle, ReadLhe};
/// # use lhef::string::{LheFile, Comment};
/// let bytes = b"\
/// <LesHouchesEvents version=\"1.0\">
/// <init>
/// 2212 2212 6500 6500 0 0 13100 13100 3 1
/// 2.1 3.2E-03 1.0E+00 1
/// </init>
/// </LesHouchesEvents>";
///
/// let lhe = LheFile::read_lhe(bytes).to_full_result().unwrap();
///
/// let Comment { ref comment } = lhe.comment;
/// assert_eq!(comment, &None);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct Comment {
    pub comment: Option<String>,
}

impl ReadLhe for Comment {
    fn read_lhe(input: &[u8]) -> nom::IResult<&[u8], Comment> {
        map!(
            input,
            opt!(map_res!(
                delimited!(tag!("<!--"), take_until!("-->"), tag!("-->")),
                |x| str::from_utf8(x).map(|x| x.trim().to_string())
            )),
            |comment| Comment { comment }
        )
    }
}

impl WriteLhe for Comment {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        match self.comment {
            Some(ref s) => {
                writeln!(writer, "<!--")?;
                writeln!(writer, "{}", s)?;
                writeln!(writer, "-->")
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
impl Arbitrary for Comment {
    fn arbitrary<G: Gen>(gen: &mut G) -> Comment {
        let mut contents: Option<String> = Arbitrary::arbitrary(gen);
        while contents
            .as_ref()
            .map(|s| s.contains("-->"))
            .unwrap_or(false)
        {
            contents = Arbitrary::arbitrary(gen);
        }
        let contents = contents.map(|x| x.trim().to_string());
        Comment { comment: contents }
    }
}

/// A optional header section in an lhe file, as a string
///
/// The contents of the header are contained in `Header` as a string
/// if a comment was present in the file, with leading and trailing
/// whitespace removed.
///
/// # Examples
///
/// If a comment is there...
///
/// ```rust
/// use lhef::{Particle, ReadLhe};
/// use lhef::string::{LheFile, Header};
///
/// let bytes = b"\
/// <LesHouchesEvents version=\"1.0\">
/// <header>
/// <tag> Important header information </tag>
/// </header>
/// <init>
/// 2212 2212 6500 6500 0 0 13100 13100 3 1
/// 2.1 3.2E-03 1.0E+00 1
/// </init>
/// </LesHouchesEvents>";
///
/// let lhe = LheFile::read_lhe(bytes).to_full_result().unwrap();
///
/// let Header { ref header } = lhe.header;
/// assert_eq!(header, &Some("<tag> Important header information </tag>".to_string()));
/// ```
///
/// ... and if it isn't.
///
/// ```rust
/// # use lhef::{Particle, ReadLhe};
/// # use lhef::string::{LheFile, Header};
/// let bytes = b"\
/// <LesHouchesEvents version=\"1.0\">
/// <init>
/// 2212 2212 6500 6500 0 0 13100 13100 3 1
/// 2.1 3.2E-03 1.0E+00 1
/// </init>
/// </LesHouchesEvents>";
///
/// let lhe = LheFile::read_lhe(bytes).to_full_result().unwrap();
///
/// let Header { ref header } = lhe.header;
/// assert_eq!(header, &None);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct Header {
    pub header: Option<String>,
}

impl ReadLhe for Header {
    fn read_lhe(input: &[u8]) -> nom::IResult<&[u8], Header> {
        map!(
            input,
            opt!(map_res!(
                delimited!(
                    tag!("<header>"),
                    take_until!("</header>"),
                    tag!("</header>")
                ),
                |x| str::from_utf8(x).map(|x| x.trim().to_string())
            )),
            |header| Header { header }
        )
    }
}

impl WriteLhe for Header {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        match self.header {
            Some(ref s) => {
                writeln!(writer, "<header>")?;
                writeln!(writer, "{}", s)?;
                writeln!(writer, "</header>")
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
impl Arbitrary for Header {
    fn arbitrary<G: Gen>(gen: &mut G) -> Header {
        let mut contents: Option<String> = Arbitrary::arbitrary(gen);
        while contents
            .as_ref()
            .map(|s| s.contains("</header>"))
            .unwrap_or(false)
        {
            contents = Arbitrary::arbitrary(gen);
        }
        let contents = contents.map(|x| x.trim().to_string());
        Header { header: contents }
    }
}

/// Extra initialization information, as a string
///
/// Any additional initialization information is contained in
/// `InitExtra` as a string, with leading and trailing whitespace
/// removed.
/// If no additional information is included in the file, the string is
/// empty.
///
/// # Examples
///
/// ```rust
/// use lhef::{Particle, ReadLhe};
/// use lhef::string::{LheFile, InitExtra};
///
/// let bytes = b"\
/// <LesHouchesEvents version=\"1.0\">
/// <init>
/// 2212 2212 6500 6500 0 0 13100 13100 3 1
/// 2.1 3.2E-03 1.0E+00 1
/// ## Additional initialization information
/// ## Even more important stuff
/// 1 2 3 4 5
/// </init>
/// </LesHouchesEvents>";
///
/// let lhe = LheFile::read_lhe(bytes).to_full_result().unwrap();
///
/// let InitExtra(ref extra) = lhe.init.extra;
/// assert_eq!(extra, &"\
/// ## Additional initialization information
/// ## Even more important stuff
/// 1 2 3 4 5".to_string()
/// );
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct InitExtra(pub String);

impl ReadLhe for InitExtra {
    fn read_lhe(input: &[u8]) -> nom::IResult<&[u8], InitExtra> {
        map_res!(input, take_until!("</init>"), |x| str::from_utf8(x)
            .map(|x| InitExtra(x.trim().to_string())))
    }
}

impl WriteLhe for InitExtra {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let &InitExtra(ref s) = self;
        if s.is_empty() {
            Ok(())
        } else {
            writeln!(writer, "{}", s)
        }
    }
}

#[cfg(test)]
impl Arbitrary for InitExtra {
    fn arbitrary<G: Gen>(gen: &mut G) -> InitExtra {
        let mut contents: String = Arbitrary::arbitrary(gen);
        while contents.contains("</init>") {
            contents = Arbitrary::arbitrary(gen);
        }
        let contents = contents.trim().to_string();
        InitExtra(contents)
    }
}

/// Extra event information, as a string
///
/// Any additional event information is contained in
/// `EventExtra` as a string, with leading and trailing whitespace
/// removed.
/// If no additional information is included in the file, the string is
/// empty.
///
/// # Examples
///
/// ```rust
/// use lhef::{Particle, ReadLhe};
/// use lhef::string::{LheFile, EventExtra};
///
/// let bytes = b"\
/// <LesHouchesEvents version=\"1.0\">
/// <init>
/// 2212 2212 6500 6500 0 0 13100 13100 3 1
/// 2.1 3.2E-03 1.0E+00 1
/// </init>
/// <event>
/// 4 1 +1.04e-01 1.00e+03 7.54e-03 8.68e-02
/// -11 -1 0 0 0 0 +0.00e+00 +0.00e+00 +5.00e+02 5.00e+02 0.00e+00 0.00e+00 -1.00e+00
///  11 -1 0 0 0 0 -0.00e+00 -0.00e+00 -5.00e+02 5.00e+02 0.00e+00 0.00e+00  1.00e+00
/// -13  1 1 2 0 0 -1.97e+02 -4.52e+02 -7.94e+01 5.00e+02 0.00e+00 0.00e+00 -1.00e+00
///  13  1 1 2 0 0 +1.97e+02 +4.52e+02 +7.94e+01 5.00e+02 0.00e+00 0.00e+00  1.00e+00
/// matrix element = 1.3e-9
/// ## Additional event information
///
/// <rwgt> 5 </rwgt>
///
/// </event>
/// </LesHouchesEvents>";
///
/// let lhe = LheFile::read_lhe(bytes).to_full_result().unwrap();
///
/// let EventExtra(ref extra) = lhe.events[0].extra;
/// assert_eq!(extra, &"\
/// matrix element = 1.3e-9
/// ## Additional event information
///
/// <rwgt> 5 </rwgt>".to_string()
/// );
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct EventExtra(pub String);

impl ReadLhe for EventExtra {
    fn read_lhe(input: &[u8]) -> nom::IResult<&[u8], EventExtra> {
        map_res!(input, take_until!("</event>"), |x| str::from_utf8(x)
            .map(|x| EventExtra(x.trim().to_string())))
    }
}

impl WriteLhe for EventExtra {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let &EventExtra(ref s) = self;
        if s.is_empty() {
            Ok(())
        } else {
            writeln!(writer, "{}", s)
        }
    }
}

#[cfg(test)]
impl Arbitrary for EventExtra {
    fn arbitrary<G: Gen>(gen: &mut G) -> EventExtra {
        let mut contents: String = Arbitrary::arbitrary(gen);
        while contents.contains("</event>") {
            contents = Arbitrary::arbitrary(gen);
        }
        let contents = contents.trim().to_string();
        EventExtra(contents)
    }
}

#[cfg(test)]
mod tests {
    use quickcheck;
    use serde_json;
    use std::fs;
    use std::str;

    use {Particle, ProcInfo, ReadLhe, WriteLhe};
    use generic::{EventGeneric, InitGeneric, LheFileGeneric};
    use lorentz_vector::LorentzVector;

    use super::*;

    type Event = EventGeneric<EventExtra>;
    type Init = InitGeneric<InitExtra>;

    macro_rules! roundtrip_qc {
        ($name:ident, $ty:ident) => {
            quickcheck! {
                fn $name(start: $ty) -> quickcheck::TestResult {
                    let mut bytes = Vec::new();
                    start.write_lhe(&mut bytes).unwrap();
                        let round = match $ty::read_lhe(&bytes).to_full_result() {
                        Ok(r) => r,
                        Err(err) => {
                            println!("{}", str::from_utf8(&bytes).unwrap());
                            panic!("Failed to read roundtrip: {:?}", err);
                        },
                    };
                    if start == round {
                        quickcheck::TestResult::passed()
                    } else {
                        println!("After: {:?}", round);
                        quickcheck::TestResult::failed()
                    }
                }
            }
        }
    }
    roundtrip_qc!(init_roundtrip_qc, Init);
    roundtrip_qc!(event_roundtrip_qc, Event);
    roundtrip_qc!(lhefile_roundtrip_qc, LheFile);

    #[test]
    fn read_comment() {
        let bytes = b"<!--
File generated with HELAC-DIPOLES
-->";
        let expected = Comment {
            comment: Some("File generated with HELAC-DIPOLES".to_string()),
        };
        let comment = Comment::read_lhe(bytes as &[u8]).to_full_result().unwrap();
        assert_eq!(comment, expected);
    }

    #[test]
    fn read_header() {
        let bytes = b"<header>
header line 1
header line 2
</header>";
        let expected = Header {
            header: Some("header line 1\nheader line 2".to_string()),
        };
        let result = Header::read_lhe(bytes as &[u8]).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_header_tags() {
        let bytes = b"\
<header>
header line 1
# header line 2
<line> header line 3</line>
</header>";
        let expected = Header {
            header: Some("header line 1\n# header line 2\n<line> header line 3</line>".to_string()),
        };
        let result = Header::read_lhe(bytes as &[u8]).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_init() {
        let bytes = b"\
<init>
1 10 3. 4. 5 6 7 8 9 2
11. 12. 13. 14
15. 16. 17. 18
# extra line 1
extra line 2
</init>";
        let expected = InitGeneric {
            beam_1_id: 1,
            beam_2_id: 10,
            beam_1_energy: 3.,
            beam_2_energy: 4.,
            beam_1_pdf_group_id: 5,
            beam_2_pdf_group_id: 6,
            beam_1_pdf_id: 7,
            beam_2_pdf_id: 8,
            weighting_strategy: 9,
            process_info: vec![
                ProcInfo {
                    xsect: 11.,
                    xsect_err: 12.,
                    maximum_weight: 13.,
                    process_id: 14,
                },
                ProcInfo {
                    xsect: 15.,
                    xsect_err: 16.,
                    maximum_weight: 17.,
                    process_id: 18,
                },
            ],
            extra: InitExtra("# extra line 1\nextra line 2".to_string()),
        };
        let result = InitGeneric::<InitExtra>::read_lhe(bytes)
            .to_full_result()
            .unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_event() {
        let bytes = b"\
<event>
2 1 3. 4. 5. 6.
7 8 9 10 11 12 13. 14. 15. 16. 17. 18. 19.
20 21 22 23 24 25 26. 27. 28. 29. 30. 31. 32.
# extra line 1
extra line 2
</event>";
        let expected = EventGeneric {
            process_id: 1,
            weight: 3.,
            scale: 4.,
            alpha_ew: 5.,
            alpha_qcd: 6.,
            particles: vec![
                Particle {
                    pdg_id: 7,
                    status: 8,
                    mother_1_id: 9,
                    mother_2_id: 10,
                    color_1: 11,
                    color_2: 12,
                    momentum: LorentzVector {
                        px: 13.,
                        py: 14.,
                        pz: 15.,
                        e: 16.,
                    },
                    mass: 17.,
                    proper_lifetime: 18.,
                    spin: 19.,
                },
                Particle {
                    pdg_id: 20,
                    status: 21,
                    mother_1_id: 22,
                    mother_2_id: 23,
                    color_1: 24,
                    color_2: 25,
                    momentum: LorentzVector {
                        px: 26.,
                        py: 27.,
                        pz: 28.,
                        e: 29.,
                    },
                    mass: 30.,
                    proper_lifetime: 31.,
                    spin: 32.,
                },
            ],
            extra: EventExtra("# extra line 1\nextra line 2".to_string()),
        };
        let result = EventGeneric::<EventExtra>::read_lhe(bytes)
            .to_full_result()
            .unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_lhefile_min_ev() {
        let bytes = b"\
<LesHouchesEvents version=\"1.0\">
<init>
52 61 54. 55. 56 57 58 59 60 0
</init>
<event>
1 1 3. 4. 5. 6.
7 8 9 10 11 12 13. 14. 15. 16. 17. 18. 19.
</event>
</LesHouchesEvents>";
        let expected = LheFileGeneric {
            version: "1.0".to_string(),
            comment: Comment { comment: None },
            header: Header { header: None },
            init: InitGeneric {
                beam_1_id: 52,
                beam_2_id: 61,
                beam_1_energy: 54.,
                beam_2_energy: 55.,
                beam_1_pdf_group_id: 56,
                beam_2_pdf_group_id: 57,
                beam_1_pdf_id: 58,
                beam_2_pdf_id: 59,
                weighting_strategy: 60,
                process_info: vec![],
                extra: InitExtra("".to_string()),
            },
            events: vec![
                EventGeneric {
                    process_id: 1,
                    weight: 3.,
                    scale: 4.,
                    alpha_ew: 5.,
                    alpha_qcd: 6.,
                    particles: vec![
                        Particle {
                            pdg_id: 7,
                            status: 8,
                            mother_1_id: 9,
                            mother_2_id: 10,
                            color_1: 11,
                            color_2: 12,
                            momentum: LorentzVector {
                                px: 13.,
                                py: 14.,
                                pz: 15.,
                                e: 16.,
                            },
                            mass: 17.,
                            proper_lifetime: 18.,
                            spin: 19.,
                        },
                    ],
                    extra: EventExtra("".to_string()),
                },
            ],
        };
        let result = LheFile::read_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_lhefile_min_co() {
        let bytes = b"\
<LesHouchesEvents version=\"1.0\">
<!--
Generated using numbers
-->
<init>
52 61 54. 55. 56 57 58 59 60 0
</init>
</LesHouchesEvents>";
        let expected = LheFileGeneric {
            version: "1.0".to_string(),
            comment: Comment {
                comment: Some("Generated using numbers".to_string()),
            },
            header: Header { header: None },
            init: InitGeneric {
                beam_1_id: 52,
                beam_2_id: 61,
                beam_1_energy: 54.,
                beam_2_energy: 55.,
                beam_1_pdf_group_id: 56,
                beam_2_pdf_group_id: 57,
                beam_1_pdf_id: 58,
                beam_2_pdf_id: 59,
                weighting_strategy: 60,
                process_info: vec![],
                extra: InitExtra("".to_string()),
            },
            events: vec![],
        };
        let result = LheFile::read_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_lhefile_min_head() {
        let bytes = b"\
<LesHouchesEvents version=\"1.0\">
<init>
52 61 54. 55. 56 57 58 59 60 0
</init>
<header>
header line 1
# header line 2
<line> header line 3</line>
</header>
</LesHouchesEvents>";
        let expected = LheFileGeneric {
            version: "1.0".to_string(),
            comment: Comment { comment: None },
            header: Header {
                header: Some(
                    "header line 1\n# header line 2\n<line> header line 3</line>".to_string(),
                ),
            },
            init: InitGeneric {
                beam_1_id: 52,
                beam_2_id: 61,
                beam_1_energy: 54.,
                beam_2_energy: 55.,
                beam_1_pdf_group_id: 56,
                beam_2_pdf_group_id: 57,
                beam_1_pdf_id: 58,
                beam_2_pdf_id: 59,
                weighting_strategy: 60,
                process_info: vec![],
                extra: InitExtra("".to_string()),
            },
            events: vec![],
        };
        let result = LheFile::read_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_lhefile_min_co_head() {
        let bytes = b"\
<LesHouchesEvents version=\"1.0\">
<!--
Generated using numbers
-->
<header>
header line 1
# header line 2
<line> header line 3</line>
</header>
<init>
52 61 54. 55. 56 57 58 59 60 0
</init>
</LesHouchesEvents>";
        let expected = LheFileGeneric {
            version: "1.0".to_string(),
            comment: Comment {
                comment: Some("Generated using numbers".to_string()),
            },
            header: Header {
                header: Some(
                    "header line 1\n# header line 2\n<line> header line 3</line>".to_string(),
                ),
            },
            init: InitGeneric {
                beam_1_id: 52,
                beam_2_id: 61,
                beam_1_energy: 54.,
                beam_2_energy: 55.,
                beam_1_pdf_group_id: 56,
                beam_2_pdf_group_id: 57,
                beam_1_pdf_id: 58,
                beam_2_pdf_id: 59,
                weighting_strategy: 60,
                process_info: vec![],
                extra: InitExtra("".to_string()),
            },
            events: vec![],
        };
        let result = LheFile::read_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn qc_regression() {
        let lhe = LheFileGeneric {
            version: "".to_string(),
            comment: Comment { comment: Some(".\u{9b}{£->\u{13}\u{fff2}\n>w蓲*খ?,®xy\u{79994}\n3\u{11cdd}R_\u{84}E{\"\u{7f}\u{92}\u{0}\u{3000}|\u{82}㖬⁛ٺ$![\u{88}8?\u{10f0be}\u{e56c}¬)\u{70f}\u{9a}\u{7}\u{2005}*\u{2006}\u{9cf}퐢\"\u{e922}\n䰝\u{2}‖\n^@4#7ª\u{1d}‶9\u{fff8}}!뇝\u{92081}*\u{81}颱RA3S )%\u{81}\u{48377}\u{10ffff}".to_string())},
            header: Header { header: Some("⁂\u{9c}\u{8d}\u{1d}\u{97}\u{1c}(7«8\u{17}g\u{8e}\u{a0}娐S\u{0}?\u{89}eª⁒h𪄡u\u{f}\u{ffff}4\u{1b}\u{10}\u{d278e}\u{80}§<麡B\u{13}ªG£5揬\\톤e\u{d3677}@&\u{b}?\u{7}⁒H,\u{92}뜒t}6]\u{16}`\u{9a}ⴑ¦¡V\u{84}_]\u{91}«}2(%7X~\u{86}癪\u{16}\u{feeb9}\u{91}{u\u{206a}$]+-X).Aq[\u{f}".to_string()) },
            init: InitGeneric { beam_1_id: -17, beam_2_id: -95, beam_1_energy: -61.50434190590901, beam_2_energy: -84.36896434784065, beam_1_pdf_group_id: 15, beam_2_pdf_group_id: -91, beam_1_pdf_id: -58, beam_2_pdf_id: 84, weighting_strategy: -89, process_info: vec![ProcInfo { xsect: 3.0340493268133315, xsect_err: 73.39946519830431, maximum_weight: -70.69361722451761, process_id: -96 }], extra: InitExtra("p\n餻M⁊#𐫚&\u{84} 쒵\u{8d}a\u{8a}\\\u{2061}\u{3}8>횮\n$\u{99}\u{1c}\u{1b}:[\u{9e}פ#\u{206f}?2\u{91}\t£(+&b[\u{10715a}\u{70019}\u{17}\u{65953}&".to_string()) },
            events: vec![],
        };
        let mut bytes = Vec::new();
        lhe.write_lhe(&mut bytes).unwrap();
        let round = match LheFile::read_lhe(&bytes).to_full_result() {
            Ok(l) => l,
            Err(e) => panic!(
                "Failed to read roundtrip: {:?}\n{}",
                e,
                str::from_utf8(&bytes).unwrap()
            ),
        };
        if lhe != round {
            assert_eq!(lhe, round);
        }
    }

    #[test]
    fn read_lhefile() {
        let bytes = b"\
<LesHouchesEvents version=\"1.0\">
<!--
Generated using numbers
-->
<init>
52 61 54. 55. 56 57 58 59 60 2
62. 63. 64. 65
66. 67. 68. 69
# init extra ?
</init>
<header>
header line 1
# header line 2
<line> header line 3</line>
</header>
<event>
2 1 3. 4. 5. 6.
7 8 9 10 11 12 13. 14. 15. 16. 17. 18. 19.
20 21 22 23 24 25 26. 27. 28. 29. 30. 31. 32.
# extra 1
2nd extra
</event>
<event>
1 33 35. 36. 37. 38.
39 40 41 42 43 44 45. 46. 47. 48. 49. 50. 51.
52 53 54 55 56 57 58. 59. 60. 61. 62. 63. 64.
</event>
</LesHouchesEvents>";
        let expected = LheFileGeneric {
            version: "1.0".to_string(),
            comment: Comment {
                comment: Some("Generated using numbers".to_string()),
            },
            header: Header {
                header: Some(
                    "header line 1\n# header line 2\n<line> header line 3</line>".to_string(),
                ),
            },
            init: InitGeneric {
                beam_1_id: 52,
                beam_2_id: 61,
                beam_1_energy: 54.,
                beam_2_energy: 55.,
                beam_1_pdf_group_id: 56,
                beam_2_pdf_group_id: 57,
                beam_1_pdf_id: 58,
                beam_2_pdf_id: 59,
                weighting_strategy: 60,
                process_info: vec![
                    ProcInfo {
                        xsect: 62.,
                        xsect_err: 63.,
                        maximum_weight: 64.,
                        process_id: 65,
                    },
                    ProcInfo {
                        xsect: 66.,
                        xsect_err: 67.,
                        maximum_weight: 68.,
                        process_id: 69,
                    },
                ],
                extra: InitExtra("# init extra ?".to_string()),
            },
            events: vec![
                EventGeneric {
                    process_id: 1,
                    weight: 3.,
                    scale: 4.,
                    alpha_ew: 5.,
                    alpha_qcd: 6.,
                    particles: vec![
                        Particle {
                            pdg_id: 7,
                            status: 8,
                            mother_1_id: 9,
                            mother_2_id: 10,
                            color_1: 11,
                            color_2: 12,
                            momentum: LorentzVector {
                                px: 13.,
                                py: 14.,
                                pz: 15.,
                                e: 16.,
                            },
                            mass: 17.,
                            proper_lifetime: 18.,
                            spin: 19.,
                        },
                        Particle {
                            pdg_id: 20,
                            status: 21,
                            mother_1_id: 22,
                            mother_2_id: 23,
                            color_1: 24,
                            color_2: 25,
                            momentum: LorentzVector {
                                px: 26.,
                                py: 27.,
                                pz: 28.,
                                e: 29.,
                            },
                            mass: 30.,
                            proper_lifetime: 31.,
                            spin: 32.,
                        },
                    ],
                    extra: EventExtra("# extra 1\n2nd extra".to_string()),
                },
                EventGeneric {
                    process_id: 33,
                    weight: 35.,
                    scale: 36.,
                    alpha_ew: 37.,
                    alpha_qcd: 38.,
                    particles: vec![
                        Particle {
                            pdg_id: 39,
                            status: 40,
                            mother_1_id: 41,
                            mother_2_id: 42,
                            color_1: 43,
                            color_2: 44,
                            momentum: LorentzVector {
                                px: 45.,
                                py: 46.,
                                pz: 47.,
                                e: 48.,
                            },
                            mass: 49.,
                            proper_lifetime: 50.,
                            spin: 51.,
                        },
                    ],
                    extra: EventExtra("52 53 54 55 56 57 58. 59. 60. 61. 62. 63. 64.".to_string()),
                },
            ],
        };
        let result = LheFile::read_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_lhefile_reverse() {
        let bytes = b"\
<LesHouchesEvents version=\"1.0\">
<header>
header line 1
# header line 2
<line> header line 3</line>
</header>
<init>
52 61 54. 55. 56 57 58 59 60 2
62. 63. 64. 65
66. 67. 68. 69
# init extra ?
</init>
<!--
Generated using numbers
-->
<event>
2 1 3. 4. 5. 6.
7 8 9 10 11 12 13. 14. 15. 16. 17. 18. 19.
20 21 22 23 24 25 26. 27. 28. 29. 30. 31. 32.
# extra 1
2nd extra
</event>
<event>
1 33 35. 36. 37. 38.
39 40 41 42 43 44 45. 46. 47. 48. 49. 50. 51.
52 53 54 55 56 57 58. 59. 60. 61. 62. 63. 64.
</event>
</LesHouchesEvents>";
        let expected = LheFileGeneric {
            version: "1.0".to_string(),
            comment: Comment {
                comment: Some("Generated using numbers".to_string()),
            },
            header: Header {
                header: Some(
                    "header line 1\n# header line 2\n<line> header line 3</line>".to_string(),
                ),
            },
            init: InitGeneric {
                beam_1_id: 52,
                beam_2_id: 61,
                beam_1_energy: 54.,
                beam_2_energy: 55.,
                beam_1_pdf_group_id: 56,
                beam_2_pdf_group_id: 57,
                beam_1_pdf_id: 58,
                beam_2_pdf_id: 59,
                weighting_strategy: 60,
                process_info: vec![
                    ProcInfo {
                        xsect: 62.,
                        xsect_err: 63.,
                        maximum_weight: 64.,
                        process_id: 65,
                    },
                    ProcInfo {
                        xsect: 66.,
                        xsect_err: 67.,
                        maximum_weight: 68.,
                        process_id: 69,
                    },
                ],
                extra: InitExtra("# init extra ?".to_string()),
            },
            events: vec![
                EventGeneric {
                    process_id: 1,
                    weight: 3.,
                    scale: 4.,
                    alpha_ew: 5.,
                    alpha_qcd: 6.,
                    particles: vec![
                        Particle {
                            pdg_id: 7,
                            status: 8,
                            mother_1_id: 9,
                            mother_2_id: 10,
                            color_1: 11,
                            color_2: 12,
                            momentum: LorentzVector {
                                px: 13.,
                                py: 14.,
                                pz: 15.,
                                e: 16.,
                            },
                            mass: 17.,
                            proper_lifetime: 18.,
                            spin: 19.,
                        },
                        Particle {
                            pdg_id: 20,
                            status: 21,
                            mother_1_id: 22,
                            mother_2_id: 23,
                            color_1: 24,
                            color_2: 25,
                            momentum: LorentzVector {
                                px: 26.,
                                py: 27.,
                                pz: 28.,
                                e: 29.,
                            },
                            mass: 30.,
                            proper_lifetime: 31.,
                            spin: 32.,
                        },
                    ],
                    extra: EventExtra("# extra 1\n2nd extra".to_string()),
                },
                EventGeneric {
                    process_id: 33,
                    weight: 35.,
                    scale: 36.,
                    alpha_ew: 37.,
                    alpha_qcd: 38.,
                    particles: vec![
                        Particle {
                            pdg_id: 39,
                            status: 40,
                            mother_1_id: 41,
                            mother_2_id: 42,
                            color_1: 43,
                            color_2: 44,
                            momentum: LorentzVector {
                                px: 45.,
                                py: 46.,
                                pz: 47.,
                                e: 48.,
                            },
                            mass: 49.,
                            proper_lifetime: 50.,
                            spin: 51.,
                        },
                    ],
                    extra: EventExtra("52 53 54 55 56 57 58. 59. 60. 61. 62. 63. 64.".to_string()),
                },
            ],
        };
        let result = LheFile::read_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    const SAMPLE_FILES: &'static [&'static str] = &[
        "tests/real_world_files/helac_1loop_tree.lhe",
        "tests/real_world_files/helac_1loop_virt.lhe",
        "tests/real_world_files/helac_dipoles_i.lhe",
        "tests/real_world_files/helac_dipoles_kp.lhe",
        "tests/real_world_files/helac_dipoles_rs.lhe",
        "tests/real_world_files/mg5_aMC.lhe",
        "tests/real_world_files/mg5_aMC_NLO.lhe",
        "tests/real_world_files/mg5_aMC_NLO_rwgt.lhe",
    ];

    #[test]
    fn read_sample_files() {
        for file_name in SAMPLE_FILES {
            if let Err(e) = LheFile::read_lhe_from_file(file_name) {
                panic!("Failed to read file {}: {:?}", file_name, e);
            }
        }
    }

    #[test]
    fn roundtrip_sample_files() {
        for file_name in SAMPLE_FILES {
            let lhe = match LheFile::read_lhe_from_file(file_name) {
                Ok(l) => l,
                Err(e) => panic!("Failed to read: {:?}", e),
            };

            let mut bytes = Vec::new();
            lhe.write_lhe(&mut bytes).unwrap();
            let round = match LheFile::read_lhe(&bytes).to_full_result() {
                Ok(l) => l,
                Err(e) => panic!("Failed to read roundtrip for {}: {:?}", file_name, e),
            };
            if lhe != round {
                println!("Failure in {}:", file_name);
                assert_eq!(lhe, round);
            }
        }
    }

    #[test]
    fn validate_sample_files() {
        for file_name in SAMPLE_FILES {
            use std::path::Path;
            let lhe = LheFile::read_lhe_from_file(file_name).unwrap();
            let json = Path::new(file_name);
            let base = json.file_stem().unwrap();
            let json = json.with_file_name(format!("{}_string.json", base.to_str().unwrap()));
            let mut file = fs::File::open(&json).unwrap();
            let valid: LheFile = match serde_json::from_reader(&mut file) {
                Ok(v) => v,
                Err(e) => panic!("BUG: Error reading json for '{:?}': {:?}", json, e),
            };
            if lhe != valid {
                println!("Failure in {}:", file_name);
                assert_eq!(lhe, valid);
            }
        }
    }
}
