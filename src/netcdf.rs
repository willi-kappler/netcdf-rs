

// Rust modules
use std::path::Path;
use std::fs::File;
use std::{io, io::BufReader, io::Read};
use std::{fmt, fmt::Display, fmt::Formatter};
use std::string::FromUtf8Error;

// External modules
use log::{info, debug, error};
use byteorder::{ByteOrder, BigEndian};


// The netCDF format is described here:
// https://www.unidata.ucar.edu/software/netcdf/docs/file_format_specifications.html


type OneByte = [u8; 1];
type TwoBytes = [u8; 2];
type FourBytes = [u8; 4];
type EightBytes = [u8; 8];

const STREAMING: FourBytes = [0xff, 0xff, 0xff, 0xff];
const ZERO: FourBytes = [0x00, 0x00, 0x00, 0x00];

const NC_DIMENSION: FourBytes = [0x00, 0x00, 0x00, 0x0a];
const NC_VARIABLE: FourBytes = [0x00, 0x00, 0x00, 0x0b];
const NC_ATTRIBUTE: FourBytes = [0x00, 0x00, 0x00, 0x0c];

const NC_BYTE: FourBytes = [0x00, 0x00, 0x00, 0x01];
const NC_CHAR: FourBytes = [0x00, 0x00, 0x00, 0x02];
const NC_SHORT: FourBytes = [0x00, 0x00, 0x00, 0x03];
const NC_INT: FourBytes = [0x00, 0x00, 0x00, 0x04];
const NC_FLOAT: FourBytes = [0x00, 0x00, 0x00, 0x05];
const NC_DOUBLE: FourBytes = [0x00, 0x00, 0x00, 0x06];

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
    name: String,
    dim_length: u32,
}

#[derive(Debug)]
pub struct NetCDFAttribute {
    name: String,
    nc_type: NetCDFType,
    values: Vec<NetCDFValue>,
}

#[derive(Debug)]
pub struct NetCDFVariable {
    name: String,
    dimid: Vec<u32>,
    att_list: Vec<NetCDFAttribute>,
    nc_type: NetCDFType,
    vsize: u32,
    offset: NetCDFOffset,
}

#[derive(Debug)]
pub enum NetCDFOffset {
    Pos32(u32),
    Pos64(u64),
}

#[derive(Debug)]
pub struct NetCDFData {
    non_recs: Vec<NetCDFVarData>,
    recs: Vec<NetCDFRecord>,
}

#[derive(Debug)]
pub struct NetCDFVarData {
    values: Vec<NetCDFValue>,
}

#[derive(Debug)]
pub struct NetCDFRecord {
    record: Vec<NetCDFVarSlab>,
}

#[derive(Debug)]
pub struct NetCDFVarSlab {
    varslab: Vec<NetCDFValue>,
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

        let var_list = read_var_list(&mut buf_reader, &version)?;

        let data = read_data(&mut buf_reader)?;

