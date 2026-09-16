use ores_api_docs_client::{PageContext, PageDocument, PageResult};
use ores_api_docs_macros::ores_page;

#[ores_page(
    renderer = "mash",
    delivery = "ssr_only",
    render = "dynamic",
    title = "Runtime user",
    tags("opto-sync-test", "runtime", "dynamic")
)]
pub async fn page(ctx: PageContext) -> PageResult {
    let id = ctx
        .route_params
        .get("id")
        .cloned()
        .unwrap_or_else(|| "missing".to_owned());
    Ok(PageDocument::html(format!(
        "<!doctype html><html><head><title>user</title></head><body><main>runtime-user:{id}</main></body></html>"
    )))
}
