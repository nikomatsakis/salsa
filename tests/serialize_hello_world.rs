//! Test that a `tracked` fn on a `salsa::input`
//! compiles and executes successfully.


use salsa::{Storage, Database, StorageBuilder};
use test_log::test;

#[salsa::db]
#[derive(Clone)]
struct SerializeDatabase {
    storage: Storage<Self>,
}

#[salsa::db]
impl Database for SerializeDatabase {
    fn salsa_event(&self, _event: &dyn Fn() -> salsa::Event) {}
}

impl SerializeDatabase {
    pub fn new() -> Self {
        Self {
            storage: StorageBuilder::new()
                .serializable::<MyInput>()
                // .serializable::<intermediate_result>()
                // .serializable::<final_result>()
                .serializable::<MyTracked<'static>>()
                .build(),
        }
    }
}

#[salsa::input]
struct MyInput {
    field: u32,
}

#[salsa::tracked]
fn intermediate_result(db: &dyn salsa::Database, input: MyInput) -> MyTracked<'_> {
    // db.push_log(format!("intermediate_result({:?})", input));
    MyTracked::new(db, input.field(db) / 2)
}

#[salsa::tracked]
fn final_result(db: &dyn salsa::Database, input: MyInput) -> u32 {
    // db.push_log(format!("final_result({:?})", input));
    intermediate_result(db, input).field(db) * 2
}

#[salsa::tracked]
struct MyTracked<'db> {
    field: u32,
}

#[test]
fn execute() {
    
}

/// Create and mutate a distinct input. No re-execution required.
#[test]
fn red_herring() {
    
}
