use super::*;

pub(crate) fn main(db: &mut dyn Jar0Db) {
    let id = EntityData0 { id: 0 }.new(db);
    id.data(db);

    let ty = TyData0 { id: 0 }.intern(db);
    ty.data(db);
}
