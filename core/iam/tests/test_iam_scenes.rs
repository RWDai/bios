use std::collections::HashMap;

use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;
use tardis::web::web_resp::{TardisPage, Void};

use bios_basic::rbum::dto::rbum_cert_conf_dto::RbumCertConfDetailResp;
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryResp;
use bios_basic::rbum::dto::rbum_kind_attr_dto::{RbumKindAttrDetailResp, RbumKindAttrModifyReq, RbumKindAttrSummaryResp};
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;
use bios_basic::rbum::dto::rbum_set_dto::RbumSetPathResp;
use bios_basic::rbum::rbum_enumeration::{RbumDataTypeKind, RbumWidgetTypeKind};
use bios_iam::basic::dto::iam_account_dto::{AccountInfoResp, IamAccountAggAddReq, IamAccountAggModifyReq, IamAccountDetailResp, IamAccountSummaryResp};
use bios_iam::basic::dto::iam_attr_dto::IamKindAttrAddReq;
use bios_iam::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamUserPwdCertConfAddOrModifyReq};
use bios_iam::basic::dto::iam_cert_dto::IamUserPwdCertRestReq;
use bios_iam::basic::dto::iam_role_dto::{IamRoleAddReq, IamRoleSummaryResp};
use bios_iam::basic::dto::iam_tenant_dto::{IamTenantBoneResp, IamTenantDetailResp, IamTenantModifyReq, IamTenantSummaryResp};
use bios_iam::console_passport::dto::iam_cp_cert_dto::IamCpUserPwdLoginReq;
use bios_iam::console_system::dto::iam_cs_tenant_dto::IamCsTenantAddReq;
use bios_iam::iam_constants::RBUM_SCOPE_LEVEL_GLOBAL;
use bios_iam::iam_enumeration::IamCertKind;
use bios_iam::iam_test_helper::BIOSWebTestClient;

pub async fn test(client: &mut BIOSWebTestClient, sysadmin_name: &str, sysadmin_password: &str) -> TardisResult<()> {
    login_page(client, sysadmin_name, sysadmin_password, None, true).await?;
    let tenant_id = sys_admin_tenant_mgr_page(client).await?;
    sys_admin_account_mgr_page(client, &tenant_id).await?;
    Ok(())
}

pub async fn login_page(client: &mut BIOSWebTestClient, user_name: &str, password: &str, tenant_id: Option<String>, set_auth: bool) -> TardisResult<AccountInfoResp> {
    info!("【login_page】");
    // Find Tenants
    let _: Vec<IamTenantBoneResp> = client.get("/cp/tenant").await;
    // Login
    let account: AccountInfoResp = client
        .put(
            "/cp/login/userpwd",
            &IamCpUserPwdLoginReq {
                ak: TrimString(user_name.to_string()),
                sk: TrimString(password.to_string()),
                tenant_id,
                flag: None,
            },
        )
        .await;
    assert_eq!(account.account_name, user_name);
    // Find Context
    if set_auth {
        client.set_auth(&account.token, None).await?;
    }
    Ok(account)
}

