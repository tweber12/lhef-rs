// Copyright 2018 Torsten Weber
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use {Particle, PdgId, ProcInfo, ReadLhe, WriteLhe};
use nom_util::{parse_f64, parse_i64, parse_u64};

use nom;
use std::io;
use std::str;

#[cfg(test)]
use quickcheck::Arbitrary;
#[cfg(test)]
use quickcheck::Gen;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct LheFileGeneric<Comment, Header, InitExtra, EventExtra> {
    pub version: String,
    pub comment: Comment,
    pub header: Header,
    pub init: InitGeneric<InitExtra>,
    pub events: Vec<EventGeneric<EventExtra>>,
}

impl<Comment, Header, InitExtra, EventExtra> ReadLhe
    for LheFileGeneric<Comment, Header, InitExtra, EventExtra>
where
    Comment: ReadLhe + PartialEq,
    Header: ReadLhe + PartialEq,
    InitExtra: ReadLhe + PartialEq,
    EventExtra: ReadLhe,
{
    fn read_from_lhe(
        input: &[u8],
    ) -> nom::IResult<&[u8], LheFileGeneric<Comment, Header, InitExtra, EventExtra>> {
        do_parse!(
            input,
            ws!(tag!("<LesHouchesEvents")) >> ws!(tag!("version=")) >> tag!("\"")
                >> version:
                    map_res!(take_until!("\""), |x| str::from_utf8(x)
                        .map(|x| x.to_string())) >> tag!("\"") >> ws!(tag!(">"))
                >> hi:
                    dbg!(permutation!(
                        ws!(Comment::read_from_lhe),
                        ws!(Header::read_from_lhe),
                        ws!(InitGeneric::read_from_lhe)
                    )) >> events: dbg!(many0!(EventGeneric::read_from_lhe))
                >> dbg!(ws!(tag!("</LesHouchesEvents>"))) >> (LheFileGeneric {
                version,
                comment: hi.0,
                header: hi.1,
                init: hi.2,
                events,
            })
        )
    }
}

impl<Comment, Header, InitExtra, EventExtra> WriteLhe
    for LheFileGeneric<Comment, Header, InitExtra, EventExtra>
where
    Comment: WriteLhe,
    Header: WriteLhe,
    InitExtra: WriteLhe,
    EventExtra: WriteLhe,
{
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        write_opening_file_tag(writer, &self.version)?;
        self.comment.write_lhe(writer)?;
        self.header.write_lhe(writer)?;
        self.init.write_lhe(writer)?;
        for event in &self.events {
            event.write_lhe(writer)?;
        }
        write_closing_file_tag(writer)
    }
}

#[cfg(test)]
impl<Header, Comment, InitExtra, EventExtra> Arbitrary
    for LheFileGeneric<Header, Comment, InitExtra, EventExtra>
where
    Header: Arbitrary,
    Comment: Arbitrary,
    InitExtra: Arbitrary,
    EventExtra: Arbitrary,
{
    fn arbitrary<G: Gen>(gen: &mut G) -> LheFileGeneric<Header, Comment, InitExtra, EventExtra> {
        let mut version: String = Arbitrary::arbitrary(gen);
        while version.contains("\"") {
            version = Arbitrary::arbitrary(gen);
        }
        LheFileGeneric {
            version,
            comment: Arbitrary::arbitrary(gen),
            header: Arbitrary::arbitrary(gen),
            init: Arbitrary::arbitrary(gen),
            events: Arbitrary::arbitrary(gen),
        }
    }

    fn shrink(
        &self,
    ) -> Box<Iterator<Item = LheFileGeneric<Header, Comment, InitExtra, EventExtra>>> {
        let version = self.version.clone();
        let comment = self.comment.clone();
        let header = self.header.clone();
        let init = self.init.clone();
        let tup = (version, self.events.clone());
        let iter = tup.shrink().map(move |x| LheFileGeneric {
            version: x.0,
            comment: comment.clone(),
            header: header.clone(),
            init: init.clone(),
            events: x.1,
        });
        Box::new(iter)
    }
}

