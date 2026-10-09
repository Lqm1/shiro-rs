//! Binary model input shared by native SHIRO commands.
use liblrhsmm_rs::Model;
use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};

pub fn read_model(path: &Path) -> io::Result<Model> {
    if path.as_os_str() == "-" {
        Model::read_from(io::stdin().lock())
    } else {
        Model::read_from(BufReader::new(File::open(path)?))
    }
}
