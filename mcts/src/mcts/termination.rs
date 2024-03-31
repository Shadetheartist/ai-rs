#[allow(dead_code)]
pub enum Outcome<'p, P> {
    Winner(&'p P),
    Winners(Vec<&'p P>),
    Escape(String)
}