

// Rust modules
// use std::path::Path;
// use std::fs::File;
use std::io;
use std::{fmt, fmt::Display, fmt::Formatter};
use std::string::FromUtf8Error;

// The netCDF format is described here:
// https://www.unidata.ucar.edu/software/netcdf/docs/file_format_specifications.html

pub type OneByte = [u8; 1];
pub type TwoBytes = [u8; 2];
pub type FourBytes = [u8; 4];
pub type EightBytes = [u8; 8];

pub const STREAMING: FourBytes = [0xff, 0xff, 0xff, 0xff];
pub const ZERO: FourBytes = [0x00, 0x00, 0x00, 0x00];
pub const VERSION1: FourBytes = [0x43, 0x44, 0x46, 0x01];
pub const VERSION2: FourBytes = [0x43, 0x44, 0x46, 0x02];

pub const NC_DIMENSION: FourBytes = [0x00, 0x00, 0x00, 0x0a];
pub const NC_VARIABLE: FourBytes = [0x00, 0x00, 0x00, 0x0b];
pub const NC_ATTRIBUTE: FourBytes = [0x00, 0x00, 0x00, 0x0c];

pub const NC_BYTE: FourBytes = [0x00, 0x00, 0x00, 0x01];
pub const NC_CHAR: FourBytes = [0x00, 0x00, 0x00, 0x02];
pub const NC_SHORT: FourBytes = [0x00, 0x00, 0x00, 0x03];
pub const NC_INT: FourBytes = [0x00, 0x00, 0x00, 0x04];
pub const NC_FLOAT: FourBytes = [0x00, 0x00, 0x00, 0x05];
pub const NC_DOUBLE: FourBytes = [0x00, 0x00, 0x00, 0x06];

pub struct NetCDF {
    pub header: NetCDFHeader,
    pub data: NetCDFData,
}

pub struct NetCDFHeader {
    pub version: NetCDFVersion,
    pub numrecs: NetCDFStreaming,
    pub dim_list: Vec<NetCDFDimension>,
    pub att_list: Vec<NetCDFAttribute>,
    pub var_list: Vec<NetCDFVariable>,
}

impl Display for NetCDF {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(formatter, "Version: {:x?}\n", self.header.version)?;
        write!(formatter, "Number of records: {:?}\n", self.header.numrecs)?;
        write!(formatter, "Number of dimensions: {}\n", self.header.dim_list.len())?;
        write!(formatter, "Number of attributes: {}\n", self.header.att_list.len())?;
        write!(formatter, "Number of variables: {}\n", self.header.var_list.len())
    }
}

#[derive(Debug)]
pub enum NetCDFVersion {
    CDF01,
    CDF02,
}

#[derive(Debug)]
pub enum NetCDFStreaming {
    Streaming,
    Normal(u32),
}

#[derive(Debug)]
pub enum NetCDFType {
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
pub struct NetCDFDimension {
    pub name: String,
    pub dim_length: u32,
}

#[derive(Debug)]
pub struct NetCDFAttribute {
    pub name: String,
    pub nc_type: NetCDFType,
    pub values: Vec<NetCDFValue>,
}

#[derive(Debug)]
pub struct NetCDFVariable {
    pub name: String,
    pub dimid: Vec<u32>,
    pub att_list: Vec<NetCDFAttribute>,
    pub nc_type: NetCDFType,
    pub vsize: u32,
    pub offset: NetCDFOffset,
}

#[derive(Debug)]
pub enum NetCDFOffset {
    Pos32(u32),
    Pos64(u64),
}

#[derive(Debug)]
pub struct NetCDFData {
    pub non_recs: Vec<NetCDFVarData>,
    pub recs: Vec<NetCDFRecord>,
}

#[derive(Debug)]
pub struct NetCDFVarData {
    pub values: Vec<NetCDFValue>,
}

#[derive(Debug)]
pub struct NetCDFRecord {
    pub record: Vec<NetCDFVarSlab>,
}

#[derive(Debug)]
pub struct NetCDFVarSlab {
    pub varslab: Vec<NetCDFValue>,
}

#[derive(Debug)]
pub enum NetCDFError {
    IOError(io::Error),
    UnknownVersion(FourBytes),
    FromUtf8(FromUtf8Error),
    DimListTag((FourBytes, FourBytes)),
    AttrListTag((FourBytes, FourBytes)),
    NCType(FourBytes),
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
        }
    }
}

