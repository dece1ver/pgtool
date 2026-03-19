use std::{path::Path, sync::OnceLock};

use regex::Regex;

use crate::data::{PathExt, Tool};

pub(crate) fn parse(path: &Path) -> Vec<Tool> {
    let Ok(lines) = path.read_lines() else {
        return vec![];
    };
    lines
        .flatten()
        .filter_map(|line| parse_tool_call(&line))
        .map(Tool::gcode)
        .collect()
}

fn parse_tool_call(line: &str) -> Option<u32> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re =
        RE.get_or_init(|| Regex::new(r"(?i)tool\s+call\s+(\d+)").expect("failed create regex"));
    re.captures(line)?.get(1)?.as_str().parse().ok()
}
