

mod netcdf;
mod reader;
mod writer;

pub mod prelude {
    pub use crate::netcdf::{NetCDF, NetCDFError};
    pub use crate::reader::{load_file, load_reader};
}
