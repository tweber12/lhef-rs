// Copyright 2018 Torsten Weber
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use {PdgId, ReadLhe, WriteLhe};
use generic::LheFileGeneric;
use nom_util::{parse_f64, parse_i64, parse_i8, parse_u64, parse_u8};

use nom;
use std::io;
use std::str;

#[cfg(test)]
use quickcheck::Arbitrary;
#[cfg(test)]
use quickcheck::Gen;

pub type LheFileRS = LheFileGeneric<Comment, Header, InitExtraRS, EventExtraRS>;
pub type LheFileI = LheFileGeneric<Comment, Header, PdfSum, EventExtraI>;
pub type LheFileKP = LheFileGeneric<Comment, Header, PdfSumKP, EventExtraKP>;
pub type LheFile1loop = LheFileGeneric<Comment, Header, InitExtra1loop, EventExtra1loop>;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct Comment(pub String);

impl ReadLhe for Comment {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], Comment> {
        map_res!(
            input,
            delimited!(tag!("<!--"), take_until!("-->"), tag!("-->")),
            |x| str::from_utf8(x).map(|x| Comment(x.trim().to_string()))
        )
    }
}

impl WriteLhe for Comment {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let &Comment(ref s) = self;
        writeln!(writer, "<!--")?;
        writeln!(writer, "{}", s)?;
        writeln!(writer, "-->")
    }
}

