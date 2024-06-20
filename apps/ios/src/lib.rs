uniffi::setup_scaffolding!();

#[uniffi::export]
pub fn hello(name: &str) -> String {
    format!("Hello, {name}!")
}
