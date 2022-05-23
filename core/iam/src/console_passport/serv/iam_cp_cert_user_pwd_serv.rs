use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::result::TardisResult;

use bios_basic::rbum::helper::rbum_scope_helper::get_max_level_id_by_context;
use bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ;

use crate::basic::dto::iam_account_dto::AccountInfoResp;
use crate::basic::dto::iam_cert_dto::IamUserPwdCertModifyReq;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use crate::iam_enumeration::IamCertKind;

pub struct IamCpCertUserPwdServ;

impl<'a> IamCpCertUserPwdServ {
    pub async fn modify_cert_user_pwd(id: &str, modify_req: &IamUserPwdCertModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_code(IamCertKind::UserPwd.to_string().as_str(), get_max_level_id_by_context(cxt), funs).await?;
        IamCertUserPwdServ::modify_cert(modify_req, id, &rbum_cert_conf_id, funs, cxt).await
    }

    pub async fn login_by_user_pwd(login_req: &IamCpUserPwdLoginReq, funs: &TardisFunsInst<'a>) -> TardisResult<AccountInfoResp> {
        let tenant_id = if let Some(tenant_id) = &login_req.tenant_id { tenant_id } else { "" };
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_code(&IamCertKind::UserPwd.to_string(), Some(tenant_id.to_string()), funs).await?;
        let (rbum_cert_id, _, rbum_item_id) = RbumCertServ::validate(&login_req.ak.0, &login_req.sk.0, &rbum_cert_conf_id, tenant_id, funs).await?;
        let resp = IamCertServ::package_tardis_context_and_resp(login_req.tenant_id.clone(), &login_req.ak.0, &rbum_item_id, &rbum_cert_id, login_req.flag.clone(), funs).await?;
        Ok(resp)
    }
}
