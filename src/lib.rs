// Copyright 2018 Torsten Weber
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate lorentz_vector;
#[macro_use]
extern crate nom;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
#[macro_use]
extern crate serde;
#[cfg(test)]
#[cfg(test)]
extern crate serde_json;

#[macro_use]
pub mod nom_util;
pub mod generic;
pub mod helac;
pub mod plain;
pub mod string;

use lorentz_vector::LorentzVector;

use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::io::Read;
use std::marker;
use std::path::Path;

#[cfg(test)]
use quickcheck::Arbitrary;
#[cfg(test)]
use quickcheck::Gen;

use nom_util::{parse_f64, parse_i64};

pub type PdgId = i64;

pub trait ReadLhe
where
    Self: marker::Sized,
{
    fn read_from_lhe(&[u8]) -> nom::IResult<&[u8], Self>;

    fn read_lhe_from_file<P: AsRef<Path>>(path: &P) -> Result<Self, ReadError> {
        let mut file = fs::File::open(path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        Self::read_from_lhe(&contents)
            .to_full_result()
            .map_err(ReadError::Nom)
    }
}

pub trait WriteLhe {
    fn write_lhe<W: io::Write>(&self, &mut W) -> io::Result<()>;

    fn write_lhe_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file = fs::File::create(path)?;
        self.write_lhe(&mut file)
    }
}

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Nom(nom::IError),
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> ReadError {
        ReadError::Io(err)
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReadError::Io(ref err) => {
                write!(f, "Failed to read the lhe file with an IO error: {}", err)
            }
            ReadError::Nom(ref err) => write!(
                f,
                "Failed to read the lhe file with a parse error: {:?}",
                err
            ),
        }
    }
}

impl error::Error for ReadError {
    fn description(&self) -> &str {
        match *self {
            ReadError::Io(..) => &"Failed to read the lhe file with an IO error",
            ReadError::Nom(..) => &"Failed to read the lhe file with a parse error",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ReadError::Io(ref err) => Some(err),
            ReadError::Nom(_) => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct ProcInfo {
    pub xsect: f64,
    pub xsect_err: f64,
    pub maximum_weight: f64,
    pub process_id: i64,
}

impl ReadLhe for ProcInfo {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], ProcInfo> {
        do_parse!(
            input,
            xsect: ws!(parse_f64) >> xsect_err: ws!(parse_f64) >> maximum_weight: ws!(parse_f64)
                >> process_id: ws!(parse_i64) >> (ProcInfo {
                xsect,
                xsect_err,
                maximum_weight,
                process_id,
            })
        )
    }
}

impl WriteLhe for ProcInfo {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(
            writer,
            "{:e} {:e} {:e} {}",
            self.xsect, self.xsect_err, self.maximum_weight, self.process_id
        )
    }
}

#[cfg(test)]
impl Arbitrary for ProcInfo {
    fn arbitrary<G: Gen>(gen: &mut G) -> ProcInfo {
        ProcInfo {
            xsect: Arbitrary::arbitrary(gen),
            xsect_err: Arbitrary::arbitrary(gen),
            maximum_weight: Arbitrary::arbitrary(gen),
            process_id: Arbitrary::arbitrary(gen),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Serialize, Deserialize))]
pub struct Particle {
    pub pdg_id: PdgId,
    pub status: i64,
    pub mother_1_id: i64,
    pub mother_2_id: i64,
    pub color_1: i64,
    pub color_2: i64,
    pub momentum: LorentzVector,
    pub mass: f64,
    pub proper_lifetime: f64,
    pub spin: f64,
}

impl ReadLhe for Particle {
    fn read_from_lhe(input: &[u8]) -> nom::IResult<&[u8], Particle> {
        do_parse!(
            input,
            pdg_id: ws!(parse_i64) >> status: ws!(parse_i64) >> mother_1_id: ws!(parse_i64)
                >> mother_2_id: ws!(parse_i64) >> color_1: ws!(parse_i64)
                >> color_2: ws!(parse_i64) >> px: ws!(parse_f64) >> py: ws!(parse_f64)
                >> pz: ws!(parse_f64) >> e: ws!(parse_f64) >> mass: ws!(parse_f64)
                >> proper_lifetime: ws!(parse_f64) >> spin: ws!(parse_f64)
                >> (Particle {
                    pdg_id,
                    status,
                    mother_1_id,
                    mother_2_id,
                    color_1,
                    color_2,
                    momentum: LorentzVector { e, px, py, pz },
                    mass,
                    proper_lifetime,
                    spin,
                })
        )
    }
}

impl WriteLhe for Particle {
    fn write_lhe<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writeln!(
            writer,
            "{} {} {} {} {} {} {:e} {:e} {:e} {:e} {:e} {:e} {:e}",
            self.pdg_id,
            self.status,
            self.mother_1_id,
            self.mother_2_id,
            self.color_1,
            self.color_2,
            self.momentum.px,
            self.momentum.py,
            self.momentum.pz,
            self.momentum.e,
            self.mass,
            self.proper_lifetime,
            self.spin
        )
    }
}

