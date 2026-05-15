use serde::{Deserialize, Serialize};
use colored::Colorize;
use clap::{Parser, Subcommand};
use home::home_dir;
use anyhow::Result;
use tokio::fs;
use dialoguer::Confirm;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    List,
    Cat {
        index: usize,

        
        #[arg(short, long)]
        critical: Option<bool>
    },
    New {
        name: String,
        content: Option<String>,

        #[arg(short, long)]
        critical: Option<bool>,
    },
    Delete {
        index: usize,

        
        #[arg(short, long)]
        critical: Option<bool>,

        #[arg(short, long)]
        noconfirm: Option<bool>,
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Root {
    critical: Vec<Task>,
    common: Vec<Task>
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Task {
    name: String,
    content: Option<String>
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => {
            let data = get_data().await?;
            // if let Some(critical_notes) = data.critical {
            //     for i in critical_notes { println!("{}", format!("! {}", i.name).red()) }
            // }
            // if let Some(common_notes) = data.common {
            //     for i in common_notes { println!("{}", i.name) }
            // }
            for (i, v) in data.critical.iter().enumerate() {
                println!("{}. {}", i.to_string().yellow(), format!("! {}", v.name).red());
            }
            for (i, v) in data.common.iter().enumerate() {
                println!("{}. - {}", i.to_string().yellow(), v.name);
            }
        },
        Commands::Cat { index, critical } => {
            let data = get_data().await?;

            match critical {
                Some(true) => {
                    println!("{}. {}:", index.to_string().yellow(),
                        format!("! {}", data.critical[index].name).red()
                    );
                    if let Some(content) = &data.critical[index].content { println!("{}", content);}
                },
                Some(false) | None => {
                    println!("{}. - {}:", index.to_string().yellow(),
                        format!("{}", data.common[index].name).yellow()
                    );
                    if let Some(content) = &data.common[index].content { println!("{}", content);}
                }
            }
        },
        Commands::New { name, content, critical } => {
            let mut data = get_data().await?;

            match critical {
                Some(true) => {
                    data.critical.push(Task {name: name, content: content});
                },
                Some(false) | None => {
                    data.common.push(Task {name: name, content: content});
                }
            }

            let _ = save_data(data).await?;
        }
        Commands::Delete { index, critical, noconfirm } => {
            let mut data = get_data().await?;

            match critical {
                Some(true) => {
                    if noconfirm == Some(true) { let _ = data.critical.remove(index); } else {
                        println!("Задача: {}", data.critical[index].name.red());
                        if Confirm::new().with_prompt("Удалить?").interact()? {
                            let _ = data.critical.remove(index);
                        }
                    }
                },
                Some(false) | None => {
                    if noconfirm == Some(true) { let _ = data.common.remove(index); } else {
                        println!("Задача: {}", data.common[index].name.yellow());
                        if Confirm::new().with_prompt("Удалить?").interact()? {
                            let _ = data.common.remove(index);
                        }
                    }
                }
            }

            let _ = save_data(data).await?;
        }
    }

    Ok(())
}

async fn get_data() -> Result<Root> {
    let mut data_dir = home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    data_dir.push(".clorine/notes");
    if !data_dir.exists() { fs::create_dir_all(&data_dir).await?; }
    let data_file = data_dir.join("data.json");
    if data_file.exists() {
        Ok(ron::from_str(&fs::read_to_string(data_file).await?)?)
    } else {
        Ok(Root{critical: Vec::new(), common: Vec::new()})
    }
}

async fn save_data(data: Root) -> Result<()> {
    let mut data_dir = home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    data_dir.push(".clorine/notes");
    if !data_dir.exists() { fs::create_dir_all(&data_dir).await?; }
    let data_file = data_dir.join("data.json");
    fs::write(data_file, ron::to_string(&data)?).await?;
    Ok(())
}