fn write_opening_file_tag<W: io::Write>(writer: &mut W, version: &str) -> io::Result<()> {
    writeln!(writer, "<LesHouchesEvents version=\"{}\">", version)
}
fn write_closing_file_tag<W: io::Write>(writer: &mut W) -> io::Result<()> {
    writeln!(writer, "</LesHouchesEvents>")
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct InitGeneric<InitExtra> {
    pub beam_1_id: PdgId,
    pub beam_2_id: PdgId,
    pub beam_1_energy: f64,
    pub beam_2_energy: f64,
    pub beam_1_pdf_group_id: i64,
    pub beam_2_pdf_group_id: i64,
    pub beam_1_pdf_id: i64,
    pub beam_2_pdf_id: i64,
    pub weighting_strategy: i64,
    pub process_info: Vec<ProcInfo>,
    pub extra: InitExtra,
}

impl<InitExtra> ReadLhe for InitGeneric<InitExtra>
where
    InitExtra: ReadLhe,
{
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], InitGeneric<InitExtra>> {
        do_parse!(
            input,
            ws!(tag!("<init>")) >> beam_1_id: ws!(parse_i64) >> beam_2_id: ws!(parse_i64)
                >> beam_1_energy: ws!(parse_f64) >> beam_2_energy: ws!(parse_f64)
                >> beam_1_pdf_group_id: ws!(parse_i64)
                >> beam_2_pdf_group_id: ws!(parse_i64) >> beam_1_pdf_id: ws!(parse_i64)
                >> beam_2_pdf_id: ws!(parse_i64) >> weighting_strategy: ws!(parse_i64)
                >> n_processes: ws!(parse_u64)
                >> process_info: count!(ws!(ProcInfo::read_from_lhe), n_processes as usize)
                >> extra: ws!(InitExtra::read_from_lhe) >> ws!(tag!("</init>"))
                >> (InitGeneric {
                    beam_1_id,
                    beam_2_id,
                    beam_1_energy,
                    beam_2_energy,
                    beam_1_pdf_group_id,
                    beam_2_pdf_group_id,
                    beam_1_pdf_id,
                    beam_2_pdf_id,
                    weighting_strategy,
                    process_info,
                    extra,
                })
        )
    }
}

impl<InitExtra> WriteLhe for InitGeneric<InitExtra>
where
    InitExtra: WriteLhe,
{
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "<init>")?;
        writeln!(
            writer,
            "{} {} {:e} {:e} {} {} {} {} {} {}",
            self.beam_1_id,
            self.beam_2_id,
            self.beam_1_energy,
            self.beam_2_energy,
            self.beam_1_pdf_group_id,
            self.beam_2_pdf_group_id,
            self.beam_1_pdf_id,
            self.beam_2_pdf_id,
            self.weighting_strategy,
            self.process_info.len()
        )?;
        for p in &self.process_info {
            p.write_lhe(writer)?;
        }
        self.extra.write_lhe(writer)?;
        writeln!(writer, "</init>")
    }
}

#[cfg(test)]
impl<InitExtra> Arbitrary for InitGeneric<InitExtra>
where
    InitExtra: Arbitrary + Clone,
{
    fn arbitrary<G: Gen>(gen: &mut G) -> InitGeneric<InitExtra> {
        let process_info: Vec<ProcInfo> = Arbitrary::arbitrary(gen);
        InitGeneric {
            beam_1_id: Arbitrary::arbitrary(gen),
            beam_2_id: Arbitrary::arbitrary(gen),
            beam_1_energy: Arbitrary::arbitrary(gen),
            beam_2_energy: Arbitrary::arbitrary(gen),
            beam_1_pdf_group_id: Arbitrary::arbitrary(gen),
            beam_2_pdf_group_id: Arbitrary::arbitrary(gen),
            beam_1_pdf_id: Arbitrary::arbitrary(gen),
            beam_2_pdf_id: Arbitrary::arbitrary(gen),
            weighting_strategy: Arbitrary::arbitrary(gen),
            process_info,
            extra: Arbitrary::arbitrary(gen),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct EventGeneric<EventExtra> {
    pub process_id: i64,
    pub weight: f64,
    pub scale: f64,
    pub alpha_ew: f64,
    pub alpha_qcd: f64,
    pub particles: Vec<Particle>,
    pub extra: EventExtra,
}

impl<EventExtra> ReadLhe for EventGeneric<EventExtra>
where
    EventExtra: ReadLhe,
{
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], EventGeneric<EventExtra>> {
        do_parse!(
            input,
            ws!(tag!("<event>")) >> n_particles: ws!(parse_i64) >> process_id: ws!(parse_i64)
                >> weight: ws!(parse_f64) >> scale: ws!(parse_f64)
                >> alpha_ew: ws!(parse_f64) >> alpha_qcd: ws!(parse_f64)
                >> particles: count!(Particle::read_from_lhe, n_particles as usize)
                >> extra: ws!(EventExtra::read_from_lhe) >> ws!(tag!("</event>"))
                >> (EventGeneric {
                    process_id,
                    weight,
                    scale,
                    alpha_ew,
                    alpha_qcd,
                    particles,
                    extra,
                })
        )
    }
}

impl<EventExtra> WriteLhe for EventGeneric<EventExtra>
where
    EventExtra: WriteLhe,
{
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "<event>")?;
        writeln!(
            writer,
            "{} {} {:e} {:e} {:e} {:e}",
            self.particles.len(),
            self.process_id,
            self.weight,
            self.scale,
            self.alpha_ew,
            self.alpha_qcd
        )?;
        for particle in &self.particles {
            particle.write_lhe(writer)?;
        }
        self.extra.write_lhe(writer)?;
        writeln!(writer, "</event>")
    }
}

