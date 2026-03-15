use std::{
    collections::HashSet,
    io::{BufRead, BufReader},
    path::Path,
    sync::OnceLock,
};

use regex::Regex;

use crate::data::Tool;

pub(crate) fn parse(path: &Path) -> Vec<Tool> {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    let re = REGEX.get_or_init(|| Regex::new(r"T(?P<num>\d+)").unwrap());

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return vec![],
    };

    let mut seen: HashSet<u32> = HashSet::new();
    let reader = BufReader::new(file);

    for line in reader.lines().map_while(Result::ok) {
        if !line.starts_with('T') && !line.contains("M6") {
            continue;
        }
        if let Some(caps) = re.captures(&line)
            && let Some(num) = caps.name("num")
            && let Ok(value) = num.as_str().parse::<u32>()
        {
            seen.insert(value);
        }
    }

    let mut result: Vec<Tool> = seen.into_iter().map(Tool::gcode).collect();
    result.sort_by_key(|t| t.number);
    result
}
