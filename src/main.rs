use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() -> anyhow::Result<()> {
    let Args { name, count } = Args::parse();

    for _ in 0..count {
        println!("Hello {}!", name);
    }

    Ok(())
}
