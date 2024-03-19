use bios_basic::rbum::{
    dto::{
        rbum_filer_dto::{RbumBasicFilterReq, RbumCertFilterReq, RbumRelFilterReq},
        rbum_rel_agg_dto::{RbumRelAggAddReq, RbumRelEnvAggAddReq},
        rbum_rel_dto::RbumRelAddReq,
    },
    rbum_enumeration::{RbumRelEnvKind, RbumRelFromKind},
    serv::{rbum_cert_serv::RbumCertServ, rbum_crud_serv::RbumCrudOperation, rbum_item_serv::RbumItemCrudOperation, rbum_rel_serv::RbumRelServ},
};
use bios_sdk_invoke::clients::spi_kv_client::SpiKvClient;
use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult}, TardisFuns, TardisFunsInst
};
use tardis::{
    chrono::{self, Utc},
    futures::future::join_all,
};

use crate::{
    basic::dto::{
        iam_cert_conf_dto::IamCertConfAkSkAddOrModifyReq, iam_cert_dto::IamCertAkSkAddReq, iam_filer_dto::IamResFilterReq, iam_open_dto::{IamOpenAddProductReq, IamOpenAkSkAddReq, IamOpenAkSkResp, IamOpenBindAkProductReq}, iam_res_dto::IamResAddReq
    },
    iam_config::IamConfig,
    iam_enumeration::{IamCertKernelKind, IamRelKind, IamResKind},
};

use super::{iam_cert_aksk_serv::IamCertAkSkServ, iam_cert_serv::IamCertServ, iam_key_cache_serv::IamIdentCacheServ, iam_rel_serv::IamRelServ, iam_res_serv::IamResServ, iam_tenant_serv::IamTenantServ};

pub struct IamOpenServ;

