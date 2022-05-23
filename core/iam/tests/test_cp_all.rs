use std::time::Duration;

use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::tokio::time::sleep;

use bios_basic::rbum::serv::rbum_cert_serv::RbumCertServ;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_iam::basic::dto::iam_account_dto::IamAccountSelfModifyReq;
use bios_iam::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use bios_iam::basic::dto::iam_cert_dto::{IamContextFetchReq, IamMailVCodeCertAddReq, IamUserPwdCertModifyReq};
use bios_iam::basic::dto::iam_filer_dto::IamAccountFilterReq;
use bios_iam::basic::serv::iam_account_serv::IamAccountServ;
use bios_iam::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;
use bios_iam::console_passport::dto::iam_cp_cert_dto::{IamCpMailVCodeLoginReq, IamCpUserPwdLoginReq};
use bios_iam::console_passport::serv::iam_cp_cert_mail_vcode_serv::IamCpCertMailVCodeServ;
use bios_iam::console_passport::serv::iam_cp_cert_user_pwd_serv::IamCpCertUserPwdServ;
use bios_iam::console_system::dto::iam_cs_tenant_dto::IamCsTenantAddReq;
use bios_iam::console_system::serv::iam_cs_tenant_serv::IamCsTenantServ;
use bios_iam::iam_constants;

