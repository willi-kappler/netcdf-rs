

// Rust modules
use std::path::Path;
use std::fs::File;
use std::{io::BufReader, io::Read};
// use std::{fmt, fmt::Display, fmt::Formatter};
// use std::string::FromUtf8Error;

// External modules
use log::{info, debug};
use byteorder::{ByteOrder, BigEndian};

// Internal modules
use crate::netcdf::*;


pub fn load_file<T: AsRef<Path>>(path: T) -> Result<NetCDF, NetCDFError> {
    let file_path = path.as_ref();
    info!("reader.rs, load_file, tryingo to open file: '{}'", file_path.display());
    let file = File::open(file_path)?;
    let mut buf_reader = BufReader::new(file);
    load_reader(&mut buf_reader)
}

pub fn load_reader<T: Read>(reader: &mut T) -> Result<NetCDF, NetCDFError> {
    let header = read_header(reader)?;
    let data = read_data(reader, &header)?;

    Ok(NetCDF{header, data})
}

fn read_header<T: Read>(reader: &mut T) -> Result<NetCDFHeader, NetCDFError> {
    let version = read_version(reader)?;
    info!("NetCDF version: {:?}", version);

    match version {
        NetCDFVersion::HDF5 => Err(NetCDFError::HDF5NotSupportetYet),
        _ => {
            let numrecs = read_numrecs(reader)?;
            info!("NetCDF number of records: {:?}", numrecs);

            let dim_list = read_dim_list(reader)?;
            let att_list = read_att_list(reader)?;
            let var_list = read_var_list(reader, &version)?;

            Ok(NetCDFHeader{version, numrecs, dim_list, att_list, var_list})
        }
    }
}

fn read_version<T: Read>(reader: &mut T) -> Result<NetCDFVersion, NetCDFError> {
    let mut buffer: FourBytes = [0; 4];
    reader.read_exact(&mut buffer)?;
    debug!("Version buffer: {:?}", buffer);

    match buffer {
        VERSION1 => Ok(NetCDFVersion::CDF01),
        VERSION2 => Ok(NetCDFVersion::CDF02),
        VERSION4 => Ok(NetCDFVersion::HDF5),
        _ => Err(NetCDFError::UnknownVersion(buffer))
    }
}

fn read_numrecs<T: Read>(reader: &mut T) -> Result<NetCDFStreaming, NetCDFError> {
    let mut buffer: FourBytes = [0; 4];
    reader.read_exact(&mut buffer)?;
    debug!("Numrecs buffer: {:?}", buffer);

    match buffer {
        STREAMING => Ok(NetCDFStreaming::Streaming),
        _ => {
            let value1 = u32::from_be_bytes(buffer);
            debug!("Numrecs BE: {}", value1);

            // let value2 = u32::from_le_bytes(buffer);
            // debug!("Numrecs LE: {}", value2);

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

fn read_data<T: Read>(reader: &mut T, header: &NetCDFHeader) -> Result<NetCDFData, NetCDFError> {
    let non_recs = read_non_records(reader)?;
    let recs = read_records(reader)?;

    Ok(NetCDFData{non_recs, recs})
}

fn read_name<T: Read>(reader: &mut T) -> Result<String, NetCDFError> {
    let mut buffer1: FourBytes = [0; 4];
    reader.read_exact(&mut buffer1)?;
    let name_length = u32::from_be_bytes(buffer1);

    let reader2 = reader.by_ref();
    let mut buffer2 = Vec::new();
    reader2.take(name_length as u64).read_to_end(&mut buffer2)?;
    String::from_utf8(buffer2).map_err(|e| NetCDFError::FromUtf8(e))
}

fn read_number_of_elements<T: Read>(reader: &mut T) -> Result<u32, NetCDFError> {
    let mut buffer: FourBytes = [0; 4];
    reader.read_exact(&mut buffer)?;
    Ok(u32::from_be_bytes(buffer))
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
        _ => Err(NetCDFError::NCType(buffer))
    }
}

fn read_values<T: Read>(reader: &mut T, nc_type: NetCDFType, nvals: u32) -> Result<Vec<NetCDFValue>, NetCDFError> {
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
        }
        NetCDFType::NCInt => {
            let mut buffer: FourBytes = [0; 4];

            for _ in 0..nvals {
                reader.read_exact(&mut buffer)?;
                result.push(NetCDFValue::Int(i32::from_be_bytes(buffer)))
            }
        }
        NetCDFType::NCFloat => {
            let mut buffer: FourBytes = [0; 4];

            for _ in 0..nvals {
                reader.read_exact(&mut buffer)?;
                result.push(NetCDFValue::Float(BigEndian::read_f32(&buffer)))
            }
        }
        NetCDFType::NCDouble => {
            let mut buffer: EightBytes = [0; 8];

            for _ in 0..nvals {
                reader.read_exact(&mut buffer)?;
                result.push(NetCDFValue::Double(BigEndian::read_f64(&buffer)))
            }
        }
    }

    Ok(result)
}

fn read_dimension<T: Read>(reader: &mut T) -> Result<NetCDFDimension, NetCDFError> {
    let name = read_name(reader)?;
    let length = read_number_of_elements(reader)?;
    Ok(NetCDFDimension{name, length})
}

fn read_attribute<T: Read>(reader: &mut T) -> Result<NetCDFAttribute, NetCDFError> {
    let name = read_name(reader)?;
    let nc_type = read_nc_type(reader)?;
    let nvals = read_number_of_elements(reader)?;
    let values = read_values(reader, nc_type, nvals)?;
    Ok(NetCDFAttribute{name, values})
}

fn read_variable<T: Read>(reader: &mut T, version: &NetCDFVersion) -> Result<NetCDFVariable, NetCDFError> {
    let name = read_name(reader)?;
    let dimid = read_dimension_ids(reader)?;
    let att_list = read_att_list(reader)?;
    let nc_type = read_nc_type(reader)?;
    let vsize = read_number_of_elements(reader)?;
    let offset = read_offset(reader, version)?;
    Ok(NetCDFVariable{name, dimid, att_list, nc_type, vsize, offset})
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
        _ => Err(NetCDFError::UnknownOffsetVersion)
    }
}

fn read_non_records<T: Read>(reader: &mut T) -> Result<Vec<NetCDFVarData>, NetCDFError> {
    let result = Vec::new();
    Ok(result)
}

fn read_records<T: Read>(reader: &mut T) -> Result<Vec<NetCDFRecord>, NetCDFError> {
    let result = Vec::new();
    Ok(result)
}
