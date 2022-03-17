use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::TardisFuns;

use bios_basic::rbum::dto::filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use bios_basic::rbum::dto::rbum_item_attr_dto::{RbumItemAttrAddReq, RbumItemAttrModifyReq};
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemModifyReq};
use bios_basic::rbum::dto::rbum_kind_attr_dto::RbumKindAttrAddReq;
use bios_basic::rbum::dto::rbum_kind_dto::RbumKindAddReq;
use bios_basic::rbum::enumeration::{RbumDataTypeKind, RbumScopeKind, RbumWidgetKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;
use bios_basic::rbum::serv::rbum_item_serv::{RbumItemAttrServ, RbumItemServ};
use bios_basic::rbum::serv::rbum_kind_serv::{RbumKindAttrServ, RbumKindServ};

pub async fn test() -> TardisResult<()> {
    test_rbum_item().await?;
    test_rbum_item_attr().await?;
    Ok(())
}

async fn test_rbum_item() -> TardisResult<()> {
    let context = bios_basic::rbum::initializer::get_sys_admin_context().await?;
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;

    // Prepare Kind
    let kind_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            uri_scheme: TrimString("reldb".to_string()),
            name: TrimString("关系型数据库".to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_kind: None,
        },
        &tx,
        &context,
    )
    .await?;

    // Prepare Domain
    let domain_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("mysql_dev".to_string()),
            name: TrimString("Mysql测试集群".to_string()),
            note: Some("...".to_string()),
            icon: Some("...".to_string()),
            sort: None,
            scope_kind: None,
        },
        &tx,
        &context,
    )
    .await?;

    // -----------------------------------

    // Test Add
    assert!(RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            code: TrimString("".to_string()),
            uri_path: TrimString("".to_string()),
            name: TrimString("".to_string()),
            icon: None,
            sort: None,
            scope_kind: None,
            disabled: None,
            rel_rbum_kind_id: "".to_string(),
            rel_rbum_domain_id: domain_id.to_string()
        },
        &tx,
        &context,
    )
    .await
    .is_err());

    assert!(RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            code: TrimString("".to_string()),
            uri_path: TrimString("".to_string()),
            name: TrimString("".to_string()),
            icon: None,
            sort: None,
            scope_kind: None,
            disabled: None,
            rel_rbum_kind_id: kind_id.to_string(),
            rel_rbum_domain_id: "".to_string()
        },
        &tx,
        &context,
    )
    .await
    .is_err());

    assert!(RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            code: TrimString("".to_string()),
            uri_path: TrimString("".to_string()),
            name: TrimString("".to_string()),
            icon: None,
            sort: None,
            scope_kind: None,
            disabled: None,
            rel_rbum_kind_id: "123".to_string(),
            rel_rbum_domain_id: "123".to_string()
        },
        &tx,
        &context,
    )
    .await
    .is_err());

    let id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            code: TrimString("inst1".to_string()),
            uri_path: TrimString("inst1".to_string()),
            name: TrimString("实例1".to_string()),
            icon: None,
            sort: None,
            scope_kind: None,
            disabled: None,
            rel_rbum_kind_id: kind_id.to_string(),
            rel_rbum_domain_id: domain_id.to_string(),
        },
        &tx,
        &context,
    )
    .await?;

    // Test Get
    let rbum = RbumItemServ::get_rbum(&id, &RbumBasicFilterReq::default(), &tx, &context).await?;
    assert_eq!(rbum.id, id);
    assert_eq!(rbum.uri_path, "inst1");
    assert_eq!(rbum.name, "实例1");

    // Test Modify
    RbumItemServ::modify_rbum(
        &id,
        &mut RbumItemModifyReq {
            code: None,
            uri_path: None,
            name: Some(TrimString("数据库实例1".to_string())),
            icon: None,
            sort: None,
            scope_kind: None,
            disabled: None,
        },
        &tx,
        &context,
    )
    .await?;

    // Test Find
    let rbums = RbumItemServ::paginate_rbums(
        &RbumBasicFilterReq {
            scope_kind: Some(RbumScopeKind::App),
            name: Some("%据库%".to_string()),
            ..Default::default()
        },
        1,
        10,
        None,
        &tx,
        &context,
    )
    .await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert_eq!(rbums.records.get(0).unwrap().name, "数据库实例1");

    // Test Delete
    RbumItemServ::delete_rbum(&id, &tx, &context).await?;
    assert!(RbumItemServ::get_rbum(&id, &RbumBasicFilterReq::default(), &tx, &context).await.is_err());

    tx.rollback().await?;

    Ok(())
}

