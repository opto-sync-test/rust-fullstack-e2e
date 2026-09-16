use ores_api_docs_client::{PageContext, PageDocument, PageResult};
use ores_api_docs_macros::ores_page;

#[ores_page(
    renderer = "mash",
    delivery = "ssr_only",
    render = "dynamic",
    title = "Runtime home",
    tags("opto-sync-test", "runtime")
)]
pub async fn page(_ctx: PageContext) -> PageResult {
    Ok(PageDocument::html(
        "<!doctype html><html><head><title>runtime</title></head><body><main>runtime-home</main></body></html>",
    ))
}
