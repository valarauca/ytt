
use scc::Guard;

pub fn guarded<F,O>(lambda: F) -> O
where
    F: FnOnce(&Guard) -> O,
{
    let g = Guard::new();
    let out: O = (lambda)(&g);
    if g.has_garbage() {
        g.accelerate();
    }
    out
}
