use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::data::{Machine, Part, PartGroup, Program, Tool, is_part_dir};
use anyhow::{Result, anyhow};

pub mod gcode;
pub mod h;
pub mod maz;
pub mod mpf;
pub mod pbg;

fn has_part_children(path: &Path) -> bool {
    path.read_dir()
        .map(|entries| {
            entries.flatten().any(|e| {
                e.file_type().map(|t| t.is_dir()).unwrap_or(false)
                    && is_part_dir(&e.file_name().to_string_lossy())
            })
        })
        .unwrap_or(false)
}

fn parse_tools_in_file(path: &Path) -> Vec<Tool> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "pbg" => pbg::parse(path),
        "maz" => maz::parse(path),
        "h" => h::parse(path),
        "mpf" => mpf::parse(path),
        _ => gcode::parse(path),
    }
}

fn should_skip(path: &Path) -> bool {
    static SKIP: OnceLock<HashSet<&'static str>> = OnceLock::new();
    let skip = SKIP.get_or_init(|| {
        [
            "png", "jpg", "jpeg", "gif", "bmp", "ico", "tiff", "tif", "webp", "svg", "psd", "dwg",
            "dxf", "step", "stp", "iges", "igs", "stl", "mp4", "avi", "mov", "mkv", "wmv", "mp3",
            "wav", "flac", "aac", "zip", "rar", "7z", "tar", "gz", "exe", "dll", "db", "sqlite",
            "mdb", "accdb", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "pdf", "ttf", "otf",
            "iso", "img", "pbn", "ezd", "lnk",
        ]
        .into_iter()
        .collect()
    });

    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| skip.contains(ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn collect_programs(part_path: &Path) -> Vec<Program> {
    WalkDir::new(part_path)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .filter(|e| !should_skip(e.path()))
        .map(|e| {
            let path = e.path().to_path_buf();
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let tools = parse_tools_in_file(&path);
            Program { name, tools }
        })
        .filter(|p| !p.tools.is_empty())
        .collect()
}

fn collect_parts(group_path: &Path) -> Vec<Part> {
    group_path
        .read_dir()
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|e| is_part_dir(&e.file_name().to_string_lossy()))
        .map(|e| {
            let path = e.path();
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let programs = collect_programs(&path);
            Part { name, programs }
        })
        .collect()
}

fn collect_part_groups(path: &Path, groups: &mut Vec<PartGroup>, pb: &ProgressBar) -> Result<()> {
    for entry in path.read_dir()?.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }

        let name = entry_path
            .file_name()
            .ok_or_else(|| anyhow!("Не удалось получить имя папки: {:?}", entry_path))?
            .to_string_lossy()
            .to_string();

        if is_part_dir(&name) {
            continue;
        }

        if has_part_children(&entry_path) {
            pb.set_message(name.clone());
            let parts = collect_parts(&entry_path);
            groups.push(PartGroup::new(name, parts));
            pb.inc(1);
        } else {
            collect_part_groups(&entry_path, groups, pb)?;
        }
    }
    Ok(())
}

fn find_part_groups(machine_path: &Path, pb: &ProgressBar) -> Result<Vec<PartGroup>> {
    let mut groups = Vec::new();
    collect_part_groups(machine_path, &mut groups, pb)?;
    groups.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(groups)
}

pub(crate) fn init_machines(path: &Path) -> Result<Vec<Machine>> {
    let dirs: Vec<PathBuf> = path
        .read_dir()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();

    let mp = MultiProgress::new();

    let style = ProgressStyle::with_template(
        "{prefix:.bold.cyan} {spinner:.green} {pos:>3} групп  {msg:.dim}",
    )
    .unwrap()
    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ");

    let machines = dirs
        .par_iter()
        .map(|path| -> Result<Machine> {
            let name = path
                .file_name()
                .ok_or_else(|| anyhow!("Не удалось получить имя станка: {:?}", path))?
                .to_string_lossy()
                .to_string();

            let pb = mp.add(ProgressBar::new_spinner());
            pb.set_style(style.clone());
            pb.set_prefix(format!("{:<30}", name));
            pb.enable_steady_tick(std::time::Duration::from_millis(80));

            let part_groups = find_part_groups(path, &pb)?;

            pb.finish_with_message(format!(
                "✓  {:>3} групп, {:>4} деталей, {:>5} программ",
                part_groups.len(),
                part_groups.iter().map(|g| g.parts_count()).sum::<usize>(),
                part_groups
                    .iter()
                    .map(|g| g.programs_count())
                    .sum::<usize>(),
            ));

            Ok(Machine { name, part_groups })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(machines)
}
fn corrector_char(value: u8) -> char {
    match value {
        1 => 'A',
        2 => 'B',
        3 => 'C',
        4 => 'D',
        5 => 'E',
        6 => 'F',
        7 => 'G',
        8 => 'H',
        9 => 'J',
        10 => 'K',
        11 => 'L',
        12 => 'M',
        13 => 'N',
        14 => 'P',
        15 => 'Q',
        16 => 'R',
        17 => 'S',
        18 => 'T',
        19 => 'U',
        20 => 'V',
        21 => 'W',
        22 => 'X',
        23 => 'Y',
        24 => 'Z',
        _ => '\0',
    }
}
