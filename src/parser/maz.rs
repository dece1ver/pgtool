use std::{collections::HashSet, path::Path};

use crate::{data::Tool, parser::corrector_char};

const SECTION_SIZE: usize = 0x64;
const TYPE_TOOL: u16 = 0xB0;
const TYPE_PROBE: u16 = 0xC0;

pub(crate) fn parse(path: &Path) -> Vec<Tool> {
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(_) => return vec![],
    };

    let mut seen: HashSet<(u32, u8)> = HashSet::new();
    let mut offset = 0;

    while offset + SECTION_SIZE <= data.len() {
        let sec_type = u16::from_le_bytes([data[offset], data[offset + 1]]);

        match sec_type {
            TYPE_TOOL => {
                let drill_type = data[offset + 0x09];
                let corrector = data[offset + 0x0B];
                let diameter: u32 = if drill_type == 4 {
                    u16::from_le_bytes([data[offset + 0x26], data[offset + 0x27]]) as u32 * 1000
                } else {
                    u32::from_le_bytes([
                        data[offset + 0x24],
                        data[offset + 0x25],
                        data[offset + 0x26],
                        data[offset + 0x27],
                    ])
                };
                seen.insert((diameter, corrector));
            }
            TYPE_PROBE => {
                let pocket = u16::from_le_bytes([data[offset + 0x08], data[offset + 0x09]]);
                let corrector = data[offset + 0x0B];
                // маркируем датчики смещением чтобы не пересекались с диаметрами
                seen.insert((0xFFFF0000 | pocket as u32, corrector));
            }
            _ => {}
        }

        offset += SECTION_SIZE;
    }

    let mut result: Vec<Tool> = seen
        .into_iter()
        .map(|(number, corr)| Tool::mazatrol(number, corrector_char(corr)))
        .collect();
    result.sort_by_key(|t| t.number);
    result
}