#[cfg(test)]
impl<EventExtra> Arbitrary for EventGeneric<EventExtra>
where
    EventExtra: Arbitrary,
{
    fn arbitrary<G: Gen>(gen: &mut G) -> EventGeneric<EventExtra> {
        EventGeneric {
            process_id: Arbitrary::arbitrary(gen),
            weight: Arbitrary::arbitrary(gen),
            scale: Arbitrary::arbitrary(gen),
            alpha_ew: Arbitrary::arbitrary(gen),
            alpha_qcd: Arbitrary::arbitrary(gen),
            particles: Arbitrary::arbitrary(gen),
            extra: Arbitrary::arbitrary(gen),
        }
    }

    fn shrink(&self) -> Box<Iterator<Item = EventGeneric<EventExtra>>> {
        let tup = (
            self.process_id,
            self.weight,
            self.scale,
            self.alpha_ew,
            self.alpha_qcd,
            self.particles.clone(),
            self.extra.clone(),
        );
        let iter = tup.shrink().map(|x| EventGeneric {
            process_id: x.0,
            weight: x.1,
            scale: x.2,
            alpha_ew: x.3,
            alpha_qcd: x.4,
            particles: x.5,
            extra: x.6,
        });
        Box::new(iter)
    }
}

#[cfg(test)]
mod tests {
    use nom;
    use quickcheck;
    use std::{io, str};

    use {Particle, ProcInfo, ReadLhe, WriteLhe};
    use lorentz_vector::LorentzVector;

    use super::{EventGeneric, InitGeneric, LheFileGeneric};

