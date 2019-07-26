

use netcdfrs::prelude::*;



fn main() {

    match NetCDF::load("version1.nc") {
        Err(e) => {
            // println!("An error occurred: {:?}", e);
        }
        Ok(net_cdf) => {

        }
    }

}
