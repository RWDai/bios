use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::iam_constants;
use crate::basic::dto::iam_filer_dto::IamHttpResFilterReq;
use crate::basic::dto::iam_http_res_dto::{IamHttpResAddReq, IamHttpResDetailResp, IamHttpResModifyReq, IamHttpResSummaryResp};
use crate::iam_enumeration::IAMRelKind;
use crate::basic::serv::iam_http_res_serv::IamHttpResServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_tenant::dto::iam_ct_http_res_dto::{IamCtHttpResAddReq, IamCtHttpResModifyReq};

pub struct IamCtHttpResServ;

impl<'a> IamCtHttpResServ {
    pub async fn add_http_res(add_req: &mut IamCtHttpResAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamHttpResServ::add_item_with_simple_rel(
            &mut IamHttpResAddReq {
                name: add_req.name.clone(),
                code: add_req.code.clone(),
                icon: add_req.icon.clone(),
                disabled: add_req.disabled,
                scope_level: iam_constants::RBUM_SCOPE_LEVEL_TENANT,
                sort: add_req.sort,
                method: add_req.method.clone(),
            },
            &IAMRelKind::IamHttpResTenant.to_string(),
            &IamTenantServ::get_id_by_cxt(cxt)?,
            funs,
            cxt,
        )
        .await
    }

    pub async fn modify_http_res(id: &str, modify_req: &mut IamCtHttpResModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamHttpResServ::modify_item(
            id,
            &mut IamHttpResModifyReq {
                name: modify_req.name.clone(),
                code: modify_req.code.clone(),
                icon: modify_req.icon.clone(),
                disabled: modify_req.disabled,
                scope_level: None,
                sort: modify_req.sort,
                method: modify_req.method.clone(),
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn get_http_res(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<IamHttpResDetailResp> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamHttpResServ::get_item(id, &IamHttpResFilterReq::default(), funs, cxt).await
    }

    pub async fn paginate_http_res(
        q_name: Option<String>,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<IamHttpResSummaryResp>> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamHttpResServ::paginate_items(
            &IamHttpResFilterReq {
                basic: RbumBasicFilterReq {
                    name: q_name,
                    own_paths: Some(IamTenantServ::get_id_by_cxt(cxt)?),
                    ..Default::default()
                },
                ..Default::default()
            },
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            cxt,
        )
        .await
    }

    pub async fn paginate_rel_roles(
        iam_http_res_id: &str,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAggResp>> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamRelServ::paginate_to_rels(
            IAMRelKind::IamRoleHttpRes,
            iam_http_res_id,
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            funs,
            cxt,
        )
        .await
    }

    pub async fn delete_http_res(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamHttpResServ::delete_item(id, funs, cxt).await
    }
}