#[cfg(test)]
impl Arbitrary for Particle {
    fn arbitrary<G: Gen>(gen: &mut G) -> Particle {
        let momentum = LorentzVector {
            e: Arbitrary::arbitrary(gen),
            px: Arbitrary::arbitrary(gen),
            py: Arbitrary::arbitrary(gen),
            pz: Arbitrary::arbitrary(gen),
        };
        Particle {
            pdg_id: Arbitrary::arbitrary(gen),
            status: Arbitrary::arbitrary(gen),
            mother_1_id: Arbitrary::arbitrary(gen),
            mother_2_id: Arbitrary::arbitrary(gen),
            color_1: Arbitrary::arbitrary(gen),
            color_2: Arbitrary::arbitrary(gen),
            momentum,
            mass: Arbitrary::arbitrary(gen),
            proper_lifetime: Arbitrary::arbitrary(gen),
            spin: Arbitrary::arbitrary(gen),
        }
    }
}

#[cfg(test)]
mod tests {
    use lorentz_vector::LorentzVector;
    use super::{ReadLhe, WriteLhe};
    use super::{Particle, ProcInfo};

    #[test]
    fn read_procinfo() {
        let bytes = b"1. 2. 3. 4\n";
        let expected = ProcInfo {
            xsect: 1.,
            xsect_err: 2.,
            maximum_weight: 3.,
            process_id: 4,
        };
        let result = ProcInfo::read_from_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn read_particle() {
        let bytes = b"1 2 3 4 5 6 7. 8. 9. 10. 11. 12. 13.\n";
        let expected = Particle {
            pdg_id: 1,
            status: 2,
            mother_1_id: 3,
            mother_2_id: 4,
            color_1: 5,
            color_2: 6,
            momentum: LorentzVector {
                px: 7.,
                py: 8.,
                pz: 9.,
                e: 10.,
            },
            mass: 11.,
            proper_lifetime: 12.,
            spin: 13.,
        };
        let result = Particle::read_from_lhe(bytes).to_full_result().unwrap();
        assert_eq!(result, expected);
    }

    quickcheck! {
        fn proc_info_roundtrip_qc(p: ProcInfo) -> bool {
            let mut bytes = Vec::new();
            p.write_lhe(&mut bytes).unwrap();
            let round = match ProcInfo::read_from_lhe(&bytes).to_full_result() {
                Ok(r) => r,
                Err(err) => panic!("Failed to read roundtrip: {:?}", err),
            };
            p == round
        }
    }

    quickcheck! {
        fn particle_roundtrip_qc(m: Particle) -> bool {
            let mut bytes = Vec::new();
            m.write_lhe(&mut bytes).unwrap();
            let round = match Particle::read_from_lhe(&bytes).to_full_result() {
                Ok(r) => r,
                Err(err) => panic!("Failed to read roundtrip: {:?}", err),
            };
            m == round
        }
    }
}
