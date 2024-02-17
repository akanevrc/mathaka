use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "mathaka", author, version, about, long_about = None)]
struct Args {
    /// Problem file paths
    #[arg(short, long = "file")]
    files: Vec<String>,
}

fn solve_problem_files(files: Vec<String>) -> Result<String> {
    let mut props = Vec::new();
    for file in files {
        let prop = std::fs::read_to_string(file)?;
        props.push(prop);
    }
    let mut res = true;
    for prop in props {
        res = res && mathakalib::solve_problems(prop)?;
    }
    if res {
        Ok("TRUE - all propositions hold true.".to_owned())
    } else {
        Ok("FALSE - any propositions do not hold true.".to_owned())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("{}", solve_problem_files(args.files)?);
    Ok(())
}
