// Copyright 2018 Torsten Weber
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A module to read and write lhe files and ignore all extra information
//!
//! This module contains types that can be used in `LheFileGeneric` to
//! read lhe files and ignore all additional information that might be
//! contained in the files.

use {ReadLhe, WriteLhe};
use generic::LheFileGeneric;

use nom;
use std::io;

#[cfg(test)]
use quickcheck::Arbitrary;
#[cfg(test)]
use quickcheck::Gen;

/// A type to read and write lhe files and ignore all extra information
pub type LheFile = LheFileGeneric<Comment, Header, InitExtra, EventExtra>;

/// A dummy comment
///
/// This type can read an optional comment from an lhe file, but throws
/// away the contents.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct Comment {}

impl ReadLhe for Comment {
    fn read_lhe(input: &[u8]) -> nom::IResult<&[u8], Comment> {
        do_parse!(
            input,
            opt!(delimited!(tag!("<!--"), take_until!("-->"), tag!("-->"))) >> (Comment {})
        )
    }
}

impl WriteLhe for Comment {
    fn write_lhe<W: io::Write>(&self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
impl Arbitrary for Comment {
    fn arbitrary<G: Gen>(_gen: &mut G) -> Comment {
        Comment {}
    }
}

/// A dummy lhe file header
///
/// This type can read an optional header from an lhe file, but throws
/// away the contents.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct Header {}

impl ReadLhe for Header {
    fn read_lhe(input: &[u8]) -> nom::IResult<&[u8], Header> {
        do_parse!(
            input,
            opt!(delimited!(
                tag!("<header>"),
                take_until!("</header>"),
                tag!("</header>")
            )) >> (Header {})
        )
    }
}

impl WriteLhe for Header {
    fn write_lhe<W: io::Write>(&self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
impl Arbitrary for Header {
    fn arbitrary<G: Gen>(_gen: &mut G) -> Header {
        Header {}
    }
}

/// Dummy initialization information
///
/// This type can parse additional initialization that may be present
/// in the init section of an lhe file, but throws away the contents.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct InitExtra {}

impl ReadLhe for InitExtra {
    fn read_lhe(input: &[u8]) -> nom::IResult<&[u8], InitExtra> {
        do_parse!(input, take_until!("</init>") >> (InitExtra {}))
    }
}

impl WriteLhe for InitExtra {
    fn write_lhe<W: io::Write>(&self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
impl Arbitrary for InitExtra {
    fn arbitrary<G: Gen>(_gen: &mut G) -> InitExtra {
        InitExtra {}
    }
}

/// Dummy event information
///
/// This type can parse additional event information that may be present
/// in the events of an lhe file, but throws away the contents.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct EventExtra {}

impl ReadLhe for EventExtra {
    fn read_lhe(input: &[u8]) -> nom::IResult<&[u8], EventExtra> {
        do_parse!(input, take_until!("</event>") >> (EventExtra {}))
    }
}

impl WriteLhe for EventExtra {
    fn write_lhe<W: io::Write>(&self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
impl Arbitrary for EventExtra {
    fn arbitrary<G: Gen>(_gen: &mut G) -> EventExtra {
        EventExtra {}
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
        let expected = Comment {};
        let comment = Comment::read_lhe(bytes as &[u8]).to_full_result().unwrap();
        assert_eq!(comment, expected);
    }

    #[test]
    fn read_header() {
        let bytes = b"<header>
header line 1
header line 2
</header>";
        let expected = Header {};
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
        let expected = Header {};
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
            extra: InitExtra {},
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
            extra: EventExtra {},
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
            comment: Comment {},
            header: Header {},
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
                extra: InitExtra {},
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
                    extra: EventExtra {},
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
            comment: Comment {},
            header: Header {},
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
                extra: InitExtra {},
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
            comment: Comment {},
            header: Header {},
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
                extra: InitExtra {},
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
            comment: Comment {},
            header: Header {},
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
                extra: InitExtra {},
            },
            events: vec![],
        };
        let result = LheFile::read_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
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
            comment: Comment {},
            header: Header {},
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
                extra: InitExtra {},
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
                    extra: EventExtra {},
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
                    extra: EventExtra {},
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
            comment: Comment {},
            header: Header {},
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
                extra: InitExtra {},
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
                    extra: EventExtra {},
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
                    extra: EventExtra {},
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
            let json = json.with_file_name(format!("{}_plain.json", base.to_str().unwrap()));
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
