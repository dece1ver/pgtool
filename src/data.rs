use core::fmt;

use serde::{Deserialize, Serialize};

const NUMBER_SIGNS_AREOPAG: &[&str] = &[
    "АР",
    "АРМ",
    "АРКП",
    "АРПГА",
    "АРКО",
    "АРНП",
    "АТГ",
    "НМГ",
    "АМГ",
    "АРН",
    "АРС",
    "АРФС",
    "М8Л",
    "ТОС",
    "AP",
];
const NUMBER_SIGNS_THIRD: &[&str] = &["ТОМ3", "ИН0", "ИНО"];
const NUMBER_SIGNS_GOST: &[&str] = &["10-", "15-", "25-", "32-", "40-", "50-"];

pub(crate) fn is_part_dir(name: &str) -> bool {
    NUMBER_SIGNS_AREOPAG
        .iter()
        .chain(NUMBER_SIGNS_THIRD)
        .chain(NUMBER_SIGNS_GOST)
        .any(|sign| name.contains(sign))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct Tool {
    pub(crate) number: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    corrector: Option<char>,
}

impl Tool {
    pub(crate) fn gcode(number: u32) -> Self {
        Self {
            number,
            corrector: None,
        }
    }

    pub(crate) fn mazatrol(number: u32, corrector: char) -> Self {
        Self {
            number,
            corrector: Some(corrector),
        }
    }
}

impl fmt::Display for Tool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.corrector {
            Some(c) => write!(f, "{}{}", self.number, c),
            None => write!(f, "T{}", self.number),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SummaryMachine {
    pub(crate) name: String,
    pub(crate) part_groups: Vec<SummaryPartGroup>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Machine {
    pub(crate) name: String,
    pub(crate) part_groups: Vec<PartGroup>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PartGroup {
    pub(crate) name: String,
    pub(crate) avg_tools_count: usize,
    pub(crate) parts: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SummaryPartGroup {
    pub(crate) name: String,
    pub(crate) avg_tools_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Part {
    pub(crate) name: String,
    pub(crate) programs: Vec<Program>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Program {
    pub(crate) name: String,
    pub(crate) tools: Vec<Tool>,
}

impl Program {
    pub(crate) fn tools_count(&self) -> usize {
        self.tools.len()
    }
}

impl Part {
    pub(crate) fn programs_count(&self) -> usize {
        self.programs.len()
    }
}

impl PartGroup {
    pub(crate) fn new(name: String, parts: Vec<Part>) -> Self {
        let (tools_sum, program_count) = parts
            .iter()
            .flat_map(|p| p.programs.iter())
            .fold((0usize, 0usize), |(sum, count), prog| {
                (sum + prog.tools_count(), count + 1)
            });
        let avg_tools_count = if program_count == 0 {
            0
        } else {
            tools_sum / program_count
        };
        Self {
            name,
            avg_tools_count,
            parts,
        }
    }

    pub(crate) fn _avg_tools_per_program(&self) -> usize {
        let programs: Vec<&Program> = self.parts.iter().flat_map(|p| p.programs.iter()).collect();
        if programs.is_empty() {
            return 0;
        }
        programs.iter().map(|p| p.tools_count()).sum::<usize>() / programs.len()
    }

    pub(crate) fn parts_count(&self) -> usize {
        self.parts.len()
    }

    pub(crate) fn programs_count(&self) -> usize {
        self.parts.iter().map(|p| p.programs_count()).sum()
    }
}
impl From<&PartGroup> for SummaryPartGroup {
    fn from(value: &PartGroup) -> Self {
        Self {
            name: value.name.clone(),
            avg_tools_count: value.avg_tools_count,
        }
    }
}

impl Machine {
    pub(crate) fn parts_count(&self) -> usize {
        self.part_groups.iter().map(|g| g.parts_count()).sum()
    }

    pub(crate) fn programs_count(&self) -> usize {
        self.part_groups.iter().map(|g| g.programs_count()).sum()
    }
}

impl From<&Machine> for SummaryMachine {
    fn from(value: &Machine) -> Self {
        Self {
            name: value.name.clone(),
            part_groups: value
                .part_groups
                .iter()
                .map(SummaryPartGroup::from)
                .collect(),
        }
    }
}
impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{} инстр.]", self.name, self.tools_count())?;
        if !self.tools.is_empty() {
            let tools: Vec<String> = self.tools.iter().map(|t| t.to_string()).collect();
            write!(f, " → {}", tools.join(", "))?;
        }
        Ok(())
    }
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.name)?;
        let count = self.programs.len();
        for (i, prog) in self.programs.iter().enumerate() {
            let branch = if i == count - 1 {
                "        └──"
            } else {
                "        ├──"
            };
            writeln!(f, "{} {}", branch, prog)?;
        }
        Ok(())
    }
}

impl fmt::Display for PartGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "  📂 {} (В среднем инструмента на программу: {})",
            self.name, self.avg_tools_count
        )?;
        Ok(())
    }
}

impl fmt::Display for Machine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "🔧 {}", self.name,)?;
        for group in &self.part_groups {
            write!(f, "{}", group)?;
        }
        Ok(())
    }
}
