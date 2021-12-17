/////
///// Locations
/////

#[derive(Debug)]
pub enum LocationInfo {
    FileLineCol(String, usize, usize),
}
