extern crate clap;
extern crate gdal;


use las::reader::Read;
use clap::App;
use std::path::{Path, PathBuf};
use las::Reader;
use std::io::prelude::*;
use std::result::Result;
use std::error::Error;
use gdal::vector::{Driver, FieldValue, Geometry, OGRFieldType, OGRwkbGeometryType};
use gdal::spatial_ref::SpatialRef;
use std::fmt;
use std::env;


enum LasBoundsError {
    GdalError(gdal::errors::Error),
    IOError(std::io::Error),
    LASError(las::Error)
}

impl fmt::Debug for LasBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for LasBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GdalError(e) => write!(f, "GdalError: TODO"),
            Self::IOError(e) => write!(f, "IOError: {}", e),
            Self::LASError(e) => write!(f, "LASError: {}", e)
        }
    }
}


impl Error for LasBoundsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::GdalError(_) => None,
            Self::IOError(e) => Some(e),
            Self::LASError(e) => Some(e)
        }
    }
}

impl From<std::io::Error> for LasBoundsError {
    fn from(error: std::io::Error) -> Self {
        Self::IOError(error)
    }
}

impl From<gdal::errors::Error> for LasBoundsError {
    fn from(error: gdal::errors::Error) -> Self {
        Self::GdalError(error)
    }
}

impl From<las::Error> for LasBoundsError {
    fn from(error: las::Error) -> Self {
        Self::LASError(error)
    }
}

fn build_app<'a, 'b>() -> clap::App<'a, 'b> {

    App::new("las-bounds")
    .version("0.0.0")
    .author("nemq")
    .about("Generates bounds of LAS files and saves them in ESRI Shapefiles.")
    .args_from_usage("<DIRECTORY>   'Directory containing LAS files.'")
    .args_from_usage("-e, --epsg <num>    'EPSG code of LAS coordinate system.")
}



fn list_las(dir: &Path) -> Result<Vec<PathBuf>, LasBoundsError> {

    let mut vec = Vec::new();
    for path in dir.read_dir()?
                   .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                   .filter(|path| path.extension().and_then(|ext| ext.to_str())
                   .filter(|&ext| ext == "las").is_some()) {

        vec.push(path);
    }

    Ok(vec)
}

fn read_bounds(las: &Path) -> Result<las::Bounds, LasBoundsError> {

    let reader = Reader::from_path(las)?;
    let header = reader.header();
    Ok(header.bounds())
}

fn write_bounds(shp: &Path, bounds: &las::Bounds, srs: Option<&SpatialRef>) ->Result<(), LasBoundsError> {

    let driver = Driver::get("ESRI Shapefile")?;
    let mut ds = driver.create(shp)?;
    let layer = ds.create_layer_ext("bounds", srs, OGRwkbGeometryType::wkbPolygon)?;

    layer.create_defn_fields(&[
        ("Name", OGRFieldType::OFTString),
    ])?;


    layer.create_feature_fields(
        Geometry::bbox(bounds.min.x, bounds.min.y, bounds.max.x, bounds.max.y)?,
        &["Name"],
        &[
            FieldValue::StringValue("BBOX".to_string()),
        ],
    )?;

    Ok(())
}

fn main() -> Result<(), LasBoundsError> {

    let app = build_app();
    let matches = app.get_matches();

    let dir_val = matches.value_of("DIRECTORY").unwrap();
    let dir_path = Path::new(&dir_val);
    println!("Searching in: {}", dir_val);

    let mut srs = None;
    if let Some(epsg) = matches.value_of("epsg").and_then(|s| (s.parse::<u32>().ok())) {
        srs = Some(SpatialRef::from_epsg(epsg)?);
    }

    for las in list_las(dir_path)? {
        let bounds = read_bounds(&las)?;
        let txt = las.with_extension("txt");
        let mut txt_file = std::fs::File::create(txt)?;
        txt_file.write_fmt(format_args!("{:?}", bounds))?;

        let shp = las.with_extension("shp");
        write_bounds(&shp, &bounds, srs.as_ref())?;
    }


    Ok(())
}