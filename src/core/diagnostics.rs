/////
///// Locations
/////

#[derive(Debug, Hash)]
pub enum LocationInfo {
    Unknown,
    FileLineCol(String, usize, usize),
    NameFileLineCol(String, String, usize, usize),
    InlinedFrom(Vec<LocationInfo>),
}