impl IamOpenServ {
    pub async fn add_product(add_req: &IamOpenAddProductReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let product_id = IamResServ::add_item(
            &mut IamResAddReq {
                code: add_req.code.clone(),
                name: add_req.name.clone(),
                kind: IamResKind::Product,
                scope_level: add_req.scope_level.clone(),
                disabled: add_req.disabled,
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        for spec in &add_req.specifications {
            let spec_id = IamResServ::add_item(
                &mut IamResAddReq {
                    code: spec.code.clone(),
                    name: spec.name.clone(),
                    kind: IamResKind::Spec,
                    scope_level: spec.scope_level.clone(),
                    disabled: spec.disabled,
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            IamRelServ::add_simple_rel(&IamRelKind::IamProductSpec, &product_id, &spec_id, None, None, false, false, funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn bind_cert_product_and_spec(cert_id: &str, bind_req: &IamOpenBindAkProductReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let old_rels = RbumRelServ::find_detail_rbums(
            &RbumRelFilterReq {
                from_rbum_id: Some(cert_id.to_string()),
                from_rbum_kind: Some(RbumRelFromKind::Cert),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        for rel in old_rels {
            RbumRelServ::delete_rbum(&rel.id, funs, ctx).await?;
        }

        let product_id = IamResServ::find_one_detail_item(
            &IamResFilterReq {
                basic: RbumBasicFilterReq {
                    code: Some(format!("{}/*/{}", IamResKind::Product.to_int(), &bind_req.product_code)),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .ok_or_else(|| funs.err().internal_error("iam_open", "bind_cert_product_and_spec", "illegal response", "404-iam-res-not-exist"))?
        .id;
        let spec_id = IamResServ::find_one_detail_item(
            &IamResFilterReq {
                basic: RbumBasicFilterReq {
                    code: Some(format!("{}/*/{}", IamResKind::Spec.to_int(), &bind_req.spec_code)),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .ok_or_else(|| funs.err().internal_error("iam_open", "bind_cert_product_and_spec", "illegal response", "404-iam-res-not-exist"))?
        .id;

        Self::bind_cert_product(cert_id, &product_id, None, funs, ctx).await?;
        Self::bind_cert_spec(
            cert_id,
            &spec_id,
            None,
            bind_req.start_time,
            bind_req.end_time,
            bind_req.api_call_frequency,
            bind_req.api_call_count,
            funs,
            ctx,
        )
        .await?;
        Self::set_rules_cache(
            cert_id,
            bind_req.start_time,
            bind_req.end_time,
            bind_req.api_call_frequency,
            bind_req.api_call_count,
            funs,
            ctx,
        )
        .await?;
        Ok(())
    }

    async fn bind_cert_product(cert_id: &str, product_id: &str, own_paths: Option<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let req = &mut RbumRelAggAddReq {
            rel: RbumRelAddReq {
                tag: IamRelKind::IamCertProduct.to_string(),
                note: None,
                from_rbum_kind: RbumRelFromKind::Cert,
                from_rbum_id: cert_id.to_string(),
                to_rbum_item_id: product_id.to_string(),
                to_own_paths: own_paths.unwrap_or_else(|| ctx.own_paths.clone()),
                to_is_outside: true,
                ext: None,
            },
            attrs: vec![],
            envs: vec![],
        };
        RbumRelServ::add_rel(req, funs, ctx).await?;
        Ok(())
    }

    async fn bind_cert_spec(
        cert_id: &str,
        spec_id: &str,
        own_paths: Option<String>,
        start_time: Option<chrono::DateTime<Utc>>,
        end_time: Option<chrono::DateTime<Utc>>,
        api_call_frequency: Option<u32>,
        api_call_count: Option<u32>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let mut envs = vec![];
        if start_time.is_some() || end_time.is_some() {
            envs.push(RbumRelEnvAggAddReq {
                kind: RbumRelEnvKind::DatetimeRange,
                value1: start_time.unwrap().to_string(),
                value2: Some(end_time.unwrap().to_string()),
            });
        }
        if let Some(frequency) = api_call_frequency {
            envs.push(RbumRelEnvAggAddReq {
                kind: RbumRelEnvKind::CallFrequency,
                value1: frequency.to_string(),
                value2: None,
            });
        }
        if let Some(count) = api_call_count {
            envs.push(RbumRelEnvAggAddReq {
                kind: RbumRelEnvKind::CallCount,
                value1: count.to_string(),
                value2: None,
            });
        }
        let req = &mut RbumRelAggAddReq {
            rel: RbumRelAddReq {
                tag: IamRelKind::IamCertSpec.to_string(),
                note: None,
                from_rbum_kind: RbumRelFromKind::Cert,
                from_rbum_id: cert_id.to_string(),
                to_rbum_item_id: spec_id.to_string(),
                to_own_paths: own_paths.unwrap_or_else(|| ctx.own_paths.clone()),
                to_is_outside: true,
                ext: None,
            },
            attrs: vec![],
            envs,
        };
        RbumRelServ::add_rel(req, funs, ctx).await?;
        Ok(())
    }

    async fn set_rules_cache(
        cert_id: &str,
        start_time: Option<chrono::DateTime<Utc>>,
        end_time: Option<chrono::DateTime<Utc>>,
        api_call_frequency: Option<u32>,
        api_call_count: Option<u32>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let ak = RbumCertServ::find_one_detail_rbum(
            &RbumCertFilterReq {
                id: Some(cert_id.to_string()),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .ok_or_else(|| funs.err().internal_error("iam_open", "set_rules_cache", "illegal response", "401-iam-cert-code-not-exist"))?
        .ak;
        if start_time.is_some() && end_time.is_some() {
            IamIdentCacheServ::add_gateway_rule_info(
                &ak,
                &RbumRelEnvKind::DatetimeRange.to_string(),
                None,
                &format!("{},{}", start_time.unwrap(), end_time.unwrap()),
                funs,
            )
            .await?;
        }
        if let Some(frequency) = api_call_frequency {
            IamIdentCacheServ::add_gateway_rule_info(&ak, &RbumRelEnvKind::CallFrequency.to_string(), None, &frequency.to_string(), funs).await?;
        }
        if let Some(count) = api_call_count {
            IamIdentCacheServ::add_gateway_rule_info(&ak, &RbumRelEnvKind::CallCount.to_string(), None, &count.to_string(), funs).await?;
        }
        let spec_id = IamRelServ::find_from_id_rels(&IamRelKind::IamCertSpec, false, cert_id, None, None, funs, ctx).await?.pop().unwrap_or_default();
        if !spec_id.is_empty() {
            let spec = IamResServ::find_one_detail_item(
                &IamResFilterReq {
                    basic: RbumBasicFilterReq {
                        ids: Some(vec![spec_id]),
                        ..Default::default()
                    },
                    rel: None,
                    rel2: None,
                    kind: None,
                    icon: None,
                    sort: None,
                    method: None,
                },
                funs,
                ctx,
            )
            .await?
            .ok_or_else(|| funs.err().internal_error("iam_open", "set_rules_cache", "illegal response", "404-iam-res-not-exist"))?;
            IamIdentCacheServ::add_gateway_rule_info(&ak, "rewrite", None, &spec.ext, funs).await?;
        }
        Ok(())
    }

    pub async fn general_cert(add_req: IamOpenAkSkAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamOpenAkSkResp> {
        let rel_iam_item_id = IamTenantServ::get_id_by_ctx(ctx, funs)?;
        let cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::AkSk.to_string().as_str(), Some(rel_iam_item_id.clone()), funs).await
        .unwrap_or(
            IamCertAkSkServ::add_cert_conf(&IamCertConfAkSkAddOrModifyReq {
                name: TrimString(format!("AkSk-{}", &rel_iam_item_id)),
                 expire_sec: None,
            }, Some(IamTenantServ::get_id_by_ctx(ctx, funs)?), funs, ctx).await?
        );
        let ak = TardisFuns::crypto.key.generate_ak()?;
        let sk = TardisFuns::crypto.key.generate_sk(&ak)?;

        let cert_id = IamCertAkSkServ::add_cert(&IamCertAkSkAddReq {
            tenant_id:add_req.tenant_id,
            app_id:add_req.app_id,
        }, &ak, &sk, &cert_conf_id, funs, ctx).await?;
        Ok(IamOpenAkSkResp { id: cert_id, ak, sk })
    }

    pub async fn refresh_cert_cumulative_count(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let cert_ids = RbumRelServ::find_rels(
            &RbumRelFilterReq {
                tag: Some(IamRelKind::IamCertSpec.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .map(|rel_agg| rel_agg.rel.from_rbum_id)
        .collect_vec();
        let _ = join_all(
            cert_ids
                .into_iter()
                .map(|cert_id| async move {
                    if let Ok(Some(cert)) = RbumCertServ::find_one_detail_rbum(
                        &RbumCertFilterReq {
                            id: Some(cert_id.to_string()),
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await
                    {
                        let count = IamIdentCacheServ::get_gateway_cumulative_count(&cert.ak, None, funs).await.unwrap_or_default();
                        let _ = SpiKvClient::add_or_modify_key_name(
                            &format!("{}:{}", funs.conf::<IamConfig>().spi.kv_api_call_count_prefix.clone(), cert.id.as_str()),
                            &count.unwrap_or_default(),
                            funs,
                            ctx,
                        )
                        .await;
                    }
                })
                .collect_vec(),
        )
        .await;
        Ok(())
    }
}
