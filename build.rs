#[cfg(windows)]
#[path = "src/view_assets_catalog.rs"]
pub(crate) mod resource_catalog;

#[cfg(windows)]
fn compile_resource() {
    use resource_catalog as catalog;
    use resw::*;

    Build::with_two_languages(lang::LANG_CHS)
        .resource(
            catalog::IDI_CHARLESMINE,
            resource::Icon::from_file("./favicon.ico")
        )
        .compile()
        .expect("Failed to compile resource");
}

fn main() {
    compile_resource();
}