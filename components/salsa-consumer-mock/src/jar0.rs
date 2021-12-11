#[salsa::jar(Jar0Db)]
pub struct Jar0(
    Entity0,
    Ty0,
    EntityComponent0,
    my_func,
    my_func_ref,
    my_func_ref_eq,
);
pub trait Jar0Db: salsa::DbWithJar<Jar0> {}
impl<DB> Jar0Db for DB where DB: ?Sized + salsa::DbWithJar<Jar0> {}

#[salsa::entity(Entity0 in Jar0)]
#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct EntityData0 {
    // FIXME: structs and things have to be pub because of Rust's silly rules
    id: u32,
}

#[salsa::interned(Ty0 in Jar0)]
#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct TyData0 {
    id: u32,
}

#[salsa::component(EntityComponent0 in Jar0)]
impl Entity0 {
    fn method(self, _db: &dyn Jar0Db) -> String {
        format!("Hello, world")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NotClone(u32);

#[salsa::memoized(in Jar0 ref)]
fn my_func_ref(_db: &dyn Jar0Db, input1: u32, input2: u32) -> NotClone {
    NotClone(input1 + input2)
}

#[salsa::memoized(in Jar0)]
fn my_func(_db: &dyn Jar0Db, input1: u32, input2: u32) -> String {
    format!("Hello, world ({}, {})", input1, input2)
}

#[derive(Debug)]
pub struct NotCloneNotEq(u32);

#[salsa::memoized(in Jar0 ref no_eq)]
fn my_func_ref_eq(_db: &dyn Jar0Db, input1: u32, input2: u32) -> NotCloneNotEq {
    NotCloneNotEq(input1 + input2)
}
