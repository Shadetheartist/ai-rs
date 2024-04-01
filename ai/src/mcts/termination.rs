#[allow(dead_code)]
pub enum Outcome<P> {
    Winner(P),
    Winners(Vec<P>),
    Escape(String)
}