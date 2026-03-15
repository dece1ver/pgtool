use std::time::Instant;

use anyhow::Result;

use crate::{args::Args, data::SummaryMachine, parser::init_machines};

mod args;
mod data;
mod parser;
fn main() -> Result<()> {
    let args = Args::parse();

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

    let json = if !args.full {
        let machines: Vec<SummaryMachine> = machines.iter().map(SummaryMachine::from).collect();
        serde_json::to_string_pretty(&machines)?
    } else {
        serde_json::to_string_pretty(&machines)?
    };
    std::fs::write(&args.output, &json)?;
    println!("Записано в {} ({} байт)", args.output.display(), json.len());

    Ok(())
}