#[cfg(test)]
impl Arbitrary for Comment {
    fn arbitrary<G: Gen>(gen: &mut G) -> Comment {
        let mut contents: String = Arbitrary::arbitrary(gen);
        while contents.contains("-->") {
            contents = Arbitrary::arbitrary(gen);
        }
        let contents = contents.trim().to_string();
        Comment(contents)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct Header {}
impl ReadLhe for Header {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], Header> {
        nom::IResult::Done(input, Header {})
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

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct PdfSum {
    pub pdf_sum_pairs: Vec<(PdgId, PdgId)>,
}

impl ReadLhe for PdfSum {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], PdfSum> {
        do_parse!(
            input,
            ws!(tag!("#")) >> ws!(tag!("SUMPDF")) >> n: ws!(parse_u64)
                >> pdf_sum_pairs: count!(ws!(pair!(ws!(parse_i64), ws!(parse_i64))), n as usize)
                >> (PdfSum { pdf_sum_pairs })
        )
    }
}

impl WriteLhe for PdfSum {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        write!(writer, "# SUMPDF {}", self.pdf_sum_pairs.len())?;
        for pair in &self.pdf_sum_pairs {
            write!(writer, " {} {}", pair.0, pair.1)?;
        }
        writeln!(writer, "")
    }
}

#[cfg(test)]
impl Arbitrary for PdfSum {
    fn arbitrary<G: Gen>(gen: &mut G) -> PdfSum {
        PdfSum {
            pdf_sum_pairs: Arbitrary::arbitrary(gen),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct PdfInfo {
    pub x1: f64,
    pub x2: f64,
    pub scale: f64,
}

impl ReadLhe for PdfInfo {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], PdfInfo> {
        do_parse!(
            input,
            ws!(tag!("#")) >> ws!(tag!("pdf")) >> x1: ws!(parse_f64) >> x2: ws!(parse_f64)
                >> scale: ws!(parse_f64) >> (PdfInfo { x1, x2, scale })
        )
    }
}

impl WriteLhe for PdfInfo {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(writer, "# pdf {:e} {:e} {:e}", self.x1, self.x2, self.scale)
    }
}

#[cfg(test)]
impl Arbitrary for PdfInfo {
    fn arbitrary<G: Gen>(gen: &mut G) -> PdfInfo {
        PdfInfo {
            x1: Arbitrary::arbitrary(gen),
            x2: Arbitrary::arbitrary(gen),
            scale: Arbitrary::arbitrary(gen),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct InitExtraRS {
    pub pdf_sum: PdfSum,
    pub dip_map: DipMapInfo,
    pub jet_algo: JetAlgoInfo,
}

impl ReadLhe for InitExtraRS {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], InitExtraRS> {
        do_parse!(
            input,
            ex:
                permutation!(
                    ws!(PdfSum::read_from_lhe),
                    ws!(DipMapInfo::read_from_lhe),
                    ws!(JetAlgoInfo::read_from_lhe)
                ) >> (InitExtraRS {
                pdf_sum: ex.0,
                dip_map: ex.1,
                jet_algo: ex.2,
            })
        )
    }
}

impl WriteLhe for InitExtraRS {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        self.pdf_sum.write_lhe(writer)?;
        self.dip_map.write_lhe(writer)?;
        self.jet_algo.write_lhe(writer)
    }
}

#[cfg(test)]
impl Arbitrary for InitExtraRS {
    fn arbitrary<G: Gen>(gen: &mut G) -> InitExtraRS {
        InitExtraRS {
            pdf_sum: Arbitrary::arbitrary(gen),
            dip_map: Arbitrary::arbitrary(gen),
            jet_algo: Arbitrary::arbitrary(gen),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct JetAlgoInfo {
    pub algorithm_id: i8,
    pub n_bjets: u8,
    pub eta_max: f64,
    pub dr: f64,
    pub pt_veto: Option<f64>,
}

impl ReadLhe for JetAlgoInfo {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], JetAlgoInfo> {
        do_parse!(
            input,
            ws!(tag!("#")) >> ws!(tag!("JETALGO")) >> algorithm_id: ws!(parse_i8)
                >> n_bjets: ws!(parse_u8) >> eta_max: ws!(parse_f64)
                >> dr: ws!(parse_f64)
                >> has_pt_veto: ws!(alt!(tag!("T") => {|_| true} | tag!("F") => {|_| false}))
                >> pt_veto: ws!(parse_f64) >> (JetAlgoInfo {
                algorithm_id,
                n_bjets,
                eta_max,
                dr,
                pt_veto: if has_pt_veto { Some(pt_veto) } else { None },
            })
        )
    }
}

impl WriteLhe for JetAlgoInfo {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(
            writer,
            "# JETALGO {} {} {:?} {:?} {} {:?}",
            self.algorithm_id,
            self.n_bjets,
            self.eta_max,
            self.dr,
            if self.pt_veto.is_some() { "T" } else { "F" },
            self.pt_veto.unwrap_or(0.)
        )
    }
}

#[cfg(test)]
impl Arbitrary for JetAlgoInfo {
    fn arbitrary<G: Gen>(gen: &mut G) -> JetAlgoInfo {
        JetAlgoInfo {
            algorithm_id: Arbitrary::arbitrary(gen),
            n_bjets: Arbitrary::arbitrary(gen),
            eta_max: Arbitrary::arbitrary(gen),
            dr: Arbitrary::arbitrary(gen),
            pt_veto: Arbitrary::arbitrary(gen),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct DipMapInfo {
    pub dipole_type: i8,
    pub dipole_map: Vec<(i8, i8)>,
}

impl ReadLhe for DipMapInfo {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], DipMapInfo> {
        do_parse!(
            input,
            ws!(tag!("#")) >> ws!(tag!("DIPMAP")) >> dipole_type: ws!(parse_i8)
                >> ndipoles: ws!(parse_u8)
                >> dipole_map: count!(ws!(pair!(ws!(parse_i8), ws!(parse_i8))), ndipoles as usize)
                >> (DipMapInfo {
                    dipole_type,
                    dipole_map,
                })
        )
    }
}

impl WriteLhe for DipMapInfo {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        write!(
            writer,
            "# DIPMAP {} {}",
            self.dipole_type,
            self.dipole_map.len()
        )?;
        for dip in &self.dipole_map {
            write!(writer, " {} {}", dip.0, dip.1)?;
        }
        writeln!(writer, "")
    }
}

#[cfg(test)]
impl Arbitrary for DipMapInfo {
    fn arbitrary<G: Gen>(gen: &mut G) -> DipMapInfo {
        DipMapInfo {
            dipole_type: Arbitrary::arbitrary(gen),
            dipole_map: Arbitrary::arbitrary(gen),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct EventExtraRS {
    pub pdf: PdfInfo,
    pub me: MeInfoRS,
    pub jet: JetInfo,
}

impl ReadLhe for EventExtraRS {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], EventExtraRS> {
        do_parse!(
            input,
            ex:
                permutation!(
                    ws!(PdfInfo::read_from_lhe),
                    ws!(MeInfoRS::read_from_lhe),
                    ws!(JetInfo::read_from_lhe)
                ) >> (EventExtraRS {
                pdf: ex.0,
                me: ex.1,
                jet: ex.2,
            })
        )
    }
}

impl WriteLhe for EventExtraRS {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        self.pdf.write_lhe(writer)?;
        self.me.write_lhe(writer)?;
        self.jet.write_lhe(writer)
    }
}

#[cfg(test)]
impl Arbitrary for EventExtraRS {
    fn arbitrary<G: Gen>(gen: &mut G) -> EventExtraRS {
        EventExtraRS {
            pdf: Arbitrary::arbitrary(gen),
            me: Arbitrary::arbitrary(gen),
            jet: Arbitrary::arbitrary(gen),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct MeInfoRS {
    pub weight: f64,
    pub max_ew: u8,
    pub max_qcd: u8,
    pub real_weight: f64,
    pub scale: f64,
    pub dipole_ids: Vec<i8>,
    pub dipole_weights: Vec<f64>,
    pub dipole_mu_rs: Option<Vec<f64>>,
}

impl ReadLhe for MeInfoRS {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], MeInfoRS> {
        do_parse!(
            input,
            ws!(tag!("#")) >> ws!(tag!("me")) >> weight: ws!(parse_f64) >> max_ew: ws!(parse_u8)
                >> max_qcd: ws!(parse_u8) >> real_weight: ws!(parse_f64)
                >> scale: ws!(parse_f64) >> irun: ws!(parse_u8)
                >> num_dipoles: ws!(parse_u8)
                >> dipole_ids: count!(ws!(parse_i8), num_dipoles as usize)
                >> dipole_weights: count!(ws!(parse_f64), num_dipoles as usize)
                >> dipole_mu_rs:
                    count!(
                        ws!(parse_f64),
                        (if irun > 0 { num_dipoles } else { 0 }) as usize
                    ) >> (MeInfoRS {
                weight,
                max_ew,
                max_qcd,
                real_weight,
                scale,
                dipole_ids,
                dipole_weights,
                dipole_mu_rs: if irun > 0 { Some(dipole_mu_rs) } else { None },
            })
        )
    }
}

impl WriteLhe for MeInfoRS {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        write!(
            writer,
            "# me {:e} {} {} {:e} {:e} {} {}",
            self.weight,
            self.max_ew,
            self.max_qcd,
            self.real_weight,
            self.scale,
            if self.dipole_mu_rs.is_some() { 1 } else { 0 },
            self.dipole_ids.len(),
        )?;
        for id in &self.dipole_ids {
            write!(writer, " {}", id)?;
        }
        for weight in &self.dipole_weights {
            write!(writer, " {}", weight)?;
        }
        if let Some(ref mu_rs) = self.dipole_mu_rs {
            for mu_r in mu_rs {
                write!(writer, " {}", mu_r)?;
            }
        }
        writeln!(writer, "")
    }
}

#[cfg(test)]
impl Arbitrary for MeInfoRS {
    fn arbitrary<G: Gen>(gen: &mut G) -> MeInfoRS {
        let dip: Vec<(i8, f64, f64)> = Arbitrary::arbitrary(gen);
        let mut dipole_ids = Vec::new();
        let mut dipole_weights = Vec::new();
        let irun: bool = Arbitrary::arbitrary(gen);
        let mut dipole_mu_rs = if irun { Some(Vec::new()) } else { None };
        for (i, w, m) in dip {
            dipole_ids.push(i);
            dipole_weights.push(w);
            dipole_mu_rs.as_mut().map(|v| v.push(m));
        }
        MeInfoRS {
            weight: Arbitrary::arbitrary(gen),
            max_ew: Arbitrary::arbitrary(gen),
            max_qcd: Arbitrary::arbitrary(gen),
            real_weight: Arbitrary::arbitrary(gen),
            scale: Arbitrary::arbitrary(gen),
            dipole_ids,
            dipole_weights,
            dipole_mu_rs,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct JetInfo {
    pub ibvjet1: i8,
    pub ibvjet2: i8,
    pub ibvflreco: i8,
}

impl ReadLhe for JetInfo {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], JetInfo> {
        do_parse!(
            input,
            ws!(tag!("#")) >> ws!(tag!("jet")) >> ibvjet1: ws!(parse_i8) >> ibvjet2: ws!(parse_i8)
                >> ibvflreco: ws!(parse_i8) >> (JetInfo {
                ibvjet1: ibvjet1,
                ibvjet2: ibvjet2,
                ibvflreco: ibvflreco,
            })
        )
    }
}

impl WriteLhe for JetInfo {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(
            writer,
            "# jet {} {} {}",
            self.ibvjet1, self.ibvjet2, self.ibvflreco,
        )
    }
}

#[cfg(test)]
impl Arbitrary for JetInfo {
    fn arbitrary<G: Gen>(gen: &mut G) -> JetInfo {
        JetInfo {
            ibvjet1: Arbitrary::arbitrary(gen),
            ibvjet2: Arbitrary::arbitrary(gen),
            ibvflreco: Arbitrary::arbitrary(gen),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct EventExtraI {
    pub pdf: PdfInfo,
    pub me: MeInfoI,
}

impl ReadLhe for EventExtraI {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], EventExtraI> {
        do_parse!(
            input,
            ex: permutation!(ws!(PdfInfo::read_from_lhe), ws!(MeInfoI::read_from_lhe))
                >> (EventExtraI {
                    pdf: ex.0,
                    me: ex.1,
                })
        )
    }
}

impl WriteLhe for EventExtraI {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        self.pdf.write_lhe(writer)?;
        self.me.write_lhe(writer)
    }
}

#[cfg(test)]
impl Arbitrary for EventExtraI {
    fn arbitrary<G: Gen>(gen: &mut G) -> EventExtraI {
        EventExtraI {
            pdf: Arbitrary::arbitrary(gen),
            me: Arbitrary::arbitrary(gen),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct MeInfoI {
    pub max_ew: u8,
    pub max_qcd: u8,
    pub weight: f64,
    pub coeff_a: f64,
    pub coeff_b: f64,
    pub coeff_c: f64,
    pub log_term: i8,
}

impl ReadLhe for MeInfoI {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], MeInfoI> {
        do_parse!(
            input,
            ws!(tag!("#")) >> ws!(tag!("me")) >> max_ew: ws!(parse_u8) >> max_qcd: ws!(parse_u8)
                >> weight: ws!(parse_f64) >> coeff_a: ws!(parse_f64)
                >> coeff_b: ws!(parse_f64) >> coeff_c: ws!(parse_f64)
                >> log_term: ws!(parse_i8) >> (MeInfoI {
                max_ew,
                max_qcd,
                weight,
                coeff_a,
                coeff_b,
                coeff_c,
                log_term,
            })
        )
    }
}

impl WriteLhe for MeInfoI {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(
            writer,
            "# me {} {} {:e} {:e} {:e} {:e} {}",
            self.max_ew,
            self.max_qcd,
            self.weight,
            self.coeff_a,
            self.coeff_b,
            self.coeff_c,
            self.log_term
        )
    }
}

#[cfg(test)]
impl Arbitrary for MeInfoI {
    fn arbitrary<G: Gen>(gen: &mut G) -> MeInfoI {
        MeInfoI {
            max_ew: Arbitrary::arbitrary(gen),
            max_qcd: Arbitrary::arbitrary(gen),
            weight: Arbitrary::arbitrary(gen),
            coeff_a: Arbitrary::arbitrary(gen),
            coeff_b: Arbitrary::arbitrary(gen),
            coeff_c: Arbitrary::arbitrary(gen),
            log_term: Arbitrary::arbitrary(gen),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct PdfSumKP {
    pub beam_1_gluon_id: Option<PdgId>,
    pub beam_2_gluon_id: Option<PdgId>,
    pub beam_1_quark_ids: Vec<PdgId>,
    pub beam_2_quark_ids: Vec<PdgId>,
}

impl ReadLhe for PdfSumKP {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], PdfSumKP> {
        do_parse!(
            input,
            ws!(tag!("#")) >> ws!(tag!("SUMPDF")) >> n_pdf_g_1: ws!(parse_u64)
                >> n_pdf_q_1: ws!(parse_u64) >> n_pdf_g_2: ws!(parse_u64)
                >> n_pdf_q_2: ws!(parse_u64) >> beam_1_gluon_id: ws!(parse_i64)
                >> beam_1_quark_ids: count!(ws!(parse_i64), n_pdf_q_1 as usize)
                >> beam_2_gluon_id: ws!(parse_i64)
                >> beam_2_quark_ids: count!(ws!(parse_i64), n_pdf_q_2 as usize)
                >> (PdfSumKP {
                    beam_1_gluon_id: if n_pdf_g_1 != 0 {
                        Some(beam_1_gluon_id)
                    } else {
                        None
                    },
                    beam_2_gluon_id: if n_pdf_g_2 != 0 {
                        Some(beam_2_gluon_id)
                    } else {
                        None
                    },
                    beam_1_quark_ids,
                    beam_2_quark_ids,
                })
        )
    }
}

impl WriteLhe for PdfSumKP {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        write!(
            writer,
            "# SUMPDF {} {} {} {}",
            if self.beam_1_gluon_id.is_some() { 1 } else { 0 },
            self.beam_1_quark_ids.len(),
            if self.beam_2_gluon_id.is_some() { 1 } else { 0 },
            self.beam_2_quark_ids.len(),
        )?;
        write!(writer, " {}", self.beam_1_gluon_id.unwrap_or(0))?;
        for id in &self.beam_1_quark_ids {
            write!(writer, " {}", id)?;
        }
        write!(writer, " {}", self.beam_2_gluon_id.unwrap_or(0))?;
        for id in &self.beam_2_quark_ids {
            write!(writer, " {}", id)?;
        }
        writeln!(writer, "")
    }
}

#[cfg(test)]
impl Arbitrary for PdfSumKP {
    fn arbitrary<G: Gen>(gen: &mut G) -> PdfSumKP {
        PdfSumKP {
            beam_1_gluon_id: Arbitrary::arbitrary(gen),
            beam_2_gluon_id: Arbitrary::arbitrary(gen),
            beam_1_quark_ids: Arbitrary::arbitrary(gen),
            beam_2_quark_ids: Arbitrary::arbitrary(gen),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct EventExtraKP {
    pub pdf: PdfInfo,
    pub me: MeInfoKP,
}

impl ReadLhe for EventExtraKP {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], EventExtraKP> {
        do_parse!(
            input,
            ex: permutation!(ws!(PdfInfo::read_from_lhe), ws!(MeInfoKP::read_from_lhe))
                >> (EventExtraKP {
                    pdf: ex.0,
                    me: ex.1,
                })
        )
    }
}

impl WriteLhe for EventExtraKP {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        self.pdf.write_lhe(writer)?;
        self.me.write_lhe(writer)
    }
}

#[cfg(test)]
impl Arbitrary for EventExtraKP {
    fn arbitrary<G: Gen>(gen: &mut G) -> EventExtraKP {
        EventExtraKP {
            pdf: Arbitrary::arbitrary(gen),
            me: Arbitrary::arbitrary(gen),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct MeInfoKP {
    pub max_ew: u8,
    pub max_qcd: u8,
    pub weight: f64,
    pub x1_prime: f64,
    pub x2_prime: f64,
    pub weight_a1g_l0: f64,
    pub weight_a1g_l1: f64,
    pub weight_a1q_l0: f64,
    pub weight_a1q_l1: f64,
    pub weight_b1g_l0: f64,
    pub weight_b1g_l1: f64,
    pub weight_b1q_l0: f64,
    pub weight_b1q_l1: f64,
    pub weight_a2g_l0: f64,
    pub weight_a2g_l1: f64,
    pub weight_a2q_l0: f64,
    pub weight_a2q_l1: f64,
    pub weight_b2g_l0: f64,
    pub weight_b2g_l1: f64,
    pub weight_b2q_l0: f64,
    pub weight_b2q_l1: f64,
}

impl ReadLhe for MeInfoKP {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], MeInfoKP> {
        do_parse!(
            input,
            ws!(tag!("#")) >> ws!(tag!("me")) >> max_ew: ws!(parse_u8) >> max_qcd: ws!(parse_u8)
                >> weight: ws!(parse_f64) >> x1_prime: ws!(parse_f64)
                >> x2_prime: ws!(parse_f64) >> weight_a1g_l0: ws!(parse_f64)
                >> weight_a1g_l1: ws!(parse_f64) >> weight_a1q_l0: ws!(parse_f64)
                >> weight_a1q_l1: ws!(parse_f64) >> weight_b1g_l0: ws!(parse_f64)
                >> weight_b1g_l1: ws!(parse_f64) >> weight_b1q_l0: ws!(parse_f64)
                >> weight_b1q_l1: ws!(parse_f64) >> weight_a2g_l0: ws!(parse_f64)
                >> weight_a2g_l1: ws!(parse_f64) >> weight_a2q_l0: ws!(parse_f64)
                >> weight_a2q_l1: ws!(parse_f64) >> weight_b2g_l0: ws!(parse_f64)
                >> weight_b2g_l1: ws!(parse_f64) >> weight_b2q_l0: ws!(parse_f64)
                >> weight_b2q_l1: ws!(parse_f64) >> (MeInfoKP {
                max_ew,
                max_qcd,
                weight,
                x1_prime,
                x2_prime,
                weight_a1g_l0,
                weight_a1g_l1,
                weight_a1q_l0,
                weight_a1q_l1,
                weight_b1g_l0,
                weight_b1g_l1,
                weight_b1q_l0,
                weight_b1q_l1,
                weight_a2g_l0,
                weight_a2g_l1,
                weight_a2q_l0,
                weight_a2q_l1,
                weight_b2g_l0,
                weight_b2g_l1,
                weight_b2q_l0,
                weight_b2q_l1,
            })
        )
    }
}

impl WriteLhe for MeInfoKP {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        write!(
            writer,
            "# me {} {} {:e} {:e} {:e}",
            self.max_ew, self.max_qcd, self.weight, self.x1_prime, self.x2_prime
        )?;
        write!(
            writer,
            " {:e} {:e} {:e} {:e} {:e} {:e} {:e} {:e}",
            self.weight_a1g_l0,
            self.weight_a1g_l1,
            self.weight_a1q_l0,
            self.weight_a1q_l1,
            self.weight_b1g_l0,
            self.weight_b1g_l1,
            self.weight_b1q_l0,
            self.weight_b1q_l1,
        )?;
        writeln!(
            writer,
            " {:e} {:e} {:e} {:e} {:e} {:e} {:e} {:e}",
            self.weight_a2g_l0,
            self.weight_a2g_l1,
            self.weight_a2q_l0,
            self.weight_a2q_l1,
            self.weight_b2g_l0,
            self.weight_b2g_l1,
            self.weight_b2q_l0,
            self.weight_b2q_l1,
        )
    }
}

#[cfg(test)]
impl Arbitrary for MeInfoKP {
    fn arbitrary<G: Gen>(gen: &mut G) -> MeInfoKP {
        MeInfoKP {
            max_ew: Arbitrary::arbitrary(gen),
            max_qcd: Arbitrary::arbitrary(gen),
            weight: Arbitrary::arbitrary(gen),
            x1_prime: Arbitrary::arbitrary(gen),
            x2_prime: Arbitrary::arbitrary(gen),
            weight_a1g_l0: Arbitrary::arbitrary(gen),
            weight_a1g_l1: Arbitrary::arbitrary(gen),
            weight_a1q_l0: Arbitrary::arbitrary(gen),
            weight_a1q_l1: Arbitrary::arbitrary(gen),
            weight_b1g_l0: Arbitrary::arbitrary(gen),
            weight_b1g_l1: Arbitrary::arbitrary(gen),
            weight_b1q_l0: Arbitrary::arbitrary(gen),
            weight_b1q_l1: Arbitrary::arbitrary(gen),
            weight_a2g_l0: Arbitrary::arbitrary(gen),
            weight_a2g_l1: Arbitrary::arbitrary(gen),
            weight_a2q_l0: Arbitrary::arbitrary(gen),
            weight_a2q_l1: Arbitrary::arbitrary(gen),
            weight_b2g_l0: Arbitrary::arbitrary(gen),
            weight_b2g_l1: Arbitrary::arbitrary(gen),
            weight_b2q_l0: Arbitrary::arbitrary(gen),
            weight_b2q_l1: Arbitrary::arbitrary(gen),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct InitExtra1loop {
    pub pdf_sum: PdfSum,
    pub norm: Norm,
}

impl ReadLhe for InitExtra1loop {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], InitExtra1loop> {
        do_parse!(
            input,
            ex: permutation!(ws!(PdfSum::read_from_lhe), ws!(Norm::read_from_lhe))
                >> (InitExtra1loop {
                    pdf_sum: ex.0,
                    norm: ex.1,
                })
        )
    }
}

impl WriteLhe for InitExtra1loop {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        self.norm.write_lhe(writer)?;
        self.pdf_sum.write_lhe(writer)
    }
}

#[cfg(test)]
impl Arbitrary for InitExtra1loop {
    fn arbitrary<G: Gen>(gen: &mut G) -> InitExtra1loop {
        InitExtra1loop {
            pdf_sum: Arbitrary::arbitrary(gen),
            norm: Arbitrary::arbitrary(gen),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct Norm {
    pub n_unweighted_events: u64,
    pub alpha: f64,
    pub alpha_err: f64,
}

impl ReadLhe for Norm {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], Norm> {
        do_parse!(
            input,
            ws!(tag!("#")) >> ws!(tag!("NORM")) >> n_unweighted_events: ws!(parse_u64)
                >> alpha: ws!(parse_f64) >> alpha_err: ws!(parse_f64) >> (Norm {
                n_unweighted_events,
                alpha,
                alpha_err,
            })
        )
    }
}

impl WriteLhe for Norm {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(
            writer,
            "# NORM {} {:e} {:e}",
            self.n_unweighted_events, self.alpha, self.alpha_err
        )
    }
}

#[cfg(test)]
impl Arbitrary for Norm {
    fn arbitrary<G: Gen>(gen: &mut G) -> Norm {
        Norm {
            n_unweighted_events: Arbitrary::arbitrary(gen),
            alpha: Arbitrary::arbitrary(gen),
            alpha_err: Arbitrary::arbitrary(gen),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct EventExtra1loop {
    pub pdf: PdfInfo,
    pub me: MeInfo1loop,
}

impl ReadLhe for EventExtra1loop {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], EventExtra1loop> {
        do_parse!(
            input,
            ex: permutation!(ws!(PdfInfo::read_from_lhe), ws!(MeInfo1loop::read_from_lhe))
                >> (EventExtra1loop {
                    pdf: ex.0,
                    me: ex.1,
                })
        )
    }
}

impl WriteLhe for EventExtra1loop {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        self.pdf.write_lhe(writer)?;
        self.me.write_lhe(writer)
    }
}

#[cfg(test)]
impl Arbitrary for EventExtra1loop {
    fn arbitrary<G: Gen>(gen: &mut G) -> EventExtra1loop {
        EventExtra1loop {
            pdf: Arbitrary::arbitrary(gen),
            me: Arbitrary::arbitrary(gen),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct MeInfo1loop {
    pub max_ew_lo: i64,
    pub max_qcd_lo: i64,
    pub weight_lo: f64,
    pub max_ew_1loop: i64,
    pub max_qcd_1loop: i64,
    pub weight_1loop: f64,
    pub coeff_a: f64,
    pub coeff_b: f64,
    pub coeff_c: f64,
}

impl ReadLhe for MeInfo1loop {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], MeInfo1loop> {
        do_parse!(
            input,
            ws!(tag!("#")) >> ws!(tag!("me")) >> max_ew_lo: ws!(parse_i64)
                >> max_qcd_lo: ws!(parse_i64) >> weight_lo: ws!(parse_f64)
                >> max_ew_1loop: ws!(parse_i64) >> max_qcd_1loop: ws!(parse_i64)
                >> weight_1loop: ws!(parse_f64) >> coeff_a: ws!(parse_f64)
                >> coeff_b: ws!(parse_f64) >> coeff_c: ws!(parse_f64)
                >> (MeInfo1loop {
                    max_ew_lo,
                    max_qcd_lo,
                    weight_lo,
                    max_ew_1loop,
                    max_qcd_1loop,
                    weight_1loop,
                    coeff_a,
                    coeff_b,
                    coeff_c,
                })
        )
    }
}

impl WriteLhe for MeInfo1loop {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(
            writer,
            "# me {} {} {:e} {} {} {:e} {:e} {:e} {:e}",
            self.max_ew_lo,
            self.max_qcd_lo,
            self.weight_lo,
            self.max_ew_1loop,
            self.max_qcd_1loop,
            self.weight_1loop,
            self.coeff_a,
            self.coeff_b,
            self.coeff_c,
        )
    }
}

#[cfg(test)]
impl Arbitrary for MeInfo1loop {
    fn arbitrary<G: Gen>(gen: &mut G) -> MeInfo1loop {
        MeInfo1loop {
            max_ew_lo: Arbitrary::arbitrary(gen),
            max_qcd_lo: Arbitrary::arbitrary(gen),
            weight_lo: Arbitrary::arbitrary(gen),
            max_ew_1loop: Arbitrary::arbitrary(gen),
            max_qcd_1loop: Arbitrary::arbitrary(gen),
            weight_1loop: Arbitrary::arbitrary(gen),
            coeff_a: Arbitrary::arbitrary(gen),
            coeff_b: Arbitrary::arbitrary(gen),
            coeff_c: Arbitrary::arbitrary(gen),
        }
    }
}

#[cfg(test)]
mod tests {
    use quickcheck;
    use serde_json;
    use std::fs;
    use std::str;

