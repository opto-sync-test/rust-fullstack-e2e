use std::collections::BTreeMap;

use ores_api_docs_client::{PrerenderContext, PrerenderPath, PrerenderResult};
use ores_api_docs_macros::ores_generate;

#[ores_generate]
pub async fn generate_static_params(_ctx: PrerenderContext) -> PrerenderResult {
    Ok(vec![PrerenderPath {
        route_params: BTreeMap::from([("slug".to_owned(), "intro".to_owned())]),
    }])
}
