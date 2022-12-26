use bios_basic::spi::{api::spi_ci_bs_api, dto::spi_bs_dto::SpiBsCertResp, spi_constants, spi_funs::SpiBsInst, spi_initializer};
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    web::web_server::TardisWebServer,
    TardisFuns, TardisFunsInst,
};

use crate::{api::ci::api::search_item_api, search_constants::DOMAIN_CODE, serv};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    funs.begin().await?;
    let ctx = spi_initializer::init(DOMAIN_CODE, &funs).await?;
    init_db(&funs, &ctx).await?;
    funs.commit().await?;
    init_api(web_server).await
}

async fn init_db(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    spi_initializer::add_kind(spi_constants::SPI_PG_KIND_CODE, funs, ctx).await?;
    Ok(())
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server.add_module(DOMAIN_CODE, (spi_ci_bs_api::SpiCiBsApi, search_item_api::SearchCiItemApi)).await;
    Ok(())
}

pub async fn init_fun(bs_cert: SpiBsCertResp, ctx: &TardisContext) -> TardisResult<SpiBsInst> {
    match bs_cert.kind_code.as_str() {
        #[cfg(feature = "spi-pg")]
        "spi-pg" => serv::pg::search_pg_initializer::init(&bs_cert, ctx).await,
        _ => Err(TardisError::not_implemented(
            &format!("Backend service kind {} does not exist or SPI feature is not enabled", bs_cert.kind_code),
            "406-rbum-*-enum-init-error",
        ))?,
    }
}