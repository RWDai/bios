use tardis::basic::result::TardisResult;
use tardis::tokio;

mod test_basic;
mod test_rbum_cert;
mod test_rbum_domain;
mod test_rbum_item;
mod test_rbum_kind;
mod test_rbum_rel;

#[tokio::test]
async fn test_rbum() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();
    let _x = test_basic::init(&docker).await?;
    test_rbum_domain::test().await?;
    test_rbum_kind::test().await?;
    test_rbum_item::test().await?;
    test_rbum_cert::test().await?;
    test_rbum_rel::test().await?;
    Ok(())
}
