use std::error::Error;

use femark::{process_markdown_to_html, HTMLOutput};

/// parse a markdown source into its optional frontmatter and the HTML string.
pub fn parse(source: &str) -> Result<(Option<frontmatter::Yaml>, String), Box<dyn Error>> {
    let (fm, content) = extract_frontmatter(source)?;
    let html = md_to_html(content);
    Ok((fm, html))
}

fn extract_frontmatter(input: &str) -> Result<(Option<frontmatter::Yaml>, &str), Box<dyn Error>> {
    let (fm, content) = frontmatter::parse_and_find_content(input)?;
    Ok((fm, content))
}

fn md_to_html(s: &str) -> String {
    let parser = pulldown_cmark::Parser::new(s);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    // let Ok(HTMLOutput { content, .. }) = process_markdown_to_html(s) else {
    //     panic!("Error parsing markdown");
    // };
    html_output
}
