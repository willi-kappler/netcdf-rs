use std::path::Path;

// The netCDF format is described here:
// https://www.unidata.ucar.edu/software/netcdf/docs/file_format_specifications.html

pub enum NetCDFVersion {
    CDF01,
    CDF02,
    Unknown,
}

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

pub struct NetCDF {
    version: NetCDFVersion,
    numrecs: u32,
    dim_list: Vec<NetCDFDimension>,
    att_list: Vec<NetCDFAttribute>,
    var_list: Vec<NetCDFVariable>,
    data: Vec<NetCDFData>,
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


impl NetCDF {
    pub fn load<T: AsRef<Path>>(path: T) {
        let file_path = path.as_ref();
    }
}