    #[derive(Clone, Debug, PartialEq)]
    struct Nothing {}
    impl ReadLhe for Nothing {
        fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], Nothing> {
            nom::IResult::Done(input, Nothing {})
        }
    }
    impl WriteLhe for Nothing {
        fn write_lhe<W: io::Write>(&self, _writer: &mut W) -> io::Result<()> {
            Ok(())
        }
    }
    impl quickcheck::Arbitrary for Nothing {
        fn arbitrary<G: quickcheck::Gen>(_gen: &mut G) -> Nothing {
            Nothing {}
        }
    }

    #[test]
    fn read_init() {
        let bytes = b"\
<init>
1 10 3. 4. 5 6 7 8 9 2
11. 12. 13. 14
15. 16. 17. 18
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
            extra: Nothing {},
        };
        let result = InitGeneric::<Nothing>::read_from_lhe(bytes)
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
            extra: Nothing {},
        };
        let result = EventGeneric::<Nothing>::read_from_lhe(bytes)
            .to_full_result()
            .unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_lhefile() {
        let bytes = b"\
<LesHouchesEvents version=\"1.0\">
<init>
52 61 54. 55. 56 57 58 59 60 2
62. 63. 64. 65
66. 67. 68. 69
</init>
<event>
2 1 3. 4. 5. 6.
7 8 9 10 11 12 13. 14. 15. 16. 17. 18. 19.
20 21 22 23 24 25 26. 27. 28. 29. 30. 31. 32.
</event>
<event>
1 33 35. 36. 37. 38.
39 40 41 42 43 44 45. 46. 47. 48. 49. 50. 51.
</event>
</LesHouchesEvents>";
        let expected = LheFileGeneric {
            version: "1.0".to_string(),
            comment: Nothing {},
            header: Nothing {},
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
                extra: Nothing {},
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
                    extra: Nothing {},
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
                    extra: Nothing {},
                },
            ],
        };
        let result = LheFileGeneric::<Nothing, Nothing, Nothing, Nothing>::read_from_lhe(bytes)
            .to_full_result()
            .unwrap();
        assert_eq!(result, expected);
    }

    quickcheck! {
        fn nothing_roundtrip_qc(m: Nothing) -> bool {
            let mut bytes = Vec::new();
            m.write_lhe(&mut bytes).unwrap();
            let round = match Nothing::read_from_lhe(&bytes).to_full_result() {
                Ok(r) => r,
                Err(err) => {
                    panic!("Failed to read roundtrip: {:?}", err);
                },
            };
            m == round
        }
    }

    quickcheck! {
        fn event_roundtrip_qc(start: EventGeneric<Nothing>) -> bool {
            let mut bytes = Vec::new();
            start.write_lhe(&mut bytes).unwrap();
            let round = match EventGeneric::<Nothing>::read_from_lhe(&bytes).to_full_result() {
                Ok(r) => r,
                Err(err) => panic!("Failed to read roundtrip: {:?}", err),
            };
            start == round
        }
    }

    quickcheck! {
        fn init_roundtrip_qc(start: InitGeneric<Nothing>) -> bool {
            let mut bytes = Vec::new();
            start.write_lhe(&mut bytes).unwrap();
            let round = match InitGeneric::<Nothing>::read_from_lhe(&bytes).to_full_result() {
                Ok(r) => r,
                Err(err) => panic!("Failed to read roundtrip: {:?}", err),
            };
            start == round
        }
    }

    #[test]
    fn lhefile_roundtrip_simple() {
        let version = "";
        let init = InitGeneric {
            beam_1_id: 1,
            beam_2_id: 2,
            beam_1_pdf_group_id: 3,
            beam_2_pdf_group_id: 4,
            beam_1_pdf_id: 5,
            beam_2_pdf_id: 6,
            weighting_strategy: 7,
            beam_1_energy: 8.,
            beam_2_energy: 9.,
            process_info: Vec::new(),
            extra: Nothing {},
        };
        let start = LheFileGeneric {
            version: version.to_string(),
            comment: Nothing {},
            header: Nothing {},
            init,
            events: Vec::new(),
        };
        let mut bytes = Vec::new();
        start.write_lhe(&mut bytes).unwrap();
        let round = match LheFileGeneric::<Nothing, Nothing, Nothing, Nothing>::read_from_lhe(
            &bytes,
        ).to_full_result()
        {
            Ok(r) => r,
            Err(err) => {
                println!("{}", str::from_utf8(&bytes).unwrap());
                panic!("Failed to read roundtrip: {:?}", err);
            }
        };
        assert_eq!(start, round);
    }

    #[test]
    fn lhefile_roundtrip_simple_version() {
        let version = "1.0";
        let init = InitGeneric {
            beam_1_id: 1,
            beam_2_id: 2,
            beam_1_pdf_group_id: 3,
            beam_2_pdf_group_id: 4,
            beam_1_pdf_id: 5,
            beam_2_pdf_id: 6,
            weighting_strategy: 7,
            beam_1_energy: 8.,
            beam_2_energy: 9.,
            process_info: Vec::new(),
            extra: Nothing {},
        };
        let start = LheFileGeneric {
            version: version.to_string(),
            comment: Nothing {},
            header: Nothing {},
            init,
            events: Vec::new(),
        };
        let mut bytes = Vec::new();
        start.write_lhe(&mut bytes).unwrap();
        let round = match LheFileGeneric::<Nothing, Nothing, Nothing, Nothing>::read_from_lhe(
            &bytes,
        ).to_full_result()
        {
            Ok(r) => r,
            Err(err) => {
                println!("{}", str::from_utf8(&bytes).unwrap());
                panic!("Failed to read roundtrip: {:?}", err);
            }
        };
        assert_eq!(start, round);
    }

    quickcheck! {
        fn lhefile_roundtrip_noevents_qc(version: String, init: InitGeneric<Nothing>) -> quickcheck::TestResult {
            if version.contains('"') {
                return quickcheck::TestResult::discard();
            }
            let start = LheFileGeneric {
                version,
                comment: Nothing {},
                header: Nothing {},
                init,
                events: Vec::new(),
            };
            let mut bytes = Vec::new();
            start.write_lhe(&mut bytes).unwrap();
                let round = match LheFileGeneric::<Nothing, Nothing, Nothing, Nothing>::read_from_lhe(&bytes).to_full_result() {
                Ok(r) => r,
                Err(err) => {
                    println!("{}", str::from_utf8(&bytes).unwrap());
                    panic!("Failed to read roundtrip: {:?}", err);
                },
            };
            quickcheck::TestResult::from_bool(start == round)
        }
    }

    quickcheck! {
        fn lhefile_roundtrip_qc(start: LheFileGeneric<Nothing, Nothing, Nothing, Nothing>) -> quickcheck::TestResult {
            let mut bytes = Vec::new();
            start.write_lhe(&mut bytes).unwrap();
            let round = match LheFileGeneric::<Nothing, Nothing, Nothing, Nothing>::read_from_lhe(&bytes).to_full_result() {
                Ok(r) => r,
                Err(err) => {
                    println!("{}", str::from_utf8(&bytes).unwrap());
                    panic!("Failed to read roundtrip: {:?}", err);
                },
            };
            quickcheck::TestResult::from_bool(start == round)
        }
    }
}
