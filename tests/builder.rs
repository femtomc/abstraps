use {abstraps, abstraps::builder::ExtIRBuilder, abstraps::ir::Operator, serde::Serialize};

#[derive(Debug, Serialize)]
pub enum FakeIntrinsic {
    Fake,
}

#[derive(Debug, Serialize)]
pub enum FakeAttribute {
    Fake,
}

#[test]
fn build_0() {
    let mut builder = ExtIRBuilder::<FakeIntrinsic, FakeAttribute>::default();
    let v = builder.push_arg();
    builder.build_instr(
        Operator::Intrinsic(FakeIntrinsic::Fake),
        vec![v],
        Vec::new(),
    );
    builder.push_blk();
    let v = builder.push_arg();
    builder.build_instr(
        Operator::Intrinsic(FakeIntrinsic::Fake),
        vec![v],
        Vec::new(),
    );
    let serialized = serde_json::to_string(&builder).unwrap();
    println!("{}", serialized);
}