    use {ReadLhe, WriteLhe};
    use super::*;

    macro_rules! roundtrip_qc {
        ($name:ident, $ty:ident) => {
            quickcheck! {
                fn $name(start: $ty) -> quickcheck::TestResult {
                    let mut bytes = Vec::new();
                    start.write_lhe(&mut bytes).unwrap();
                        let round = match $ty::read_from_lhe(&bytes).to_full_result() {
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

    roundtrip_qc!(comment_roundtrip_qc, Comment);
    roundtrip_qc!(pdfsum_roundtrip_qc, PdfSum);
    roundtrip_qc!(pdfinfo_roundtrip_qc, PdfInfo);
    roundtrip_qc!(jetalgo_roundtrip_qc, JetAlgoInfo);
    roundtrip_qc!(dipmap_roundtrip_qc, DipMapInfo);
    roundtrip_qc!(initextrars_roundtrip_qc, InitExtraRS);
    roundtrip_qc!(meinfors_roundtrip_qc, MeInfoRS);
    roundtrip_qc!(jetinfo_roundtrip_qc, JetInfo);
    roundtrip_qc!(eventextrars_roundtrip_qc, EventExtraRS);
    roundtrip_qc!(meinfoi_roundtrip_qc, MeInfoI);
    roundtrip_qc!(eventextrai_roundtrip_qc, EventExtraI);
    roundtrip_qc!(pdfsumkp_roundtrip_qc, PdfSumKP);
    roundtrip_qc!(meinfokp_roundtrip_qc, MeInfoKP);
    roundtrip_qc!(eventextrakp_roundtrip_qc, EventExtraKP);
    roundtrip_qc!(norm_roundtrip_qc, Norm);
    roundtrip_qc!(initextra1loop_roundtrip_qc, InitExtra1loop);
    roundtrip_qc!(meinfo1loop_roundtrip_qc, MeInfo1loop);
    roundtrip_qc!(eventextra1loop_roundtrip_qc, EventExtra1loop);

    #[test]
    fn meinfors_roundtrip() {
        let start = MeInfoRS {
            weight: 1.,
            max_ew: 2,
            max_qcd: 3,
            real_weight: 4.,
            scale: 5.,
            dipole_ids: vec![7],
            dipole_weights: vec![8.],
            dipole_mu_rs: Some(vec![9.]),
        };
        let mut bytes = Vec::new();
        start.write_lhe(&mut bytes).unwrap();
        let round = match MeInfoRS::read_from_lhe(&bytes).to_full_result() {
            Ok(r) => r,
            Err(err) => {
                println!("{}", str::from_utf8(&bytes).unwrap());
                panic!("Failed to read roundtrip: {:?}", err);
            }
        };
        if start != round {
            println!("After: {:?}", round);
            assert_eq!(start, round);
        }
    }

    #[test]
    fn read_comment() {
        let bytes = b"<!--
File generated with HELAC-DIPOLES
-->";
        let expected = Comment("File generated with HELAC-DIPOLES".to_string());
        let comment = Comment::read_from_lhe(bytes as &[u8])
            .to_full_result()
            .unwrap();
        assert_eq!(comment, expected);
    }

    #[test]
    fn read_pdfsum() {
        let bytes = b"# SUMPDF 4 1 2 3 4 -1 -2 0 8\n";
        let expected = PdfSum {
            pdf_sum_pairs: vec![(1, 2), (3, 4), (-1, -2), (0, 8)],
        };
        let result = PdfSum::read_from_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_pdfsum_empty() {
        let bytes = b"# SUMPDF 0\n";
        let expected = PdfSum {
            pdf_sum_pairs: vec![],
        };
        let result = PdfSum::read_from_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_pdfinfo() {
        let bytes = b"# pdf 1.0 2.0 3.0\n";
        let expected = PdfInfo {
            x1: 1.0,
            x2: 2.0,
            scale: 3.0,
        };
        let result = PdfInfo::read_from_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_jetalgoinfo_true() {
        let bytes = b"# JETALGO 1 2 3. 4. T 5.\n";
        let expected = JetAlgoInfo {
            algorithm_id: 1,
            n_bjets: 2,
            eta_max: 3.,
            dr: 4.,
            pt_veto: Some(5.),
        };
        let result = JetAlgoInfo::read_from_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_jetalgoinfo_false() {
        let bytes = b"# JETALGO 1 2 3. 4. F 5.\n";
        let expected = JetAlgoInfo {
            algorithm_id: 1,
            n_bjets: 2,
            eta_max: 3.,
            dr: 4.,
            pt_veto: None,
        };
        let result = JetAlgoInfo::read_from_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_dipmap() {
        let bytes = b"# DIPMAP 1   9  1  7  1  8  1  9  2  7  2  8  2  9  7  8  7  9  8  9\n";
        let expected = DipMapInfo {
            dipole_type: 1,
            dipole_map: vec![
                (1, 7),
                (1, 8),
                (1, 9),
                (2, 7),
                (2, 8),
                (2, 9),
                (7, 8),
                (7, 9),
                (8, 9),
            ],
        };
        let result = DipMapInfo::read_from_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_initextrars() {
        let bytes_normal = b"# SUMPDF 4 1 2 3 4 -1 -2 0 8\n# DIPMAP 1   9  1  7  1  8  1  9  2  7  2  8  2  9  7  8  7  9  8  9\n# JETALGO 1 2 3. 4. F 5.\n";
        let bytes_reverse = b"# JETALGO 1 2 3. 4. F 5.\n# DIPMAP 1   9  1  7  1  8  1  9  2  7  2  8  2  9  7  8  7  9  8  9\n# SUMPDF 4 1 2 3 4 -1 -2 0 8\n";
        let expected = InitExtraRS {
            pdf_sum: PdfSum {
                pdf_sum_pairs: vec![(1, 2), (3, 4), (-1, -2), (0, 8)],
            },
            dip_map: DipMapInfo {
                dipole_type: 1,
                dipole_map: vec![
                    (1, 7),
                    (1, 8),
                    (1, 9),
                    (2, 7),
                    (2, 8),
                    (2, 9),
                    (7, 8),
                    (7, 9),
                    (8, 9),
                ],
            },
            jet_algo: JetAlgoInfo {
                algorithm_id: 1,
                n_bjets: 2,
                eta_max: 3.,
                dr: 4.,
                pt_veto: None,
            },
        };
        let result_normal = InitExtraRS::read_from_lhe(bytes_normal)
            .to_full_result()
            .unwrap();
        assert_eq!(result_normal, expected);
        let result_reverse = InitExtraRS::read_from_lhe(bytes_reverse)
            .to_full_result()
            .unwrap();
        assert_eq!(result_reverse, expected);
    }

    #[test]
    fn read_meinfors() {
        let bytes = b"# me 13. 1 6 3. 4. 5 2 7 8 9. 10. 11. 12.\n";
        let expected = MeInfoRS {
            weight: 13.,
            max_ew: 1,
            max_qcd: 6,
            real_weight: 3.,
            scale: 4.,
            dipole_ids: vec![7, 8],
            dipole_weights: vec![9., 10.],
            dipole_mu_rs: Some(vec![11., 12.]),
        };
        let result = MeInfoRS::read_from_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_meinfors_irun0() {
        let bytes = b"# me 13. 1 6 3. 4. 0 2 7 8 9. 10. 11. 12.\n";
        let expected = MeInfoRS {
            weight: 13.,
            max_ew: 1,
            max_qcd: 6,
            real_weight: 3.,
            scale: 4.,
            dipole_ids: vec![7, 8],
            dipole_weights: vec![9., 10.],
            dipole_mu_rs: None,
        };
        let result = MeInfoRS::read_from_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_jetinfo() {
        let bytes = b"# jet 1 2 3\n";
        let expected = JetInfo {
            ibvjet1: 1,
            ibvjet2: 2,
            ibvflreco: 3,
        };
        let result = JetInfo::read_from_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_eventextrars() {
        let bytes_normal =
            b"# pdf 1.0 2.0 3.0\n# me 13. 1 6 3. 4. 5 2 7 8 9. 10. 11. 12.\n# jet 1 2 3\n";
        let bytes_reverse =
            b"# jet 1 2 3\n# me 13. 1 6 3. 4. 5 2 7 8 9. 10. 11. 12.\n# pdf 1.0 2.0 3.0\n";
        let expected = EventExtraRS {
            pdf: PdfInfo {
                x1: 1.0,
                x2: 2.0,
                scale: 3.0,
            },
            me: MeInfoRS {
                weight: 13.,
                max_ew: 1,
                max_qcd: 6,
                real_weight: 3.,
                scale: 4.,
                dipole_ids: vec![7, 8],
                dipole_weights: vec![9., 10.],
                dipole_mu_rs: Some(vec![11., 12.]),
            },
            jet: JetInfo {
                ibvjet1: 1,
                ibvjet2: 2,
                ibvflreco: 3,
            },
        };
        let result_normal = EventExtraRS::read_from_lhe(bytes_normal)
            .to_full_result()
            .unwrap();
        assert_eq!(result_normal, expected);
        let result_reverse = EventExtraRS::read_from_lhe(bytes_reverse)
            .to_full_result()
            .unwrap();
        assert_eq!(result_reverse, expected);
    }

    #[test]
    fn read_meinfoi() {
        let bytes = b"# me 1 2 3. 4. 5. 6. 7\n";
        let expected = MeInfoI {
            max_ew: 1,
            max_qcd: 2,
            weight: 3.,
            coeff_a: 4.,
            coeff_b: 5.,
            coeff_c: 6.,
            log_term: 7,
        };
        let result = MeInfoI::read_from_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_eventextrai() {
        let bytes_normal = b"# pdf 1.0 2.0 3.0\n# me 1 2 3. 4. 5. 6. 7\n";
        let bytes_reverse = b"# me 1 2 3. 4. 5. 6. 7\n# pdf 1.0 2.0 3.0\n";
        let expected = EventExtraI {
            pdf: PdfInfo {
                x1: 1.0,
                x2: 2.0,
                scale: 3.0,
            },
            me: MeInfoI {
                max_ew: 1,
                max_qcd: 2,
                weight: 3.,
                coeff_a: 4.,
                coeff_b: 5.,
                coeff_c: 6.,
                log_term: 7,
            },
        };
        let result_normal = EventExtraI::read_from_lhe(bytes_normal)
            .to_full_result()
            .unwrap();
        assert_eq!(result_normal, expected);
        let result_reverse = EventExtraI::read_from_lhe(bytes_reverse)
            .to_full_result()
            .unwrap();
        assert_eq!(result_reverse, expected);
    }

    #[test]
    fn read_pdfsumkp() {
        let bytes = b"# SUMPDF 1 2 0 3 4 5 6 7 8 9 10\n";
        let expected = PdfSumKP {
            beam_1_gluon_id: Some(4),
            beam_2_gluon_id: None,
            beam_1_quark_ids: vec![5, 6],
            beam_2_quark_ids: vec![8, 9, 10],
        };
        let result = PdfSumKP::read_from_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_meinfokp() {
        let bytes =
            b"# me 1 2 3. 4. 5. 6. 7. 8. 9. 10. 11. 12. 13. 14. 15. 16. 17. 18. 19. 20. 21.\n";
        let expected = MeInfoKP {
            max_ew: 1,
            max_qcd: 2,
            weight: 3.,
            x1_prime: 4.,
            x2_prime: 5.,
            weight_a1g_l0: 6.,
            weight_a1g_l1: 7.,
            weight_a1q_l0: 8.,
            weight_a1q_l1: 9.,
            weight_b1g_l0: 10.,
            weight_b1g_l1: 11.,
            weight_b1q_l0: 12.,
            weight_b1q_l1: 13.,
            weight_a2g_l0: 14.,
            weight_a2g_l1: 15.,
            weight_a2q_l0: 16.,
            weight_a2q_l1: 17.,
            weight_b2g_l0: 18.,
            weight_b2g_l1: 19.,
            weight_b2q_l0: 20.,
            weight_b2q_l1: 21.,
        };
        let result = MeInfoKP::read_from_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_eventextrakp() {
        let bytes_normal = b"# pdf 1.0 2.0 3.0\n# me 1 2 3. 4. 5. 6. 7. 8. 9. 10. 11. 12. 13. 14. 15. 16. 17. 18. 19. 20. 21.\n";
        let bytes_reverse = b"# me 1 2 3. 4. 5. 6. 7. 8. 9. 10. 11. 12. 13. 14. 15. 16. 17. 18. 19. 20. 21.\n# pdf 1.0 2.0 3.0\n";
        let expected = EventExtraKP {
            pdf: PdfInfo {
                x1: 1.0,
                x2: 2.0,
                scale: 3.0,
            },
            me: MeInfoKP {
                max_ew: 1,
                max_qcd: 2,
                weight: 3.,
                x1_prime: 4.,
                x2_prime: 5.,
                weight_a1g_l0: 6.,
                weight_a1g_l1: 7.,
                weight_a1q_l0: 8.,
                weight_a1q_l1: 9.,
                weight_b1g_l0: 10.,
                weight_b1g_l1: 11.,
                weight_b1q_l0: 12.,
                weight_b1q_l1: 13.,
                weight_a2g_l0: 14.,
                weight_a2g_l1: 15.,
                weight_a2q_l0: 16.,
                weight_a2q_l1: 17.,
                weight_b2g_l0: 18.,
                weight_b2g_l1: 19.,
                weight_b2q_l0: 20.,
                weight_b2q_l1: 21.,
            },
        };
        let result_normal = EventExtraKP::read_from_lhe(bytes_normal)
            .to_full_result()
            .unwrap();
        assert_eq!(result_normal, expected);
        let result_reverse = EventExtraKP::read_from_lhe(bytes_reverse)
            .to_full_result()
            .unwrap();
        assert_eq!(result_reverse, expected);
    }

    #[test]
    fn read_norm() {
        let bytes = b"# NORM 1 2. 3.\n";
        let expected = Norm {
            n_unweighted_events: 1,
            alpha: 2.,
            alpha_err: 3.,
        };
        let result = Norm::read_from_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_initextra1loop() {
        let bytes_normal = b"# NORM 1 2. 3.\n# SUMPDF 4 1 2 3 4 -1 -2 0 8\n";
        let bytes_reverse = b"# SUMPDF 4 1 2 3 4 -1 -2 0 8\n# NORM 1 2. 3.\n";
        let expected = InitExtra1loop {
            pdf_sum: PdfSum {
                pdf_sum_pairs: vec![(1, 2), (3, 4), (-1, -2), (0, 8)],
            },
            norm: Norm {
                n_unweighted_events: 1,
                alpha: 2.,
                alpha_err: 3.,
            },
        };
        let result_normal = InitExtra1loop::read_from_lhe(bytes_normal)
            .to_full_result()
            .unwrap();
        assert_eq!(result_normal, expected);
        let result_reverse = InitExtra1loop::read_from_lhe(bytes_reverse)
            .to_full_result()
            .unwrap();
        assert_eq!(result_reverse, expected);
    }

    #[test]
    fn read_meinfo1loop() {
        let bytes = b"# me 1 2 3. 4 5 6. 7. 8. 9.\n";
        let expected = MeInfo1loop {
            max_ew_lo: 1,
            max_qcd_lo: 2,
            weight_lo: 3.,
            max_ew_1loop: 4,
            max_qcd_1loop: 5,
            weight_1loop: 6.,
            coeff_a: 7.,
            coeff_b: 8.,
            coeff_c: 9.,
        };
        let result = MeInfo1loop::read_from_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_eventextra1loop() {
        let bytes_normal = b"# pdf 1.0 2.0 3.0\n# me 1 2 3. 4 5 6. 7. 8. 9.\n";
        let bytes_reverse = b"# me 1 2 3. 4 5 6. 7. 8. 9.\n# pdf 1.0 2.0 3.0\n";
        let expected = EventExtra1loop {
            pdf: PdfInfo {
                x1: 1.0,
                x2: 2.0,
                scale: 3.0,
            },
            me: MeInfo1loop {
                max_ew_lo: 1,
                max_qcd_lo: 2,
                weight_lo: 3.,
                max_ew_1loop: 4,
                max_qcd_1loop: 5,
                weight_1loop: 6.,
                coeff_a: 7.,
                coeff_b: 8.,
                coeff_c: 9.,
            },
        };
        let result_normal = EventExtra1loop::read_from_lhe(bytes_normal)
            .to_full_result()
            .unwrap();
        assert_eq!(result_normal, expected);
        let result_reverse = EventExtra1loop::read_from_lhe(bytes_reverse)
            .to_full_result()
            .unwrap();
        assert_eq!(result_reverse, expected);
    }

    #[test]
    fn read_rs() {
        LheFileRS::read_lhe_from_file(&"tests/real_world_files/helac_dipoles_rs.lhe").unwrap();
    }

    #[test]
    fn validate_rs() {
        let lhe =
            LheFileRS::read_lhe_from_file(&"tests/real_world_files/helac_dipoles_rs.lhe").unwrap();
        let mut file = fs::File::open("tests/real_world_files/helac_dipoles_rs.json").unwrap();
        let valid: LheFileRS = serde_json::from_reader(&mut file).unwrap();
        assert_eq!(lhe, valid);
    }

    #[test]
    fn roundtrip_rs() {
        let lhe =
            match LheFileRS::read_lhe_from_file(&"tests/real_world_files/helac_dipoles_rs.lhe") {
                Ok(l) => l,
                Err(e) => panic!("Failed to read: {:?}", e),
            };

        let mut bytes = Vec::new();
        lhe.write_lhe(&mut bytes).unwrap();
        let round = match LheFileRS::read_from_lhe(&bytes).to_full_result() {
            Ok(l) => l,
            Err(e) => panic!("Failed to read roundtrip: {:?}", e),
        };
        assert_eq!(lhe, round);
    }

    #[test]
    fn read_i() {
        LheFileI::read_lhe_from_file(&"tests/real_world_files/helac_dipoles_i.lhe").unwrap();
    }

    #[test]
    fn roundtrip_i() {
        let lhe = match LheFileI::read_lhe_from_file(&"tests/real_world_files/helac_dipoles_i.lhe")
        {
            Ok(l) => l,
            Err(e) => panic!("Failed to read: {:?}", e),
        };

        let mut bytes = Vec::new();
        lhe.write_lhe(&mut bytes).unwrap();
        let round = match LheFileI::read_from_lhe(&bytes).to_full_result() {
            Ok(l) => l,
            Err(e) => panic!("Failed to read roundtrip: {:?}", e),
        };
        assert_eq!(lhe, round);
    }

    #[test]
    fn validate_i() {
        let lhe =
            LheFileI::read_lhe_from_file(&"tests/real_world_files/helac_dipoles_i.lhe").unwrap();
        let mut file = fs::File::open("tests/real_world_files/helac_dipoles_i.json").unwrap();
        let valid: LheFileI = serde_json::from_reader(&mut file).unwrap();
        assert_eq!(lhe, valid);
    }

    #[test]
    fn read_kp() {
        LheFileKP::read_lhe_from_file(&"tests/real_world_files/helac_dipoles_kp.lhe").unwrap();
    }

    #[test]
    fn roundtrip_kp() {
        let lhe =
            match LheFileKP::read_lhe_from_file(&"tests/real_world_files/helac_dipoles_kp.lhe") {
                Ok(l) => l,
                Err(e) => panic!("Failed to read: {:?}", e),
            };

        let mut bytes = Vec::new();
        lhe.write_lhe(&mut bytes).unwrap();
        let round = match LheFileKP::read_from_lhe(&bytes).to_full_result() {
            Ok(l) => l,
            Err(e) => panic!("Failed to read roundtrip: {:?}", e),
        };
        assert_eq!(lhe, round);
    }

    #[test]
    fn validate_kp() {
        let lhe =
            LheFileKP::read_lhe_from_file(&"tests/real_world_files/helac_dipoles_kp.lhe").unwrap();
        let mut file = fs::File::open("tests/real_world_files/helac_dipoles_kp.json").unwrap();
        let valid: LheFileKP = serde_json::from_reader(&mut file).unwrap();
        assert_eq!(lhe, valid);
    }

    #[test]
    fn read_1loop() {
        LheFile1loop::read_lhe_from_file(&"tests/real_world_files/helac_1loop_virt.lhe").unwrap();
    }

    #[test]
    fn roundtrip_1loop() {
        let lhe = match LheFile1loop::read_lhe_from_file(
            &"tests/real_world_files/helac_1loop_virt.lhe",
        ) {
            Ok(l) => l,
            Err(e) => panic!("Failed to read: {:?}", e),
        };

        let mut bytes = Vec::new();
        lhe.write_lhe(&mut bytes).unwrap();
        let round = match LheFile1loop::read_from_lhe(&bytes).to_full_result() {
            Ok(l) => l,
            Err(e) => panic!("Failed to read roundtrip: {:?}", e),
        };
        assert_eq!(lhe, round);
    }

    #[test]
    fn validate_1loop() {
        let lhe = LheFile1loop::read_lhe_from_file(&"tests/real_world_files/helac_1loop_virt.lhe")
            .unwrap();
        let mut file = fs::File::open("tests/real_world_files/helac_1loop_virt.json").unwrap();
        let valid: LheFile1loop = serde_json::from_reader(&mut file).unwrap();
        assert_eq!(lhe, valid);
    }
}
