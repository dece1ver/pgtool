use anyhow::{Result, anyhow};
use lexopt::Arg::{Long, Short, Value};
use std::{path::PathBuf, process};
const VERSION: &str = env!("CARGO_PKG_VERSION");
const BIN_NAME: &str = env!("CARGO_PKG_NAME");

pub(crate) struct Args {
    /// Путь к корневой директории с программами станков
    pub(crate) archive: PathBuf,
    /// Включить полные данные (детали и программы) в JSON
    pub(crate) full: bool,
    /// Путь к выходному файлу (по умолчанию .\output.json)
    pub(crate) output: PathBuf,
}

impl Args {
    pub(crate) fn parse() -> Self {
        match Self::try_parse() {
            Ok(args) => args,
            Err(e) => {
                eprintln!("Ошибка: {e}");
                eprintln!("Попробуйте '{BIN_NAME} --help' для справки.");
                process::exit(2);
            }
        }
    }

    fn try_parse() -> Result<Self> {
        let mut parser = lexopt::Parser::from_env();
        let mut archive: Option<PathBuf> = None;
        let mut output: Option<PathBuf> = None;
        let mut full = false;
        while let Some(arg) = parser.next()? {
            match arg {
                Short('h') | Long("help") => {
                    Self::print_help();
                    process::exit(0);
                }
                Short('V') | Long("version") => {
                    println!("{BIN_NAME} {VERSION}");
                    process::exit(0);
                }
                Short('f') | Long("full") => {
                    full = true;
                }
                Short('o') | Long("output") => {
                    if output.is_some() {
                        return Err(anyhow!("флаг --output указан более одного раза"));
                    }
                    output = Some(parser.value()?.into());
                }
                Value(v) if archive.is_none() => {
                    archive = Some(v.into());
                }
                Value(v) => {
                    return Err(anyhow!(
                        "лишний аргумент: {:?}\n\
                         Путь к архиву уже задан. \
                         Для вывода используйте --output <ПУТЬ>.",
                        v
                    ));
                }
                arg => return Err(anyhow!("{}", arg.unexpected())),
            }
        }

        let archive = archive.ok_or_else(|| anyhow!("обязательный аргумент <АРХИВ> не указан"))?;

        if !archive.exists() {
            return Err(anyhow!("путь не существует: {}", archive.display()));
        }
        if !archive.is_dir() {
            return Err(anyhow!(
                "путь должен быть директорией, а не файлом: {}",
                archive.display()
            ));
        }

        let output = output.unwrap_or_else(|| PathBuf::from(".\\output.json"));

        Ok(Self {
            archive,
            full,
            output,
        })
    }

    fn print_help() {
        println!(
            "\
{BIN_NAME} {VERSION}
Сканирует директорию с управляющими программами и экспортирует информацию об инструментах в JSON.

ИСПОЛЬЗОВАНИЕ:
    {BIN_NAME} [ФЛАГИ] <АРХИВ> [--output <ПУТЬ>]

АРГУМЕНТЫ:
    <АРХИВ>          Путь к корневой директории с программами станков (обязательный)

ФЛАГИ:
    -o, --output <ПУТЬ>   Путь к выходному файлу (json|csv) [по умолчанию: output.json]
    -f, --full            Включить полные данные (детали и программы) в JSON
    -h, --help            Показать эту справку и выйти
    -V, --version         Показать версию и выйти

ФОРМАТЫ ПРОГРАММ:
    .pbg   Mazatrol              — бинарный формат
    .maz   Mazatrol              — бинарный формат
    .h     Heidenhain            — текстовый формат (в разработке)
    .mpf   Sinumerik             — текстовый формат (в разработке)
    *      G-code                — прочие расширения

ПРИМЕРЫ:
    {BIN_NAME} D:\\Programs
    {BIN_NAME} D:\\Programs --output .\\report.csv
    {BIN_NAME} D:\\Programs -fo C:\\reports\\tools.json"
        );
    }
}
