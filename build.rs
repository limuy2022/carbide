use shadow_rs::ShadowBuilder;

fn main() -> anyhow::Result<()> {
    ShadowBuilder::builder().build()?;
    Ok(())
}
