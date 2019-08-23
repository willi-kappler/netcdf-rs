

// Rust modules
// use std::path::Path;
// use std::fs::File;
use std::io;
use std::{fmt, fmt::Display, fmt::Formatter};
use std::string::FromUtf8Error;

// The netCDF format is described here:
// https://www.unidata.ucar.edu/software/netcdf/docs/file_format_specifications.html

pub(crate) type OneByte = [u8; 1];
pub(crate) type TwoBytes = [u8; 2];
pub(crate) type FourBytes = [u8; 4];
pub(crate) type EightBytes = [u8; 8];

pub(crate) const STREAMING: FourBytes = [0xff, 0xff, 0xff, 0xff];
pub(crate) const ZERO: FourBytes = [0x00, 0x00, 0x00, 0x00];
pub(crate) const VERSION1: FourBytes = [0x43, 0x44, 0x46, 0x01];
pub(crate) const VERSION2: FourBytes = [0x43, 0x44, 0x46, 0x02];
pub(crate) const VERSION4: FourBytes = [0x89, 0x48, 0x44, 0x46]; // HDF 5, TODO

pub(crate) const NC_DIMENSION: FourBytes = [0x00, 0x00, 0x00, 0x0a];
pub(crate) const NC_VARIABLE: FourBytes = [0x00, 0x00, 0x00, 0x0b];
pub(crate) const NC_ATTRIBUTE: FourBytes = [0x00, 0x00, 0x00, 0x0c];

pub(crate) const NC_BYTE: FourBytes = [0x00, 0x00, 0x00, 0x01];
pub(crate) const NC_CHAR: FourBytes = [0x00, 0x00, 0x00, 0x02];
pub(crate) const NC_SHORT: FourBytes = [0x00, 0x00, 0x00, 0x03];
pub(crate) const NC_INT: FourBytes = [0x00, 0x00, 0x00, 0x04];
pub(crate) const NC_FLOAT: FourBytes = [0x00, 0x00, 0x00, 0x05];
pub(crate) const NC_DOUBLE: FourBytes = [0x00, 0x00, 0x00, 0x06];

pub struct NetCDF {
    pub(crate) header: NetCDFHeader,
    pub(crate) data: NetCDFData,
}

pub(crate) struct NetCDFHeader {
    pub(crate) version: NetCDFVersion,
    pub(crate) numrecs: NetCDFStreaming,
    pub(crate) dim_list: Vec<NetCDFDimension>,
    pub(crate) att_list: Vec<NetCDFAttribute>,
    pub(crate) var_list: Vec<NetCDFVariable>,
}

impl Display for NetCDF {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let version = match self.header.version {
            NetCDFVersion:: CDF01 => "1 (CDF01)",
            NetCDFVersion:: CDF02 => "2 (CDF02)",
            NetCDFVersion:: HDF5 => "4 (HDF5)",
        };
        write!(formatter, "Version: {}\n", version)
    }
}

#[derive(Debug)]
pub(crate) enum NetCDFVersion {
    CDF01,
    CDF02,
    HDF5,
}

#[derive(Debug)]
pub(crate) enum NetCDFStreaming {
    Streaming,
    Normal(u32),
}

#[derive(Debug)]
pub(crate) enum NetCDFType {
    NCByte,
    NCChar,
    NCShort,
    NCInt,
    NCFloat,
    NCDouble,
}

#[derive(Debug)]
pub enum NetCDFValue {
    Byte(u8),
    Char(char),
    Short(i16),
    Int(i32),
    Float(f32),
    Double(f64),
}

#[derive(Debug)]
pub(crate) struct NetCDFDimension {
    pub(crate) name: String,
    pub(crate) dim_length: u32,
}

#[derive(Debug)]
pub(crate) struct NetCDFAttribute {
    pub(crate) name: String,
    pub(crate) nc_type: NetCDFType,
    pub(crate) values: Vec<NetCDFValue>,
}

#[derive(Debug)]
pub(crate) struct NetCDFVariable {
    pub(crate) name: String,
    pub(crate) dimid: Vec<u32>,
    pub(crate) att_list: Vec<NetCDFAttribute>,
    pub(crate) nc_type: NetCDFType,
    pub(crate) vsize: u32,
    pub(crate) offset: NetCDFOffset,
}

#[derive(Debug)]
pub(crate) enum NetCDFOffset {
    Pos32(u32),
    Pos64(u64),
}

#[derive(Debug)]
pub(crate) struct NetCDFData {
    pub(crate) non_recs: Vec<NetCDFVarData>,
    pub(crate) recs: Vec<NetCDFRecord>,
}

#[derive(Debug)]
pub(crate) struct NetCDFVarData {
    pub(crate) values: Vec<NetCDFValue>,
}

#[derive(Debug)]
pub(crate) struct NetCDFRecord {
    pub(crate) record: Vec<NetCDFVarSlab>,
}

#[derive(Debug)]
pub(crate) struct NetCDFVarSlab {
    pub(crate) varslab: Vec<NetCDFValue>,
}

#[derive(Debug)]
pub enum NetCDFError {
    IOError(io::Error),
    UnknownVersion(FourBytes),
    FromUtf8(FromUtf8Error),
    DimListTag((FourBytes, FourBytes)),
    AttrListTag((FourBytes, FourBytes)),
    NCType(FourBytes),
    HDF5NotSupportetYet,
    UnknownOffsetVersion,
}


impl From<io::Error> for NetCDFError {
    fn from(error: io::Error) -> NetCDFError {
        NetCDFError::IOError(error)
    }
}

/*
impl From<FromUtf8Error> for NetCDFError {
    fn from(error: FromUtf8Error) -> NetCDFError {
        NetCDFError::FromUtf8(error)
    }
}
*/

impl Display for NetCDFError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            NetCDFError::IOError(e) => {
                write!(formatter, "IO error: {}", e)
            }
            NetCDFError::UnknownVersion(v) => {
                write!(formatter, "Unknown version: {:x?}", v)
            }
            NetCDFError::DimListTag((t1, t2)) => {
                write!(formatter, "Unknown tag in dim_list: {:x?}, {:x?}", t1, t2)
            }
            NetCDFError::AttrListTag((t1, t2)) => {
                write!(formatter, "Unknown tag in attr_list: {:x?}, {:x?}", t1, t2)
            }
            NetCDFError::FromUtf8(e) => {
                write!(formatter, "Could not convert to String: {}", e)
            }
            NetCDFError::NCType(t) => {
                write!(formatter, "Unknown NetCDF type: {:x?}", t)
            }
            NetCDFError::HDF5NotSupportetYet => {
                write!(formatter, "Version 4 with HDF5 is not supported yet")
            }
            NetCDFError::UnknownOffsetVersion => {
                write!(formatter, "The offset version is not known, must be old format version 1 or 2")
            }
        }
    }
}

impl NetCDF {
    pub fn num_of_records(&self) -> u32 {
        match self.header.numrecs {
            NetCDFStreaming::Streaming => 0,
            NetCDFStreaming::Normal(n) => n,
        }
    }

    pub fn num_of_dimensions(&self) -> u32 {
        self.header.dim_list.len() as u32
    }

    pub fn num_of_attributes(&self) -> u32 {
        self.header.att_list.len() as u32
    }

    pub fn num_of_variables(&self) -> u32 {
        self.header.var_list.len() as u32
    }

}