pub async fn test(sysadmin_info: (&str, &str), system_admin_context: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_cp_all】 : Prepare : IamCsTenantServ::add_tenant");
    let (tenant_id, tenant_admin_pwd) = IamCsTenantServ::add_tenant(
        &mut IamCsTenantAddReq {
            tenant_name: TrimString("测试租户1".to_string()),
            tenant_icon: None,
            tenant_contact_phone: None,
            tenant_note: None,
            admin_username: TrimString("bios".to_string()),
            admin_name: TrimString("测试管理员".to_string()),
            admin_password: None,
            cert_conf_by_user_pwd: IamUserPwdCertConfAddOrModifyReq {
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                repeatable: Some(true),
                expire_sec: None,
            },
            cert_conf_by_phone_vcode: Some(IamPhoneVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None }),

            cert_conf_by_mail_vcode: Some(IamMailVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None }),
            disabled: None,
        },
        &funs,
    )
    .await?;
    sleep(Duration::from_secs(1)).await;

    info!("【test_cp_all】 : Login by Username and Password, Password error");
    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString("123456".to_string()),
            tenant_id: None,
            flag: None
        },
        &funs,
    )
    .await
    .is_err());

    info!("【test_cp_all】 : Login by Username and Password, Tenant error");
    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString(sysadmin_info.1.to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None
        },
        &funs,
    )
    .await
    .is_err());

    info!("【test_cp_all】 : Login by Username and Password, Tenant error");
    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString(tenant_admin_pwd.to_string()),
            tenant_id: None,
            flag: None
        },
        &funs,
    )
    .await
    .is_err());

    info!("【test_cp_all】 : Login by Username and Password, By tenant admin");
    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString(tenant_admin_pwd.to_string()),
            tenant_id: Some(tenant_id.clone()),
            flag: None,
        },
        &funs,
    )
    .await?;
    let tenant_admin_context = IamCertServ::fetch_context(
        &IamContextFetchReq {
            token: account_resp.token.to_string(),
            app_id: None,
        },
        &funs,
    )
    .await?;

    assert_eq!(account_resp.account_name, "测试管理员");
    assert_eq!(account_resp.roles.len(), 1);
    assert!(account_resp.roles.iter().any(|i| i.1 == "tenant_admin"));
    assert!(!account_resp.token.is_empty());

    info!("【test_cp_all】 : Login by Username and Password, By sys admin");
    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString(sysadmin_info.1.to_string()),
            tenant_id: None,
            flag: None,
        },
        &funs,
    )
    .await?;

    assert_eq!(account_resp.account_name, "bios");
    assert_eq!(account_resp.roles.len(), 1);
    assert!(account_resp.roles.iter().any(|i| i.1 == "sys_admin"));
    assert!(!account_resp.token.is_empty());

    let system_admin_context = TardisContext {
        own_paths: system_admin_context.own_paths.to_string(),
        ak: sysadmin_info.0.to_string(),
        owner: system_admin_context.owner.to_string(),
        token: system_admin_context.token.to_string(),
        token_kind: system_admin_context.token_kind.to_string(),
        roles: account_resp.roles.iter().map(|i| i.0.to_string()).collect(),
        // TODO
        groups: vec![],
    };

    info!("【test_cp_all】 : Modify Password, original password error");
    assert!(IamCpCertUserPwdServ::modify_cert_user_pwd(
        &system_admin_context.owner,
        &mut IamUserPwdCertModifyReq {
            original_sk: TrimString("12345".to_string()),
            new_sk: TrimString("123456".to_string()),
        },
        &funs,
        &system_admin_context,
    )
    .await
    .is_err());

    info!("【test_cp_all】 : Modify Password");
    IamCpCertUserPwdServ::modify_cert_user_pwd(
        &system_admin_context.owner,
        &mut IamUserPwdCertModifyReq {
            original_sk: TrimString(sysadmin_info.1.to_string()),
            new_sk: TrimString("123456".to_string()),
        },
        &funs,
        &system_admin_context,
    )
    .await?;

    assert!(IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString(sysadmin_info.1.to_string()),
            tenant_id: None,
            flag: None
        },
        &funs,
    )
    .await
    .is_err());

    let account_resp = IamCpCertUserPwdServ::login_by_user_pwd(
        &IamCpUserPwdLoginReq {
            ak: TrimString("bios".to_string()),
            sk: TrimString("123456".to_string()),
            tenant_id: None,
            flag: None,
        },
        &funs,
    )
    .await?;

    assert_eq!(account_resp.account_name, "bios");
    assert_eq!(account_resp.roles.len(), 1);
    assert!(account_resp.roles.iter().any(|i| i.1 == "sys_admin"));
    assert!(!account_resp.token.is_empty());

    // ------------------ Mail-VCode Cert Test Start ------------------

    info!("【test_cp_all】 : Add Mail-VCode Cert");
    let mail_vcode_cert_id = IamCpCertMailVCodeServ::add_cert_mail_vocde(
        &IamMailVCodeCertAddReq {
            mail: "i@sunisle.org".to_string(),
        },
        &funs,
        &tenant_admin_context,
    )
    .await?;

    let vcode = RbumCertServ::get_and_delete_vcode_in_cache("i@sunisle.org", &tenant_admin_context.own_paths, &funs).await?;
    assert!(vcode.is_some());

    info!("【test_cp_all】 : Resend Activation Mail");
    IamCertMailVCodeServ::resend_activation_mail(&tenant_admin_context.owner, "i@sunisle.org", &funs, &tenant_admin_context).await?;

    let vcode = RbumCertServ::get_vcode_in_cache("i@sunisle.org", &tenant_admin_context.own_paths, &funs).await?;
    assert!(vcode.is_some());

    info!("【test_cp_all】 : Activate Mail");
    IamCertMailVCodeServ::activate_mail("i@sunisle.org", &vcode.unwrap(), &funs, &tenant_admin_context).await?;

    info!("【test_cp_all】 : Send Login Mail");
    IamCertMailVCodeServ::send_login_mail("i@sunisle.org", &tenant_admin_context.own_paths, &funs).await?;
    let vcode = RbumCertServ::get_vcode_in_cache("i@sunisle.org", &tenant_admin_context.own_paths, &funs).await?;
    assert!(vcode.is_some());

    sleep(Duration::from_secs(1)).await;
    info!("【test_cp_all】 : Login by Mail And Vcode");
    IamCpCertMailVCodeServ::login_by_mail_vocde(
        &IamCpMailVCodeLoginReq {
            mail: "i@sunisle.org".to_string(),
            vcode: TrimString(vcode.unwrap()),
            tenant_id: tenant_admin_context.own_paths.clone(),
            flag: None,
        },
        &funs,
    )
    .await?;

    info!("【test_cp_all】 : Delete Mail-VCode Cert");
    IamCertServ::delete_cert(&mail_vcode_cert_id, &funs, &tenant_admin_context).await?;

    // ------------------ Mail-VCode Cert Test End ------------------

    info!("【test_cp_all】 : Modify Current Account");
    IamAccountServ::self_modify_account(
        &mut IamAccountSelfModifyReq {
            name: Some(TrimString("测试系统管理员".to_string())),
            icon: Some("/static/images/avatar.png".to_string()),
            disabled: Some(true),
            exts: Default::default(),
        },
        &funs,
        &system_admin_context,
    )
    .await?;

    info!("【test_cp_all】 : Get Current Account");
    let sysadmin = IamAccountServ::get_item(&system_admin_context.owner, &IamAccountFilterReq::default(), &funs, &system_admin_context).await?;
    assert_eq!(sysadmin.name, "测试系统管理员");
    assert_eq!(sysadmin.icon, "/static/images/avatar.png");
    assert!(sysadmin.disabled);

    info!("【test_cp_all】 : Find Rel Roles By Current Account");
    let sysadmin_roles = IamAccountServ::find_simple_rel_roles(&system_admin_context.owner, false, None, None, &funs, &system_admin_context).await?;
    assert_eq!(sysadmin_roles.len(), 1);
    assert_eq!(sysadmin_roles.get(0).unwrap().rel_name, "sys_admin");

    funs.rollback().await?;

    Ok(())
}
