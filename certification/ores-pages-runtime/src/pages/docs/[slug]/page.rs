use ores_api_docs_client::{PageContext, PageDocument, PageResult};
use ores_api_docs_macros::ores_page;

#[ores_page(
    renderer = "mash",
    delivery = "ssr_only",
    render = "static_only",
    title = "Generated docs",
    tags("opto-sync-test", "runtime", "prerender")
)]
pub async fn page(ctx: PageContext) -> PageResult {
    let slug = ctx
        .route_params
        .get("slug")
        .cloned()
        .unwrap_or_else(|| "missing".to_owned());
    Ok(PageDocument::html(format!(
        "<!doctype html><html><head><title>docs</title></head><body><main>runtime-doc:{slug}</main></body></html>"
    )))
}
