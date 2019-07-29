

// Rust modules
use std::path::Path;
use std::fs::File;
use std::{io, io::BufReader, io::Read};
use std::{fmt, fmt::Display, fmt::Formatter};
use std::string::FromUtf8Error;

// External modules
use log::{info, debug, error};
// use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian, LittleEndian};


// The netCDF format is described here:
// https://www.unidata.ucar.edu/software/netcdf/docs/file_format_specifications.html


type FourBytes = [u8; 4];
type EightBytes = [u8; 8];

const STREAMING: FourBytes = [0xff, 0xff, 0xff, 0xff];
const ZERO: FourBytes = [0x00, 0x00, 0x00, 0x00];
const NC_DIMENSION: FourBytes = [0x00, 0x00, 0x00, 0x0a];
const NC_VARIABLE: FourBytes = [0x00, 0x00, 0x00, 0x0b];
const NC_ATTRIBUTE: FourBytes = [0x00, 0x00, 0x00, 0x0c];

pub struct NetCDF {
    version: NetCDFVersion,
    numrecs: NetCDFStreaming,
    dim_list: Vec<NetCDFDimension>,
    att_list: Vec<NetCDFAttribute>,
    var_list: Vec<NetCDFVariable>,
    data: Vec<NetCDFData>,
}

impl Display for NetCDF {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(formatter, "Version: {:x?}", self.version)?;
        write!(formatter, "Number of records: {:?}", self.numrecs)
    }
}

#[derive(Debug)]
pub enum NetCDFVersion {
    CDF01,
    CDF02,
    Unknown,
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
pub enum NetCDFValues {
    Bytes(Vec<u8>),
    Chars(Vec<char>),
    Shorts(Vec<i16>),
    Ints(Vec<i16>),
    Floats(Vec<f32>),
    Doubles(Vec<f64>),
}

#[derive(Debug)]
pub struct NetCDFDimension {
    name: String,
    dim_length: u32,
}

#[derive(Debug)]
pub struct NetCDFAttribute {
    name: String,
    nc_type: NetCDFType,
    values: Vec<NetCDFValues>,
}

#[derive(Debug)]
pub struct NetCDFVariable {
    name: String,
    dimid: Vec<u32>,
    att_list: Vec<NetCDFAttribute>,
    nc_type: NetCDFType,
    id: usize,
}

#[derive(Debug)]
pub struct NetCDFData {
    non_recs: Vec<NetCDFVarData>,
    recs: Vec<NetCDFRecord>,
}

#[derive(Debug)]
pub struct NetCDFVarData {
    values: Vec<NetCDFValues>,
}

#[derive(Debug)]
pub struct NetCDFRecord {
    record: Vec<NetCDFVarSlab>,
}

#[derive(Debug)]
pub struct NetCDFVarSlab {
    varslab: Vec<NetCDFValues>,
}

#[derive(Debug)]
pub enum NetCDFError {
    IOError(io::Error),
    UnknownVersion(FourBytes),
    DimListTag((FourBytes, FourBytes)),
    FromUtf8(FromUtf8Error),
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
            NetCDFError::FromUtf8(e) => {
                write!(formatter, "Could not convert to String: {}", e)
            }
        }
    }
}

impl NetCDF {
    pub fn load<T: AsRef<Path>>(path: T) -> Result<NetCDF, NetCDFError> {
        let file_path = path.as_ref();
        let file = File::open(file_path)?;
        let mut buf_reader = BufReader::new(file);

        let version = read_version(&mut buf_reader)?;
        info!("NetCDF version: {:?}", version);

        let numrecs = read_numrecs(&mut buf_reader)?;
        info!("Number of records: {:?}", numrecs);

        let dim_list = read_dim_list(&mut buf_reader)?;

        let att_list = read_att_list(&mut buf_reader)?;

        let var_list = read_var_list(&mut buf_reader)?;

        let data = read_data(&mut buf_reader)?;

        let result = NetCDF {
            version,
            numrecs,
            dim_list,
            att_list,
            var_list,
            data,
        };

        Ok(result)
    }
}

fn read_version<T: Read>(reader: &mut T) -> Result<NetCDFVersion, NetCDFError> {
    let mut buffer: FourBytes = [0; 4];
    reader.read_exact(&mut buffer)?;
    debug!("Version buffer: {:?}", buffer);

    match buffer {
        [0x43, 0x44, 0x46, 0x01] => {
            Ok(NetCDFVersion::CDF01)
        }
        [0x43, 0x44, 0x46, 0x02] => {
            Ok(NetCDFVersion::CDF02)
        }
        _ => {
            Err(NetCDFError::UnknownVersion(buffer))
        }
    }
}

fn read_numrecs<T: Read>(reader: &mut T) -> Result<NetCDFStreaming, NetCDFError> {
    let mut buffer: FourBytes = [0; 4];
    reader.read_exact(&mut buffer)?;
    debug!("Numrecs buffer: {:?}", buffer);

    match buffer {
        [0xff, 0xff, 0xff, 0xff] => {
            Ok(NetCDFStreaming::Streaming)
        }
        _ => {
            let value1 = u32::from_be_bytes(buffer);
            debug!("Numrecs BE: {}", value1);

            let value2 = u32::from_le_bytes(buffer);
            debug!("Numrecs LE: {}", value2);

            // Big Endian is correct
            Ok(NetCDFStreaming::Normal(value1))
        }
    }
}

fn read_dim_list<T: Read>(reader: &mut T) -> Result<Vec<NetCDFDimension>, NetCDFError> {
    let mut result = Vec::new();
    let mut buffer1: FourBytes = [0; 4];
    let mut buffer2: FourBytes = [0; 4];
    reader.read_exact(&mut buffer1)?;
    reader.read_exact(&mut buffer2)?;
    debug!("Dimlist buffer1: {:?}", buffer1);
    debug!("Dimlist buffer2: {:?}", buffer2);

    match (buffer1, buffer2) {
        (ZERO, ZERO) => {
            // No dimensions given, return empty vector
            Ok(result)
        }
        (NC_DIMENSION, _) => {
            let nelem = u32::from_be_bytes(buffer2);
            debug!("Nelems dimlist BE: {}", nelem);

            let mut buffer3: FourBytes = [0; 4];

            for i in 0..nelem {
                reader.read_exact(&mut buffer3)?;
                let name_length = u32::from_be_bytes(buffer3);
                let name = read_name(name_length, reader)?;
                reader.read_exact(&mut buffer3)?;
                let dim_length = u32::from_be_bytes(buffer3);
                result.push(NetCDFDimension {
                    name,
                    dim_length,
                });
            }

            Ok(result)
        }
        _ => {
            Err(NetCDFError::DimListTag((buffer1, buffer2)))
        }
    }
}

fn read_att_list<T: Read>(reader: &mut T) -> Result<Vec<NetCDFAttribute>, NetCDFError> {
    let mut result = Vec::new();

    Ok(result)
}

fn read_var_list<T: Read>(reader: &mut T) -> Result<Vec<NetCDFVariable>, NetCDFError> {
    let mut result = Vec::new();

    Ok(result)
}

fn read_data<T: Read>(reader: &mut T) -> Result<Vec<NetCDFData>, NetCDFError> {
    let mut result = Vec::new();

    Ok(result)
}

fn read_name<T: Read>(name_length: u32, reader: &mut T) -> Result<String, NetCDFError> {
    let mut reader2 = reader.by_ref();
    let mut buffer = Vec::new();
    reader2.take(name_length as u64).read_to_end(&mut buffer)?;
    String::from_utf8(buffer).map_err(|e| NetCDFError::FromUtf8(e))
}
