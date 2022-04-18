use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::result::TardisResult;
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::iam_constants;
use crate::basic::dto::iam_app_dto::{IamAppAddReq, IamAppDetailResp, IamAppModifyReq, IamAppSummaryResp};
use crate::basic::dto::iam_filer_dto::IamAppFilterReq;
use crate::iam_enumeration::IAMRelKind;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_tenant::dto::iam_ct_app_dto::{IamCtAppAddReq, IamCtAppModifyReq};

pub struct IamCtAppServ;

impl<'a> IamCtAppServ {
    pub async fn add_app(add_req: &mut IamCtAppAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamAppServ::add_item_with_simple_rel(
            &mut IamAppAddReq {
                name: add_req.name.clone(),
                icon: add_req.icon.clone(),
                sort: None,
                contact_phone: add_req.contact_phone.clone(),
                disabled: add_req.disabled,
                scope_level: iam_constants::RBUM_SCOPE_LEVEL_TENANT,
            },
            &IAMRelKind::IamAppTenant.to_string(),
            &IamTenantServ::get_id_by_cxt(cxt)?,
            funs,
            cxt,
        )
        .await
    }

    pub async fn modify_app(id: &str, modify_req: &mut IamCtAppModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamAppServ::modify_item(
            id,
            &mut IamAppModifyReq {
                name: modify_req.name.clone(),
                icon: modify_req.icon.clone(),
                sort: modify_req.sort,
                contact_phone: modify_req.contact_phone.clone(),
                disabled: modify_req.disabled,
                scope_level: None,
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn get_app(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<IamAppDetailResp> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamAppServ::get_item(id, &IamAppFilterReq::default(), funs, cxt).await
    }

    pub async fn paginate_apps(
        q_name: Option<String>,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<IamAppSummaryResp>> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamAppServ::paginate_items(
            &IamAppFilterReq {
                basic: RbumBasicFilterReq {
                    name: q_name,
                    own_paths: Some(cxt.own_paths.clone()),
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

    pub async fn delete_app(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        IamRoleServ::need_tenant_admin(funs, cxt).await?;
        IamAppServ::delete_item(id, funs, cxt).await
    }
}