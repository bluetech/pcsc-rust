use super::*;

// We can't really assume PC/SC is available on the machine running the
// tests, hence we don't fail on any error.
//
// The point is that at least we ensure it doesn't crash due to e.g. bad
// calling convention in the FFI.

#[test]
fn test_context() {
    let ctx = match Context::establish(Scope::User) {
        Err(_) => return,  // Skip.
        Ok(ctx) => ctx,
    };

    if let Err(_) = ctx.is_valid() {
        return;
    };

    if let Err(_) = ctx.release() {
        return;
    };
}
