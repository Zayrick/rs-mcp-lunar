#[cfg(target_arch = "wasm32")]
pub mod cloudflare;
#[cfg(not(target_arch = "wasm32"))]
pub mod http;
