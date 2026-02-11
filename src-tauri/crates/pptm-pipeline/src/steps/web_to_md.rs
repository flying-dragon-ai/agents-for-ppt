use anyhow::{Context, Result};
use std::path::Path;

/// 网页转 Markdown
///
/// 使用 reqwest + scraper 获取网页内容并转换为 Markdown 格式
pub async fn web_to_md(url: &str, output_path: &Path) -> Result<()> {
    // 获取网页内容
    let html = fetch_html(url).await?;

    // 解析 HTML
    let markdown = parse_html_to_markdown(&html, url)?;

    // 写入文件
    std::fs::write(output_path, markdown).context("写入 Markdown 文件失败")?;

    Ok(())
}

/// 获取网页 HTML
async fn fetch_html(url: &str) -> Result<String> {
    // TODO: 使用 reqwest 获取网页内容
    // 示例代码（需要添加依赖）:
    // let client = reqwest::Client::builder()
    //     .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
    //     .build()?;
    // let response = client.get(url).send().await?;
    // let html = response.text().await?;
    // Ok(html)

    // 占位实现
    Ok(format!(
        "<html><body><h1>网页内容</h1><p>URL: {}</p></body></html>",
        url
    ))
}

/// 解析 HTML 为 Markdown
fn parse_html_to_markdown(_html: &str, url: &str) -> Result<String> {
    // TODO: 使用 scraper 解析 HTML
    // 示例代码（需要添加依赖）:
    // use scraper::{Html, Selector};
    // let document = Html::parse_document(html);
    //
    // // 提取标题
    // let title_selector = Selector::parse("h1, h2, h3").unwrap();
    // let titles: Vec<_> = document.select(&title_selector).collect();
    //
    // // 提取段落
    // let p_selector = Selector::parse("p").unwrap();
    // let paragraphs: Vec<_> = document.select(&p_selector).collect();
    //
    // // 转换为 Markdown
    // let mut markdown = String::new();
    // for title in titles {
    //     markdown.push_str(&format!("# {}\n\n", title.text().collect::<String>()));
    // }
    // for p in paragraphs {
    //     markdown.push_str(&format!("{}\n\n", p.text().collect::<String>()));
    // }

    // 占位实现
    let markdown = format!(
        "# 网页转换结果\n\n> 源 URL: {}\n\n## 内容\n\n待实现：使用 scraper 解析 HTML\n",
        url
    );

    Ok(markdown)
}

/// 提取主要内容（过滤广告和无关内容）
#[allow(dead_code)]
fn extract_main_content(html: &str) -> Result<String> {
    // TODO: 实现内容提取算法
    // 可以使用以下策略：
    // 1. 查找 <article> 标签
    // 2. 查找 class 包含 "content", "article", "post" 的元素
    // 3. 移除 class 包含 "ad", "sidebar", "footer" 的元素
    // 4. 使用启发式规则（文本密度、链接密度等）

    Ok(html.to_string())
}

/// 清理 HTML（移除脚本、样式等）
#[allow(dead_code)]
fn clean_html(html: &str) -> String {
    // TODO: 移除 <script>, <style>, <iframe> 等标签
    // 可以使用 ammonia crate 进行 HTML 清理

    html.to_string()
}

/// 转换为 Markdown
#[allow(dead_code)]
fn html_to_markdown(_html: &str) -> String {
    // TODO: 实现 HTML 到 Markdown 的转换
    // 支持的元素：
    // - 标题: <h1> -> # , <h2> -> ## , etc.
    // - 段落: <p> -> 文本 + 换行
    // - 列表: <ul>, <ol>, <li>
    // - 链接: <a href="...">text</a> -> [text](url)
    // - 图片: <img src="..." alt="..."> -> ![alt](url)
    // - 代码: <code>, <pre>
    // - 表格: <table>, <tr>, <td>

    String::new()
}

/// 使用 Node.js sidecar 处理复杂网页
pub async fn web_to_md_with_sidecar(url: &str, output_path: &Path) -> Result<()> {
    // TODO: 调用 Node.js sidecar（使用 puppeteer）
    // 用于处理需要 JavaScript 渲染的网页
    //
    // 示例流程：
    // 1. 启动 sidecar 进程
    // 2. 通过 IPC 发送 URL
    // 3. sidecar 使用 puppeteer 渲染页面
    // 4. 返回渲染后的 HTML
    // 5. 解析为 Markdown

    // 占位实现：回退到普通方法
    web_to_md(url, output_path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_html() {
        let html = r#"
            <html>
                <head><script>alert('test');</script></head>
                <body>
                    <h1>Title</h1>
                    <p>Content</p>
                    <script>console.log('test');</script>
                </body>
            </html>
        "#;

        let cleaned = clean_html(html);
        // TODO: 验证脚本已被移除
    }
}