pub async fn sys_admin_tenant_mgr_page(client: &mut BIOSWebTestClient) -> TardisResult<String> {
    info!("【sys_admin_tenant_mgr_page】");
    // Add Tenant
    let tenant_id: String = client
        .post(
            "/cs/tenant",
            &IamCsTenantAddReq {
                tenant_name: TrimString("测试公司1".to_string()),
                tenant_icon: Some("https://oss.minio.io/xxx.icon".to_string()),
                tenant_contact_phone: None,
                tenant_note: None,
                admin_name: TrimString("测试管理员".to_string()),
                admin_username: TrimString("admin".to_string()),
                admin_password: None,
                cert_conf_by_user_pwd: IamUserPwdCertConfAddOrModifyReq {
                    ak_note: None,
                    ak_rule: None,
                    // 密码长度，密码复杂度等使用前端自定义格式写入到sk_node字段
                    sk_note: None,
                    // 前端生成正则判断写入到sk_rule字段
                    sk_rule: None,
                    repeatable: Some(false),
                    expire_sec: None,
                },
                cert_conf_by_phone_vcode: Some(IamPhoneVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None }),
                cert_conf_by_mail_vcode: None,
                disabled: None,
            },
        )
        .await;

    // Find Tenants
    let tenants: TardisPage<IamTenantSummaryResp> = client.get("/cs/tenant?page_number=1&page_size=10").await;
    assert_eq!(tenants.total_size, 1);

    // Get Tenant by Tenant Id
    let tenant: IamTenantDetailResp = client.get(&format!("/cs/tenant/{}", tenant_id)).await;
    assert_eq!(tenant.name, "测试公司1");
    assert_eq!(tenant.icon, "https://oss.minio.io/xxx.icon");

    // Count Accounts by Tenant Id
    let tenants: u64 = client.get(&format!("/cs/account/total?tenant_id={}", tenant_id)).await;
    assert_eq!(tenants, 1);

    // Find Cert Conf by Tenant Id
    let cert_conf: Vec<RbumCertConfDetailResp> = client.get(&format!("/cs/cert-conf?tenant_id={}", tenant_id)).await;
    let cert_conf_user_pwd = cert_conf.iter().find(|x| x.code == IamCertKind::UserPwd.to_string()).unwrap();
    let cert_conf_phone_vcode = cert_conf.iter().find(|x| x.code == IamCertKind::PhoneVCode.to_string()).unwrap();
    assert_eq!(cert_conf.len(), 2);
    assert!(cert_conf_user_pwd.sk_encrypted);
    assert!(!cert_conf_user_pwd.repeatable);

    // Modify Tenant by Tenant Id
    let _: Void = client
        .put(
            &format!("/cs/tenant/{}", tenant_id),
            &IamTenantModifyReq {
                name: Some(TrimString("测试公司_new".to_string())),
                scope_level: None,
                disabled: None,
                icon: None,
                sort: None,
                contact_phone: None,
                note: None,
            },
        )
        .await;

    // Modify Cert Conf by User Pwd Id
    let _: Void = client
        .put(
            &format!("/cs/cert-conf/{}/user-pwd", cert_conf_user_pwd.id),
            &IamUserPwdCertConfAddOrModifyReq {
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                repeatable: Some(false),
                expire_sec: Some(111),
            },
        )
        .await;

    // Delete Cert Conf by Cert Conf Id
    client.delete(format!("/cs/cert-conf/{}", cert_conf_phone_vcode.id).as_str()).await;
    let cert_conf: Vec<RbumCertConfDetailResp> = client.get(&format!("/cs/cert-conf?tenant_id={}", tenant_id)).await;
    assert_eq!(cert_conf.len(), 1);

    // Add Cert Conf by Tenant Id
    let _: Void = client
        .post(
            &format!("/cs/cert-conf/mail-vcode?tenant_id={}", tenant_id),
            &IamMailVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None },
        )
        .await;
    let cert_conf: Vec<RbumCertConfDetailResp> = client.get(&format!("/cs/cert-conf?tenant_id={}", tenant_id)).await;
    assert_eq!(cert_conf.len(), 2);

    // Add Role
    let role_id: String = client
        .post(
            "/cs/role",
            &IamRoleAddReq {
                name: TrimString("审计管理员".to_string()),
                // 必须设置成全局作用域（1）
                scope_level: Some(RBUM_SCOPE_LEVEL_GLOBAL),
                disabled: None,
                icon: None,
                sort: None,
            },
        )
        .await;

    // Find Roles
    let roles: TardisPage<IamRoleSummaryResp> = client.get("/cs/role?with_sub=true&page_number=1&page_size=10").await;
    let sys_admin_role_id = &roles.records.iter().find(|i| i.name == "sys_admin").unwrap().id;
    assert_eq!(roles.total_size, 4);
    assert!(roles.records.iter().any(|i| i.name == "审计管理员"));

    // Count Accounts By Role Id
    let roles: u64 = client.get(&format!("/cs/role/{}/account/total", sys_admin_role_id)).await;
    assert_eq!(roles, 1);

    // Find Accounts
    let accounts: TardisPage<IamAccountSummaryResp> = client.get("/cs/account?with_sub=true&page_number=1&page_size=10").await;
    assert_eq!(accounts.total_size, 2);

    // Find Accounts By Role Id
    let accounts: TardisPage<IamAccountSummaryResp> = client.get(&format!("/cs/account?role_id={}&with_sub=true&page_number=1&page_size=10", sys_admin_role_id)).await;
    let sys_admin_account_id = &accounts.records.get(0).unwrap().id;
    assert_eq!(accounts.total_size, 1);
    assert_eq!(accounts.records.get(0).unwrap().name, "bios");

    let accounts: TardisPage<IamAccountSummaryResp> = client.get(&format!("/cs/account?role_id={}&with_sub=true&page_number=1&page_size=10", role_id)).await;
    assert_eq!(accounts.total_size, 0);

    // Find Role By Account Id
    let roles: Vec<RbumRelAggResp> = client.get(&format!("/cs/account/{}/roles", sys_admin_account_id)).await;
    assert_eq!(roles.len(), 1);
    assert_eq!(roles.get(0).unwrap().rel.to_rbum_item_name, "sys_admin");

    // Find Set Paths By Account Id
    let roles: Vec<Vec<Vec<RbumSetPathResp>>> = client.get(&format!("/cs/account/{}/set-paths", sys_admin_account_id)).await;
    assert_eq!(roles.len(), 0);

    // Find Certs By Account Id
    let certs: Vec<RbumCertSummaryResp> = client.get(&format!("/cs/cert?account_id={}", sys_admin_account_id)).await;
    assert_eq!(certs.len(), 2);
    assert!(certs.into_iter().any(|i| i.rel_rbum_cert_conf_code == Some("UserPwd".to_string())));

    // Lock/Unlock Account By Account Id
    let _: Void = client
        .put(
            &format!("/cs/account/{}", sys_admin_account_id),
            &IamAccountAggModifyReq {
                name: None,
                scope_level: None,
                disabled: Some(true),
                icon: None,
                exts: Default::default(),
            },
        )
        .await;

    // Rest Password By Account Id
    let _: Void = client
        .put(
            &format!("/cs/cert/user-pwd?account_id={}", sys_admin_account_id),
            &IamUserPwdCertRestReq {
                new_sk: TrimString("123456".to_string()),
            },
        )
        .await;
    login_page(client, "bios", "123456", None, true).await?;

    Ok(tenant_id)
}

