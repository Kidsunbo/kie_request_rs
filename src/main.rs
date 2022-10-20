use tokio::join;

mod error;
mod http;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let g1 = tokio::spawn(async {
        let url = "http://avalon.bytedance.net/web/measure/project_score:80";
        let mut request = http::Request::new(&url, http::RequestMethod::GET);
        let content = request
            .add_header("Connction", "Close")
            .add_header("Host", "fxg.jinritemai.com")
            .add_header("Accept-Encoding", "gzip, deflate, br")
            .add_header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.0.0.0 Safari/537.36, val")
            .add_header("sec-ch-ua", r#"Chromium";v="106", "Google Chrome";v="106", "Not;A=Brand";v="99""#)
            .add_header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9")
            .add_header("Sec-Fetch-Dest", "document")
            .get_content(url)
            .await;
        match content {
            Ok(content) => println!("{url}:\n{}", content.to_string()),
            Err(err) => println!("{}", err),
        }
    });

    let _ = join!(g1);
    Ok(())
}
