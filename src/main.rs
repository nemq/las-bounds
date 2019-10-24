extern crate clap;
extern crate gdal;


use las::reader::Read;
use clap::App;
use std::path::{Path, PathBuf};
use las::Reader;
use std::result::Result;
use std::error::Error;
use gdal::vector::{Driver, Dataset, Layer, FieldValue, Geometry, OGRFieldType, OGRwkbGeometryType};
use gdal::spatial_ref::SpatialRef;
use std::fmt;


enum LasBoundsError {
    GdalError(gdal::errors::Error),
    IOError(std::io::Error),
    LASError(las::Error),
    Custom(String)
}

impl fmt::Debug for LasBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for LasBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GdalError(_e) => write!(f, "GdalError: TODO"),
            Self::IOError(e) => write!(f, "IOError: {}", e),
            Self::LASError(e) => write!(f, "LASError: {}", e),
            Self::Custom(s) => write!(f, "Custom: {}", s)
        }
    }
}


impl Error for LasBoundsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::GdalError(_) => None,
            Self::IOError(e) => Some(e),
            Self::LASError(e) => Some(e),
            Self::Custom(_) => None
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

impl From<String> for LasBoundsError {
    fn from(s: String) -> Self {
        Self::Custom(s)
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

fn create_shp (shp: &Path) -> Result<Dataset, LasBoundsError> {
    
    let driver = Driver::get("ESRI Shapefile")?;
    let ds = driver.create(shp)?;
    Ok(ds)
}

fn create_layer<'a>(ds: &'a mut Dataset, srs: Option<SpatialRef>) -> Result<&'a mut Layer, LasBoundsError> {

    let layer = ds.create_layer_ext("bounds", srs.as_ref(), OGRwkbGeometryType::wkbPolygon)?;

    layer.create_defn_fields(&[
        ("name", OGRFieldType::OFTString),
        ("path", OGRFieldType::OFTString),
    ])?;

    Ok(layer)
}

fn write_bounds(las: &Path, layer: &mut Layer) ->Result<(), LasBoundsError> {

    let bounds = read_bounds(las)?;
    let path = las.to_string_lossy().into_owned();
    let filename = las.file_name().ok_or(format!("Could not get file name: {}", path))?.to_string_lossy().into_owned();

    layer.create_feature_fields(
        Geometry::bbox(bounds.min.x, bounds.min.y, bounds.max.x, bounds.max.y)?,
        &["name", "path"],
        &[
            FieldValue::StringValue(filename.into()),
            FieldValue::StringValue(path.into())
        ],
    )?;

    Ok(())
}

fn main() -> Result<(), LasBoundsError> {

    let app = build_app();
    let matches = app.get_matches();

    let dir_val = matches.value_of("DIRECTORY").unwrap();
    let dir_path = Path::new(&dir_val);
    let shp_path = dir_path.with_extension("shp");

    let mut srs = None;
    if let Some(epsg) = matches.value_of("epsg").and_then(|s| (s.parse::<u32>().ok())) {
        srs = Some(SpatialRef::from_epsg(epsg)?);
    }

    let mut ds = create_shp(&shp_path)?;
    let mut layer = create_layer(&mut ds, srs)?;

    let paths = list_las(dir_path)?;
    for (i, p) in paths.iter().enumerate() {
        println!("[{}/{}] {}", i + 1, paths.len(), p.to_string_lossy());
        write_bounds(&p, &mut layer)?;
    }

    Ok(())
}