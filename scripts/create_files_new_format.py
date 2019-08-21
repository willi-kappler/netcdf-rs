#!/bin/env python3

from netCDF4 import Dataset

data = Dataset("empty.nc", "w", format="NETCDF4")
data.close()

data = Dataset("small1.nc", "w", format="NETCDF4")
time = data.createDimension("time", 5)
times = data.createVariable("times","i2",("time",))
times[:] = [1, 2, 3, 90, 321]
data.close()

data = Dataset("small2.nc", "w", format="NETCDF4")
time = data.createDimension("time", 5)
temp = data.createDimension("temp", 5)
times = data.createVariable("times","i2",("time",))
times[:] = [1, 2, 3, 90, 321]
temps = data.createVariable("temps","i2",("temp",))
temps[:] = [30, 32, 34, 36, 40]
data.close()


