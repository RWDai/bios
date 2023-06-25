use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumKindAttrFilterReq, RbumKindFilterReq};
use bios_basic::rbum::dto::rbum_kind_attr_dto::RbumKindAttrSummaryResp;
use bios_basic::rbum::dto::rbum_kind_dto::RbumKindSummaryResp;
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_kind_serv::{RbumKindAttrServ, RbumKindServ};
use bios_basic::TardisFunInstExtractor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::Json;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::plugin_kind_dto::{PluginKindAddAggReq, PluginKindAggResp};
use crate::plugin_constants::KIND_MODULE_CODE;
use crate::serv::plugin_kind_serv::PluginKindServ;

pub struct PluginKindApi;

/// Plugin kind API
#[poem_openapi::OpenApi(prefix_path = "/ci/kind", tag = "bios_basic::ApiTag::Interface")]
impl PluginKindApi {
    /// add Plugin kind agg
    #[oai(path = "/agg", method = "put")]
    async fn add_agg(&self, add_req: Json<PluginKindAddAggReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        PluginKindServ::add_kind_agg_rel(&add_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// delete Plugin kind rel
    #[oai(path = "/:kind_id", method = "delete")]
    async fn delete_rel(&self, kind_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = request.tardis_fun_inst();
        PluginKindServ::delete_kind_agg_rel(&kind_id.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// find Plugin kind agg
    #[oai(path = "/agg", method = "get")]
    async fn find_agg(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<PluginKindAggResp>> {
        let funs = request.tardis_fun_inst();
        let result = PluginKindServ::find_kind_agg(&rbum_scope_helper::get_max_level_id_by_context(&ctx.0).unwrap_or_default(), &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// find Plugin kind
    #[oai(path = "/", method = "get")]
    async fn find_page(
        &self,
        code: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<RbumKindSummaryResp>> {
        let funs = request.tardis_fun_inst();
        let result = RbumKindServ::paginate_rbums(
            &RbumKindFilterReq {
                basic: RbumBasicFilterReq {
                    code: code.0,
                    ..Default::default()
                },
                module: Some(KIND_MODULE_CODE.to_string()),
            },
            page_number.0,
            page_size.0,
            None,
            None,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// find Plugin kind attr
    #[oai(path = "/attr/:kind_id", method = "get")]
    async fn find_kind_attr(&self, kind_id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<RbumKindAttrSummaryResp>> {
        let funs = request.tardis_fun_inst();
        let result = RbumKindAttrServ::find_rbums(
            &RbumKindAttrFilterReq {
                basic: RbumBasicFilterReq {
                    rbum_kind_id: Some(kind_id.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(result)
    }
}
