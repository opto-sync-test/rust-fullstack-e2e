use std::env;

include!(concat!(env!("OUT_DIR"), "/ores_pages.rs"));

#[tokio::main]
async fn main() {
    let bind = env::var("ORES_RUNTIME_ADDR").unwrap_or_else(|_| "127.0.0.1:3210".to_owned());
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .expect("bind generated page runtime");
    println!("ORES_PAGES_RUNTIME_READY {bind}");
    println!("ORES_TEST_USER_WASM_SHA {}", env!("ORES_TEST_USER_WASM_SHA"));
    println!("ORES_TEST_USER_WASM_PATH {}", env!("ORES_TEST_USER_WASM_PATH"));
    println!("ORES_TEST_USER_JS_PATH {}", env!("ORES_TEST_USER_JS_PATH"));
    axum::serve(listener, ores_pages_router::<()>())
        .await
        .expect("serve generated page runtime");
}
