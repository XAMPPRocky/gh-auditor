use async_std::task;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    task::block_on(async {
        let mut res = surf::get("https://httpbin.org/get").await?;
        dbg!(res.body_string().await?);
        Ok(())
    })
}
