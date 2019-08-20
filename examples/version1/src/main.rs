

// Local modules
use netcdfrs::prelude::*;

// External modules
use log::{info, error, debug};
use log4rs;


fn main() {
    let file_logger = log4rs::append::file::FileAppender::builder()
        .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new("{d} {l} - {m}{n}")))
        .build("version1.log").unwrap();
    let log_config = log4rs::config::Config::builder()
        .appender(log4rs::config::Appender::builder().build("file_logger", Box::new(file_logger)))
        .build(log4rs::config::Root::builder().appender("file_logger").build(log::LevelFilter::Debug))
        .unwrap();
    let _log_handle = log4rs::init_config(log_config).unwrap();



    match load_file("netcdf_files/empty.nc") {
        Err(e) => {
            error!("An error occurred: {}", e);
        }
        Ok(net_cdf) => {
            info!("File OK!");
            info!("NetCDF info:\n{}", net_cdf);
        }
    }



}
