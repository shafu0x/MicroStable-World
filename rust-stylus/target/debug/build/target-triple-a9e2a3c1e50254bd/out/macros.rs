/// Produces the value of TARGET as a string literal.
#[macro_export]
macro_rules! target {
    () => {
        "aarch64-apple-darwin"
    };
}

/// Produces the value of HOST as a string literal.
#[macro_export]
macro_rules! host {
    () => {
        "aarch64-apple-darwin"
    };
}
