mod db;
mod jar0;

fn main() {
    let mut db = db::TheDatabase::default();
    jar0::main::main(&mut db);
}