async fn test_rbum_item_attr() -> TardisResult<()> {
    let context = bios_basic::rbum::initializer::get_sys_admin_context().await?;
    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;

    // Prepare Kind
    let kind_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            uri_scheme: TrimString("reldb".to_string()),
            name: TrimString("关系型数据库".to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: None,
            scope_kind: None,
        },
        &tx,
        &context,
    )
    .await?;

    // Prepare Kind Attr
    let kind_attr_id = RbumKindAttrServ::add_rbum(
        &mut RbumKindAttrAddReq {
            name: TrimString("db_type".to_string()),
            label: "数据库类型".to_string(),
            data_type_kind: RbumDataTypeKind::String,
            widget_type: RbumWidgetKind::InputTxt,
            note: None,
            sort: None,
            main_column: None,
            position: None,
            capacity: None,
            overload: None,
            default_value: None,
            options: None,
            required: None,
            min_length: None,
            max_length: None,
            action: None,
            scope_kind: None,
            rel_rbum_kind_id: kind_id.to_string(),
        },
        &tx,
        &context,
    )
    .await?;

    // Prepare Domain
    let domain_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            uri_authority: TrimString("mysql_dev".to_string()),
            name: TrimString("Mysql测试集群".to_string()),
            note: Some("...".to_string()),
            icon: Some("...".to_string()),
            sort: None,
            scope_kind: None,
        },
        &tx,
        &context,
    )
    .await?;

    let item_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            code: TrimString("inst1".to_string()),
            uri_path: TrimString("inst1".to_string()),
            name: TrimString("实例1".to_string()),
            icon: None,
            sort: None,
            scope_kind: None,
            disabled: None,
            rel_rbum_kind_id: kind_id.to_string(),
            rel_rbum_domain_id: domain_id.to_string(),
        },
        &tx,
        &context,
    )
    .await?;

    // -----------------------------------
    // Test Add
    assert!(RbumItemAttrServ::add_rbum(
        &mut RbumItemAttrAddReq {
            value: "数据1".to_string(),
            rel_rbum_item_id: "".to_string(),
            rel_rbum_kind_attr_id: "".to_string()
        },
        &tx,
        &context,
    )
    .await
    .is_err());

    assert!(RbumItemAttrServ::add_rbum(
        &mut RbumItemAttrAddReq {
            value: "数据1".to_string(),
            rel_rbum_item_id: item_id.to_string(),
            rel_rbum_kind_attr_id: "".to_string()
        },
        &tx,
        &context,
    )
    .await
    .is_err());

    assert!(RbumItemAttrServ::add_rbum(
        &mut RbumItemAttrAddReq {
            value: "数据1".to_string(),
            rel_rbum_item_id: "".to_string(),
            rel_rbum_kind_attr_id: kind_attr_id.to_string()
        },
        &tx,
        &context,
    )
    .await
    .is_err());

    let item_attr_id = RbumItemAttrServ::add_rbum(
        &mut RbumItemAttrAddReq {
            value: "数据1".to_string(),
            rel_rbum_item_id: item_id.to_string(),
            rel_rbum_kind_attr_id: kind_attr_id.to_string(),
        },
        &tx,
        &context,
    )
    .await?;

    // Test Get
    let rbum = RbumItemAttrServ::get_rbum(&item_attr_id, &RbumBasicFilterReq::default(), &tx, &context).await?;
    assert_eq!(rbum.id, item_attr_id);
    assert_eq!(rbum.value, "数据1");
    assert_eq!(rbum.rel_rbum_item_id, item_id.to_string());
    assert_eq!(rbum.rel_rbum_item_name, "实例1".to_string());
    assert_eq!(rbum.rel_rbum_kind_attr_id, kind_attr_id.to_string());
    assert_eq!(rbum.rel_rbum_kind_attr_name, "db_type".to_string());

    // Test Modify
    assert!(RbumItemAttrServ::modify_rbum("111", &mut RbumItemAttrModifyReq { value: "数据2".to_string() }, &tx, &context).await.is_err());

    RbumItemAttrServ::modify_rbum(&item_attr_id, &mut RbumItemAttrModifyReq { value: "数据3".to_string() }, &tx, &context).await?;

    // Test Find
    let rbums = RbumItemAttrServ::paginate_rbums(&RbumBasicFilterReq::default(), 1, 10, None, &tx, &context).await?;
    assert_eq!(rbums.page_number, 1);
    assert_eq!(rbums.page_size, 10);
    assert_eq!(rbums.total_size, 1);
    assert_eq!(rbums.records.get(0).unwrap().value, "数据3");

    // Test Delete
    RbumItemAttrServ::delete_rbum(&item_attr_id, &tx, &context).await?;
    assert!(RbumItemAttrServ::get_rbum(&item_attr_id, &RbumBasicFilterReq::default(), &tx, &context).await.is_err());

    tx.rollback().await?;

    Ok(())
}
