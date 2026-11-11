/// What a page says about itself in its Open Graph tags, or failing that its `<title>`.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Meta {
    /// `og:title`, `twitter:title` or `<title>`.
    pub title: Option<String>,
    /// `og:description`, `twitter:description` or `description`.
    pub description: Option<String>,
    /// `og:site_name`.
    pub site: Option<String>,
}

const LIMIT: usize = 300;

fn decode(raw: &str) -> String {
    let text = raw
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&");
    let clean: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    clean.chars().filter(|c| !c.is_control()).take(LIMIT).collect()
}

fn attribute(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let mut from = 0;
    while let Some(at) = lower[from..].find(name).map(|at| at + from) {
        from = at + name.len();
        let before = lower[..at].chars().last().is_some_and(char::is_whitespace);
        let rest = lower[from..].trim_start();
        if !before || !rest.starts_with('=') {
            continue;
        }
        let value = tag[tag.len() - rest.len() + 1..].trim_start();
        let quote = value.chars().next().filter(|c| *c == '"' || *c == '\'')?;
        return value[1..].find(quote).map(|end| value[1..1 + end].to_owned());
    }
    None
}

fn tags(html: &str) -> impl Iterator<Item = &str> {
    html.split('<').filter(|chunk| chunk.len() > 5 && chunk[..5].eq_ignore_ascii_case("meta ")).map(|chunk| {
        let end = chunk.find('>').unwrap_or(chunk.len());
        &chunk[..end]
    })
}

fn title_tag(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let start = lower.find("<title")?;
    let open = lower[start..].find('>')? + start + 1;
    let close = lower[open..].find("</title")? + open;
    Some(html[open..close].to_owned())
}

/// Reads the tags of `html`. Values are entity-decoded, whitespace-collapsed, stripped of
/// control characters and cut at 300 characters; empty ones are dropped.
pub fn parse(html: &str) -> Meta {
    let mut found: Vec<(String, String)> = Vec::new();
    for tag in tags(html) {
        let key = attribute(tag, "property").or_else(|| attribute(tag, "name"));
        if let (Some(key), Some(content)) = (key, attribute(tag, "content")) {
            found.push((key.to_ascii_lowercase(), decode(&content)));
        }
    }
    let pick = |keys: &[&str]| {
        keys.iter().find_map(|key| found.iter().find(|(k, v)| k == key && !v.is_empty()).map(|(_, v)| v.clone()))
    };
    let title =
        pick(&["og:title", "twitter:title"]).or_else(|| title_tag(html).map(|t| decode(&t)).filter(|t| !t.is_empty()));
    Meta {
        title,
        description: pick(&["og:description", "twitter:description", "description"]),
        site: pick(&["og:site_name"]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_graph_wins_over_the_title_and_entities_are_decoded() {
        let html = r#"<html><head><title>Fallback</title>
            <meta property="og:title" content="Caf&eacute; &amp; Co">
            <META name='description' content='  A  "quoted"
              line '><meta property="og:site_name" content="Exemple"/></head></html>"#;
        let meta = parse(html);
        assert_eq!(meta.title.as_deref(), Some("Caf&eacute; & Co"));
        assert_eq!(meta.description.as_deref(), Some("A \"quoted\" line"));
        assert_eq!(meta.site.as_deref(), Some("Exemple"));
        assert_eq!(parse("<title> Only\n the title </title>").title.as_deref(), Some("Only the title"));
        assert_eq!(parse("<p>nothing</p>"), Meta::default());
    }
}
