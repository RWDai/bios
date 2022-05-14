use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_app_dto::{IamAppDetailResp, IamAppModifyReq};
use crate::basic::dto::iam_filer_dto::IamAppFilterReq;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::console_app::dto::iam_ca_app_dto::IamCaAppModifyReq;
use crate::iam_constants;

pub struct IamCaAppApi;

/// App Console App API
#[OpenApi(prefix_path = "/ca/app", tag = "crate::iam_enumeration::Tag::App")]
impl IamCaAppApi {
    /// Modify Current App
    #[oai(path = "/", method = "put")]
    async fn modify(&self, modify_req: Json<IamCaAppModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAppServ::modify_item(
            &IamAppServ::get_id_by_cxt(&cxt.0)?,
            &mut IamAppModifyReq {
                name: modify_req.0.name.clone(),
                icon: modify_req.0.icon.clone(),
                sort: modify_req.0.sort,
                contact_phone: modify_req.0.contact_phone.clone(),
                disabled: modify_req.0.disabled,
                scope_level: None,
            },
            &funs,
            &cxt.0,
        )
        .await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Current App
    #[oai(path = "/", method = "get")]
    async fn get(&self, cxt: TardisContextExtractor) -> TardisApiResult<IamAppDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAppServ::get_item(&IamAppServ::get_id_by_cxt(&cxt.0)?, &IamAppFilterReq::default(), &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }
}