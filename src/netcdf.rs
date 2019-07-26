

// Rust modules
use std::path::Path;
use std::fs::File;
use std::io;
use std::io::{BufReader, Read};


// External modules
use log::{info, debug, error};


// The netCDF format is described here:
// https://www.unidata.ucar.edu/software/netcdf/docs/file_format_specifications.html

pub struct NetCDF {
    version: NetCDFVersion,
    numrecs: u32,
    dim_list: Vec<NetCDFDimension>,
    att_list: Vec<NetCDFAttribute>,
    var_list: Vec<NetCDFVariable>,
    data: Vec<NetCDFData>,
}

#[derive(Debug)]
pub enum NetCDFVersion {
    CDF01,
    CDF02,
    Unknown,
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

pub enum NetCDFValues {
    Bytes(Vec<u8>),
    Chars(Vec<char>),
    Shorts(Vec<i16>),
    Ints(Vec<i16>),
    Floats(Vec<f32>),
    Doubles(Vec<f64>),
}

pub struct NetCDFDimension {
    name: String,
    dim_length: u32,
}

pub struct NetCDFAttribute {
    name: String,
    nc_type: NetCDFType,
    values: Vec<NetCDFValues>,
}

pub struct NetCDFVariable {
    name: String,
    dimid: Vec<u32>,
    att_list: Vec<NetCDFAttribute>,
    nc_type: NetCDFType,
    id: usize,
}

pub struct NetCDFData {
    non_recs: Vec<NetCDFVarData>,
    recs: Vec<NetCDFRecord>,
}

pub struct NetCDFVarData {
    values: Vec<NetCDFValues>,
}

pub struct NetCDFRecord {
    record: Vec<NetCDFVarSlab>,
}

pub struct NetCDFVarSlab {
    varslab: Vec<NetCDFValues>,
}

pub enum NetCDFError {
    IOError(io::Error),
}


impl From<io::Error> for NetCDFError {
    fn from(error: io::Error) -> NetCDFError {
        NetCDFError::IOError(error)
    }
}

impl NetCDF {
    pub fn load<T: AsRef<Path>>(path: T) -> Result<(), NetCDFError> {
        let file_path = path.as_ref();
        let file = File::open(file_path)?;
        let mut buf_reader = BufReader::new(file);

        let version = read_version(&mut buf_reader)?;
        info!("NetCDF version: {:?}", version);

        Ok(())
    }

}

fn read_version<T: Read>(reader: &mut T) -> Result<NetCDFVersion, NetCDFError> {
    let mut buffer = [0; 4];
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
            Ok(NetCDFVersion::Unknown)
        }
    }
}