pub async fn sys_admin_account_mgr_page(client: &mut BIOSWebTestClient, tenant_id: &str) -> TardisResult<()> {
    info!("【sys_admin_account_mgr_page】");
    // -------------------- Account Attr --------------------

    // Add Account Attr By Tenant Id
    let _: String = client
        .post(
            &format!("/cs/account/attr?tenant_id={}", tenant_id),
            &IamKindAttrAddReq {
                name: TrimString("ext1_idx".to_string()),
                label: "工号".to_string(),
                note: None,
                sort: None,
                main_column: Some(true),
                position: None,
                capacity: None,
                overload: None,
                idx: None,
                data_type: RbumDataTypeKind::String,
                widget_type: RbumWidgetTypeKind::Input,
                default_value: None,
                options: None,
                required: Some(true),
                min_length: None,
                max_length: None,
                action: None,
                ext: None,
                scope_level: None,
            },
        )
        .await;

    let attr_id: String = client
        .post(
            &format!("/cs/account/attr?tenant_id={}", tenant_id),
            &IamKindAttrAddReq {
                name: TrimString("ext9".to_string()),
                label: "岗级".to_string(),
                note: None,
                sort: None,
                main_column: Some(true),
                position: None,
                capacity: None,
                overload: None,
                idx: None,
                data_type: RbumDataTypeKind::String,
                widget_type: RbumWidgetTypeKind::Input,
                default_value: None,
                options: Some(r#"[{"l1":"L1","l2":"L2"}]"#.to_string()),
                required: None,
                min_length: None,
                max_length: None,
                action: None,
                ext: None,
                scope_level: None,
            },
        )
        .await;

    // Find Account Attrs By Tenant Id
    let attrs: Vec<RbumKindAttrSummaryResp> = client.get(&format!("/cs/account/attr?tenant_id={}", tenant_id)).await;
    assert_eq!(attrs.len(), 2);

    // Modify Account Attrs by Attr Id
    let _: Void = client
        .put(
            &format!("/cs/account/attr/{}", attr_id),
            &RbumKindAttrModifyReq {
                name: None,
                label: None,
                note: None,
                sort: None,
                main_column: None,
                position: None,
                capacity: None,
                overload: None,
                idx: None,
                data_type: None,
                widget_type: None,
                default_value: None,
                options: Some(r#"[{"l1":"L1","l2":"L2","l3":"L3"}]"#.to_string()),
                required: None,
                min_length: None,
                max_length: None,
                action: None,
                ext: None,
                scope_level: None,
            },
        )
        .await;

    // Get Account Attrs by Attr Id
    let attr: RbumKindAttrDetailResp = client.get(&format!("/cs/account/attr/{}", attr_id)).await;
    assert_eq!(attr.name, "ext9");
    assert_eq!(attr.label, "岗级");
    assert_eq!(attr.options, r#"[{"l1":"L1","l2":"L2","l3":"L3"}]"#);

    // Delete Account Attr By Attr Id
    client.delete(&format!("/cs/account/attr/{}", attr_id)).await;

    // -------------------- Account --------------------

    // Find Cert Conf by Tenant Id
    let cert_conf: Vec<RbumCertConfDetailResp> = client.get(&format!("/cs/cert-conf?tenant_id={}", tenant_id)).await;
    let cert_conf_user_pwd = cert_conf.iter().find(|x| x.code == IamCertKind::UserPwd.to_string()).unwrap();
    assert_eq!(cert_conf.len(), 2);
    assert!(cert_conf.iter().any(|x| x.code == IamCertKind::MailVCode.to_string()));
    assert!(!cert_conf_user_pwd.repeatable);

    // Find Roles by Tenant Id
    let roles: TardisPage<IamRoleSummaryResp> = client.get(&format!("/cs/role?tenant_id={}&with_sub=true&page_number=1&page_size=10", tenant_id)).await;
    let role_id = &roles.records.iter().find(|i| i.name == "审计管理员").unwrap().id;
    assert_eq!(roles.total_size, 4);
    assert!(roles.records.iter().any(|i| i.name == "sys_admin"));
    assert!(roles.records.iter().any(|i| i.name == "审计管理员"));

    // Add Account
    let account_id: String = client
        .post(
            &format!("/cs/account?tenant_id={}", tenant_id),
            &IamAccountAggAddReq {
                id: None,
                name: TrimString("用户1".to_string()),
                cert_user_name: TrimString("user1".to_string()),
                cert_password: TrimString("123456".to_string()),
                cert_phone: None,
                cert_mail: Some(TrimString("i@sunisle.org".to_string())),
                roles: Some(vec![role_id.to_string()]),
                scope_level: None,
                disabled: None,
                icon: None,
                exts: HashMap::from([("ext1_idx".to_string(), "00001".to_string())]),
            },
        )
        .await;

    // Find Accounts By Tenant Id
    let accounts: TardisPage<IamAccountSummaryResp> = client.get(&format!("/cs/account?tenant_id={}&with_sub=true&page_number=1&page_size=10", tenant_id)).await;
    assert_eq!(accounts.total_size, 2);

    // Get Account By Account Id
    let account: IamAccountDetailResp = client.get(&format!("/cs/account/{}", account_id)).await;
    assert_eq!(account.name, "用户1");
    // Find Account Attr Value By Account Id
    let account_attrs: HashMap<String, String> = client.get(&format!("/cs/account/attr/values?account_id={}&tenant_id={}", account_id, tenant_id)).await;
    assert_eq!(account_attrs.len(), 1);
    assert_eq!(account_attrs.get("ext1_idx"), Some(&"00001".to_string()));

    // Find Rel Roles By Account Id
    let roles: Vec<RbumRelAggResp> = client.get(&format!("/cs/account/{}/roles", account_id)).await;
    assert_eq!(roles.len(), 1);
    assert_eq!(roles.get(0).unwrap().rel.to_rbum_item_name, "审计管理员");

    // Modify Account By Account Id
    let _: Void = client
        .put(
            &format!("/cs/account/{}?tenant_id={}", account_id, tenant_id),
            &IamAccountAggModifyReq {
                name: Some(TrimString("用户2".to_string())),
                scope_level: None,
                disabled: None,
                icon: None,
                exts: HashMap::from([("ext1_idx".to_string(), "".to_string())]),
            },
        )
        .await;

    // Get Account By Account Id
    let account: IamAccountDetailResp = client.get(&format!("/cs/account/{}", account_id)).await;
    assert_eq!(account.name, "用户2");
    // Find Account Attr By Account Id
    let account_attrs: HashMap<String, String> = client.get(&format!("/cs/account/attr/values?account_id={}&tenant_id={}", account_id, tenant_id)).await;
    assert_eq!(account_attrs.len(), 1);
    assert_eq!(account_attrs.get("ext1_idx"), Some(&"".to_string()));

    // Find Certs By Account Id
    let certs: Vec<RbumCertSummaryResp> = client.get(&format!("/cs/cert?account_id={}&tenant_id={}", account_id, tenant_id)).await;
    assert_eq!(certs.len(), 2);
    assert!(certs.into_iter().any(|i| i.rel_rbum_cert_conf_code == Some("UserPwd".to_string())));

    // Delete Account By Account Id
    let _ = client.delete(&format!("/cs/account/{}", account_id)).await;

    // Rest Password By Account Id
    let _: Void = client
        .put(
            &format!("/cs/cert/user-pwd?account_id={}&tenant_id={}", account_id, tenant_id),
            &IamUserPwdCertRestReq {
                new_sk: TrimString("123456".to_string()),
            },
        )
        .await;

    Ok(())
}
