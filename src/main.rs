use std::{fs, path::Path, time::Instant};

use anyhow::{Result, bail};

use crate::{
    args::Args,
    data::{CsvFullRow, CsvRow, Machine, SummaryMachine},
    parser::init_machines,
};

mod args;
mod data;
mod parser;

fn main() -> Result<()> {
    let args = Args::parse();

    let write_fn: fn(&Path, &[Machine], bool) -> Result<()> = match args
        .output
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .as_deref()
    {
        Some("json") => write_json,
        Some("csv") => write_csv,
        Some(other) => bail!("Вывод в файл с расширением {} не поддерживается", other),
        None => bail!("Не указано расширение выходного файла"),
    };

    let timer = Instant::now();

    let mut machines = init_machines(&args.archive)?;
    machines.sort_by(|a, b| a.name.cmp(&b.name));

    for machine in &machines {
        println!("{}", machine);
    }

    let total_groups: usize = machines.iter().map(|m| m.part_groups.len()).sum();
    let total_parts: usize = machines.iter().map(|m| m.parts_count()).sum();
    let total_programs: usize = machines.iter().map(|m| m.programs_count()).sum();

    println!("Станков:   {}", machines.len());
    println!("Групп:     {}", total_groups);
    println!("Деталей:   {}", total_parts);
    println!("Программ:  {}", total_programs);
    println!("Время:     {:.3?}", timer.elapsed());

    write_fn(&args.output, &machines, args.full)?;

    Ok(())
}

fn write_json(path: &Path, machines: &[Machine], full: bool) -> Result<()> {
    let json = if full {
        serde_json::to_string_pretty(&machines)?
    } else {
        let machines: Vec<SummaryMachine> = machines.iter().map(SummaryMachine::from).collect();
        serde_json::to_string_pretty(&machines)?
    };
    std::fs::write(path, &json)?;
    println!("Записано в {} ({} байт)", path.display(), json.len());
    Ok(())
}

fn write_csv(path: &Path, machines: &[Machine], full: bool) -> Result<()> {
    let mut writer = csv::WriterBuilder::new().delimiter(b';').from_path(path)?;
    if full {
        for machine in machines {
            for group in &machine.part_groups {
                for part in &group.parts {
                    writer.serialize(CsvFullRow {
                        machine: &machine.name,
                        group: &group.name,
                        part: &part.name,
                        avg_programs: group.avg_tools_count,
                    })?;
                }
            }
        }
    } else {
        for machine in machines {
            for group in &machine.part_groups {
                writer.serialize(CsvRow {
                    machine: &machine.name,
                    group: &group.name,
                    avg_programs: group.avg_tools_count,
                })?;
            }
        }
    }
    writer.flush()?;
    let size = fs::metadata(path)?.len();
    println!("Записано в {} ({} байт)", path.display(), size);
    Ok(())
}
