use async_trait::async_trait;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use std::collections::HashMap;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::sea_query::{Expr, SelectStatement};
use tardis::db::sea_orm::*;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq};
use bios_basic::rbum::helper::rbum_scope_helper;

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::domain::iam_tenant;
use crate::basic::dto::iam_account_dto::IamAccountAggAddReq;
use crate::basic::dto::iam_cert_conf_dto::{IamCertConfLdapResp, IamCertConfMailVCodeAddOrModifyReq, IamCertConfPhoneVCodeAddOrModifyReq};
use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use crate::basic::dto::iam_platform_dto::{IamPlatformAggDetailResp, IamPlatformAggModifyReq};
use crate::basic::dto::iam_tenant_dto::{
    IamTenantAddReq, IamTenantAggAddReq, IamTenantAggDetailResp, IamTenantAggModifyReq, IamTenantDetailResp, IamTenantModifyReq, IamTenantSummaryResp,
};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_ldap_serv::IamCertLdapServ;
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
#[cfg(feature = "spi_kv")]
use crate::basic::serv::spi_client::spi_kv_client::SpiKvClient;
use crate::iam_config::{IamBasicConfigApi, IamBasicInfoManager, IamConfig};
use crate::iam_constants;
use crate::iam_constants::{RBUM_ITEM_ID_TENANT_LEN, RBUM_SCOPE_LEVEL_TENANT};
use crate::iam_enumeration::{IamCertExtKind, IamCertKernelKind, IamCertOAuth2Supplier, IamSetKind};

use super::iam_cert_oauth2_serv::IamCertOAuth2Serv;

pub struct IamPlatformServ;

impl IamPlatformServ {
    pub async fn modify_platform_agg(modify_req: &IamPlatformAggModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if modify_req.cert_conf_by_user_pwd.is_none() && modify_req.cert_conf_by_phone_vcode.is_none() && modify_req.cert_conf_by_mail_vcode.is_none() {
            return Ok(());
        }

        // Init cert conf
        let cert_confs = IamCertServ::find_cert_conf(true, Some("".to_string()), None, None, funs, ctx).await?;

        if let Some(cert_conf_by_user_pwd) = &modify_req.cert_conf_by_user_pwd {
            let cert_conf_by_user_pwd_id = cert_confs.iter().find(|r| r.kind == IamCertKernelKind::UserPwd.to_string()).map(|r| r.id.clone()).unwrap();
            IamCertUserPwdServ::modify_cert_conf(&cert_conf_by_user_pwd_id, cert_conf_by_user_pwd, funs, ctx).await?;
        }
        if let Some(cert_conf_by_phone_vcode) = modify_req.cert_conf_by_phone_vcode {
            if let Some(cert_conf_by_phone_vcode_id) = cert_confs.iter().find(|r| r.kind == IamCertKernelKind::PhoneVCode.to_string()).map(|r| r.id.clone()) {
                if !cert_conf_by_phone_vcode {
                    IamCertServ::disable_cert_conf(&cert_conf_by_phone_vcode_id, funs, ctx).await?;
                }
            } else if cert_conf_by_phone_vcode {
                IamCertPhoneVCodeServ::add_or_enable_cert_conf(&IamCertConfPhoneVCodeAddOrModifyReq { ak_note: None, ak_rule: None }, Some("".to_string()), funs, ctx).await?;
            }
        }

        if let Some(cert_conf_by_mail_vcode) = modify_req.cert_conf_by_mail_vcode {
            if let Some(cert_conf_by_mail_vcode_id) = cert_confs.iter().find(|r| r.kind == IamCertKernelKind::MailVCode.to_string()).map(|r| r.id.clone()) {
                if !cert_conf_by_mail_vcode {
                    IamCertServ::disable_cert_conf(&cert_conf_by_mail_vcode_id, funs, ctx).await?;
                }
            } else if cert_conf_by_mail_vcode {
                IamCertMailVCodeServ::add_or_enable_cert_conf(&IamCertConfMailVCodeAddOrModifyReq { ak_note: None, ak_rule: None }, Some("".to_string()), funs, ctx).await?;
            }
        }
        Ok(())
    }

    pub async fn get_platform_agg(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamPlatformAggDetailResp> {
        let cert_confs = IamCertServ::find_cert_conf(true, Some("".to_string()), None, None, funs, ctx).await?;
        let cert_conf_by_user_pwd = cert_confs.iter().find(|r| r.kind == IamCertKernelKind::UserPwd.to_string()).unwrap();

        let platform = IamPlatformAggDetailResp {
            cert_conf_by_user_pwd: TardisFuns::json.str_to_obj(&cert_conf_by_user_pwd.ext)?,
            cert_conf_by_phone_vcode: cert_confs.iter().any(|r| r.kind == IamCertKernelKind::PhoneVCode.to_string()),
            cert_conf_by_mail_vcode: cert_confs.iter().any(|r| r.kind == IamCertKernelKind::MailVCode.to_string()),
        };

        Ok(platform)
    }
}