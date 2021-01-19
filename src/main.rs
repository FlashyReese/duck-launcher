mod common;

#[tokio::main]
async fn main() {
    /*match common::minecraft::get_version_manifest(true).await {
        Ok(val) => {
            for version in val.versions {
                println!("Starting {}", version.id);

            }
        },
        Err(e) => panic!("{}", e)
    }*/
    match common::minecraft::get_version("1.16.5").await {
        Ok(val) => {
            match val {
                Some(value) => {
                    let value = &value;
                    common::minecraft::download_assets(&value).await;
                    common::minecraft::download_libraries(&value).await;
                    common::minecraft::download_natives(&value).await;
                }
                None => {}
            }
        }
        Err(e) => panic!("{}", e)
    }
}