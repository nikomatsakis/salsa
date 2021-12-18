#[salsa::jar(Error)]
pub struct ErrorJar(Diagnostics);

pub trait Error: salsa::DbWithJar<ErrorJar> {}
impl<T> Error for T where T: salsa::DbWithJar<ErrorJar> {}

#[derive(Clone, Debug)]
pub struct Diagnostic(pub String);

#[salsa::accumulator(in ErrorJar)]
pub struct Diagnostics(Diagnostic);
