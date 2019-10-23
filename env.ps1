

$env:Path += Resolve-Path -Path ".\gdal\release-1900-x64-gdal-2-4-2-mapserver-7-4-0\bin"
$env:GDAL_LIB_DIR = Resolve-Path -Path ".\gdal\release-1900-x64-gdal-2-4-2-mapserver-7-4-0-libs\lib"
$env:GDAL_INCLUDE_DIR = Resolve-Path -Path ".\gdal\release-1900-x64-gdal-2-4-2-mapserver-7-4-0-libs\include"
$env:GDAL_DATA = Resolve-Path -Path ".\gdal\release-1900-x64-gdal-2-4-2-mapserver-7-4-0\bin\gdal-data"