        let result = NetCDF{
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

            for _ in 0..nelem {
                let dimension = read_dimension(reader)?;
                result.push(dimension);
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
    let mut buffer1: FourBytes = [0; 4];
    let mut buffer2: FourBytes = [0; 4];
    reader.read_exact(&mut buffer1)?;
    reader.read_exact(&mut buffer2)?;
    debug!("Attlist buffer1: {:?}", buffer1);
    debug!("Attlist buffer2: {:?}", buffer2);

    match (buffer1, buffer2) {
        (ZERO, ZERO) => {
            // No attributes given, return empty vector
            Ok(result)
        }
        (NC_ATTRIBUTE, _) => {
            let nelem = u32::from_be_bytes(buffer2);
            debug!("Nelems attlist BE: {}", nelem);

            for _ in 0..nelem {
                let attribute = read_attribute(reader)?;
                result.push(attribute);
            }

            Ok(result)
        }
        _ => {
            Err(NetCDFError::AttrListTag((buffer1, buffer2)))
        }
    }
}

fn read_var_list<T: Read>(reader: &mut T, version: &NetCDFVersion) -> Result<Vec<NetCDFVariable>, NetCDFError> {
    let mut result = Vec::new();
    let mut buffer1: FourBytes = [0; 4];
    let mut buffer2: FourBytes = [0; 4];
    reader.read_exact(&mut buffer1)?;
    reader.read_exact(&mut buffer2)?;
    debug!("Varlist buffer1: {:?}", buffer1);
    debug!("Varlist buffer2: {:?}", buffer2);

    match (buffer1, buffer2) {
        (ZERO, ZERO) => {
            // No attributes given, return empty vector
            Ok(result)
        }
        (NC_VARIABLE, _) => {
            let nelem = u32::from_be_bytes(buffer2);
            debug!("Nelems varlist BE: {}", nelem);

            for _ in 0..nelem {
                let attribute = read_variable(reader, version)?;
                result.push(attribute);
            }

            Ok(result)
        }
        _ => {
            Err(NetCDFError::AttrListTag((buffer1, buffer2)))
        }
    }
}

fn read_data<T: Read>(reader: &mut T) -> Result<Vec<NetCDFData>, NetCDFError> {
    let mut result = Vec::new();

    // TODO: implement

    Ok(result)
}

fn read_name<T: Read>(reader: &mut T) -> Result<String, NetCDFError> {
    let mut buffer1: FourBytes = [0; 4];
    reader.read_exact(&mut buffer1)?;
    let name_length = u32::from_be_bytes(buffer1);

    let mut reader2 = reader.by_ref();
    let mut buffer2 = Vec::new();
    reader2.take(name_length as u64).read_to_end(&mut buffer2)?;
    String::from_utf8(buffer2).map_err(|e| NetCDFError::FromUtf8(e))
}

fn read_number_of_elements<T: Read>(reader: &mut T) -> Result<u32, NetCDFError> {
    let mut buffer: FourBytes = [0; 4];
    reader.read_exact(&mut buffer)?;
    let result = u32::from_be_bytes(buffer);
    Ok(result)
}

fn read_nc_type<T: Read>(reader: &mut T) -> Result<NetCDFType, NetCDFError> {
    let mut buffer: FourBytes = [0; 4];
    reader.read_exact(&mut buffer)?;

    match buffer {
        NC_BYTE => Ok(NetCDFType::NCByte),
        NC_CHAR => Ok(NetCDFType::NCChar),
        NC_SHORT => Ok(NetCDFType::NCShort),
        NC_INT => Ok(NetCDFType::NCInt),
        NC_FLOAT => Ok(NetCDFType::NCFloat),
        NC_DOUBLE => Ok(NetCDFType::NCDouble),
        _ => Err(NetCDFError::NCType(buffer)),
    }
}

fn read_values<T: Read>(reader: &mut T, nc_type: &NetCDFType, nvals: u32) -> Result<Vec<NetCDFValue>, NetCDFError> {
    let mut result = Vec::new();

    match nc_type {
        NetCDFType::NCByte => {
            let size_in_bytes = nvals;
            let padding = size_in_bytes % 4;

            let mut buffer: OneByte = [0; 1];

            for _ in 0..nvals {
                reader.read_exact(&mut buffer)?;
                result.push(NetCDFValue::Byte(buffer[0]))
            }

            for _ in 0..padding {
                // Ignore padding fill bytes
                reader.read_exact(&mut buffer)?;
            }

            Ok(result)
        }
        NetCDFType::NCChar => {
            let size_in_bytes = nvals;
            let padding = size_in_bytes % 4;

            let mut buffer: OneByte = [0; 1];

            for _ in 0..nvals {
                reader.read_exact(&mut buffer)?;
                result.push(NetCDFValue::Char(buffer[0] as char))
            }

            for _ in 0..padding {
                // Ignore padding fill bytes
                reader.read_exact(&mut buffer)?;
            }

            Ok(result)
        }
        NetCDFType::NCShort => {
            let size_in_bytes = nvals * 2;
            let padding = size_in_bytes % 4;

            let mut buffer: TwoBytes = [0; 2];

            for _ in 0..nvals {
                reader.read_exact(&mut buffer)?;
                result.push(NetCDFValue::Short(i16::from_be_bytes(buffer)))
            }

            if padding == 2 {
                // Ignore padding fill bytes
                // Padding can only be 0 or 2
                // and if it is 2 ready exactly 2 bytes.
                // Buffer size is also 2
                reader.read_exact(&mut buffer)?;
            }

            Ok(result)
        }
        NetCDFType::NCInt => {
            let mut buffer: FourBytes = [0; 4];

            for _ in 0..nvals {
                reader.read_exact(&mut buffer)?;
                result.push(NetCDFValue::Int(i32::from_be_bytes(buffer)))
            }

            Ok(result)
        }
        NetCDFType::NCFloat => {
            let mut buffer: FourBytes = [0; 4];

            for _ in 0..nvals {
                reader.read_exact(&mut buffer)?;
                result.push(NetCDFValue::Float(BigEndian::read_f32(&buffer)))
            }

            Ok(result)
        }
        NetCDFType::NCDouble => {
            let mut buffer: EightBytes = [0; 8];

            for _ in 0..nvals {
                reader.read_exact(&mut buffer)?;
                result.push(NetCDFValue::Double(BigEndian::read_f64(&buffer)))
            }

            Ok(result)
        }
    }
}

fn read_dimension<T: Read>(reader: &mut T) -> Result<NetCDFDimension, NetCDFError> {
    let name = read_name(reader)?;
    let dim_length = read_number_of_elements(reader)?;
    let result = NetCDFDimension{name, dim_length};
    Ok(result)
}

fn read_attribute<T: Read>(reader: &mut T) -> Result<NetCDFAttribute, NetCDFError> {
    let name = read_name(reader)?;
    let nc_type = read_nc_type(reader)?;
    let nvals = read_number_of_elements(reader)?;
    let values = read_values(reader, &nc_type, nvals)?;
    let result = NetCDFAttribute{name, nc_type, values};
    Ok(result)
}

fn read_variable<T: Read>(reader: &mut T, version: &NetCDFVersion) -> Result<NetCDFVariable, NetCDFError> {
    let name = read_name(reader)?;
    let dimid = read_dimension_ids(reader)?;
    let att_list = read_att_list(reader)?;
    let nc_type = read_nc_type(reader)?;
    let vsize = read_number_of_elements(reader)?;
    let offset = read_offset(reader, version)?;
    let result = NetCDFVariable{name, dimid, att_list, nc_type, vsize, offset};
    Ok(result)
}

fn read_dimension_ids<T: Read>(reader: &mut T) -> Result<Vec<u32>, NetCDFError> {
    let mut result = Vec::new();
    let mut buffer: FourBytes = [0; 4];
    let nelems = read_number_of_elements(reader)?;

    for _ in 0..nelems {
        reader.read_exact(&mut buffer)?;
        let dim_id = u32::from_be_bytes(buffer);
        result.push(dim_id);
    }

    Ok(result)
}

fn read_offset<T: Read>(reader: &mut T, version: &NetCDFVersion) -> Result<NetCDFOffset, NetCDFError> {
    match version {
        NetCDFVersion::CDF01 => {
            let mut buffer: FourBytes = [0; 4];
            reader.read_exact(&mut buffer)?;
            let offset = u32::from_be_bytes(buffer);
            Ok(NetCDFOffset::Pos32(offset))
        }
        NetCDFVersion::CDF02 => {
            let mut buffer: EightBytes = [0; 8];
            reader.read_exact(&mut buffer)?;
            let offset = u64::from_be_bytes(buffer);
            Ok(NetCDFOffset::Pos64(offset))
        }
    }
}
