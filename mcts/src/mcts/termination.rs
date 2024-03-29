pub enum Termination<'p, P> {
    Winner(&'p P),
    Escape
}