use std::{collections::HashSet, path::Path};

use crate::{data::Tool, parser::corrector_char};

const SECTIONS_START: usize = 0x160;
const SECTION_SIZE: usize = 0x64;
const TYPE_TOOL: u16 = 0xB4;
const OFFSET_CORRECTOR: usize = 0x0B;
const OFFSET_POCKET: usize = 0x5A;

pub(crate) fn parse(path: &Path) -> Vec<Tool> {
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(_) => return vec![],
    };

    let mut seen: HashSet<(u16, u8)> = HashSet::new();
    let mut offset = SECTIONS_START;

    while offset + SECTION_SIZE <= data.len() {
        let sec_type = u16::from_le_bytes([data[offset], data[offset + 1]]);
        if sec_type == TYPE_TOOL {
            let pocket = u16::from_le_bytes([
                data[offset + OFFSET_POCKET],
                data[offset + OFFSET_POCKET + 1],
            ]);
            let corrector = data[offset + OFFSET_CORRECTOR];
            seen.insert((pocket, corrector));
        }
        offset += SECTION_SIZE;
    }

    let mut result: Vec<Tool> = seen
        .into_iter()
        .map(|(pocket, corr)| Tool::mazatrol(pocket as u32, corrector_char(corr)))
        .collect();
    result.sort_by_key(|t| t.number);
    result
}
