use netcdfrs::prelude::*;

#[test]
fn empty() {
    let data = load_file("tests/version1/empty.nc").unwrap();


}

#[test]
fn small1() {
    let data = load_file("tests/version1/small1.nc").unwrap();

}

#[test]
fn small2() {
    let data = load_file("tests/version1/small2.nc").unwrap();

}
