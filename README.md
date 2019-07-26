# netcdf-rs
100% pure Rust support for the netCDF file format







To show the format of a file:

ncdump -k foo.nc


To convert from netCDF-4 to netCDF-3:

nccopy -k netCDF-4 foo3.nc foo4.nc
nccopy -k netCDF-4-classic foo3.nc foo4c.nc
ncks -7 foo3.nc foo4c.nc
ncdump small3.nc > small.cdl
ncgen -o small4.nc -k netCDF-4-classic small.cdl



To convert from netCDF-3 to netCDF-4:

nccopy -k classic foo4c.nc foo3.nc
ncks -3 foo4c.nc foo3.nc
ncdump small4c.nc > small4.cdl
ncgen -o small3.nc small4.cdl


To convert from netCDF (HDF5) to netCDF (CDF1):

ncks infile.hdf5 outfile.nc


See FAQ: https://www.unidata.ucar.edu/software/netcdf/docs/faq.html#How-can-I-convert-netCDF-3-files-into-netCDF-4-files

