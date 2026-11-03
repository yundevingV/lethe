use crate::bytes::parse_bytes;

pub struct DockerUsage {
    pub images: Option<u64>,
    pub build: Option<u64>,
}

pub fn parse_docker_df(text: &str) -> DockerUsage {
    let mut images = None;
    let mut build = None;
    for line in text.lines() {
        if let Some((kind, bytes)) = json_row(line) {
            assign(&kind, bytes, &mut images, &mut build);
            continue;
        }
        assign(table_kind(line), table_bytes(line), &mut images, &mut build);
    }
    DockerUsage { images, build }
}

fn json_row(line: &str) -> Option<(String, Option<u64>)> {
    let value: serde_json::Value = serde_json::from_str(line).ok()?;
    let kind = value.get("Type")?.as_str()?.to_string();
    let reclaim = value.get("Reclaimable").and_then(|v| v.as_str()).unwrap_or("");
    Some((kind, parse_bytes(reclaim)))
}

fn table_kind(line: &str) -> &str {
    if line.contains("Build Cache") {
        "Build Cache"
    } else if line.starts_with("Images") {
        "Images"
    } else {
        ""
    }
}

fn table_bytes(line: &str) -> Option<u64> {
    line.split([' ', '\t']).rev().find_map(parse_bytes)
}

fn assign(kind: &str, bytes: Option<u64>, images: &mut Option<u64>, build: &mut Option<u64>) {
    if kind == "Images" {
        *images = bytes;
    }
    if kind == "Build Cache" {
        *build = bytes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_rows() {
        let text = "{\"Type\":\"Images\",\"Reclaimable\":\"1.5GB (71%)\"}\n{\"Type\":\"Build Cache\",\"Reclaimable\":\"800MB\"}\n";
        let usage = parse_docker_df(text);
        assert_eq!(usage.images, Some(1_500_000_000));
        assert_eq!(usage.build, Some(800_000_000));
    }
}
