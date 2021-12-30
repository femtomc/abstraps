pub enum BuiltinLattice {
    Float32,
    Float64,
    Int32,
    Int64,
    Function(Vec<BuiltinLattice>, Box<BuiltinLattice>),
    Vector(Vec<usize>, Box<BuiltinLattice>),
}
