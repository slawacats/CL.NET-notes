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
    Display,
    Cat {
        index: usize,
        
        #[arg(short, long)]
        special: bool
    },
    New {
        name: String,
        content: Option<String>,

        #[arg(short, long)]
        special: bool,
    },
    Delete {
        index: usize,
        
        #[arg(short, long)]
        special: bool,

        #[arg(short, long)]
        force: bool,
    },
    PrintJson // you can use this to create your own interface
}

#[derive(Debug, Deserialize, Serialize)]
struct Root {
    special: Vec<Task>,
    common: Vec<Task>
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Task {
    name: String,
    content: Option<String>
}

#[derive(Debug, Deserialize)]
struct Config {
    style: Option<Style>,
}

#[derive(Debug, Deserialize)]
struct Style {
    #[serde(default)]
    before_text: String,

    #[serde(default)]
    after_text: String,

    #[serde(default)]
    before_specials: String,

    #[serde(default)]
    after_specials: String,

    #[serde(default)]
    before_commons: String,

    #[serde(default)]
    after_commons: String,

    #[serde(default)]
    before_special_unit: String,

    #[serde(default)]
    after_special_unit: String,

    #[serde(default)]
    before_common_unit: String,

    #[serde(default)]
    after_common_unit: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => {
            let data = get_data().await?;

            for (i, v) in data.special.iter().enumerate() {
                println!("{}. {}", i.to_string().yellow(), format!("! {}", v.name).red());
            }
            for (i, v) in data.common.iter().enumerate() {
                println!("{}. - {}", i.to_string().yellow(), v.name);
            }
        },
        Commands::Display => {
            let data = get_data().await?;

            let conf = get_config().await?;

            if let Some(style) = conf.style {
                print!("{}", style.before_text);
                
                print!("{}", style.before_specials);
                for (i, v) in data.special.iter().enumerate() {
                    println!("{}{}. {}{}", style.before_special_unit,
                        i.to_string().yellow(),
                        format!("! {}", v.name).red(),
                        style.after_special_unit
                    );
                }
                print!("{}", style.after_specials);

                print!("{}", style.before_commons);
                for (i, v) in data.common.iter().enumerate() {
                    println!("{}{}. - {}{}", style.before_common_unit,
                        i.to_string().yellow(),
                        v.name,
                        style.after_common_unit
                    );
                }
                print!("{}", style.after_commons);
                
                print!("{}", style.after_text);
            } else {
                for (i, v) in data.special.iter().enumerate() {
                    println!("{}. {}", i.to_string().yellow(), format!("! {}", v.name).red());
                }
                for (i, v) in data.common.iter().enumerate() {
                    println!("{}. - {}", i.to_string().yellow(), v.name);
                }
            }
        },
        Commands::Cat { index, special } => {
            let data = get_data().await?;

            match special {
                true => {
                    println!("{}. {}:", index.to_string().yellow(),
                        format!("! {}", data.special[index].name).red()
                    );
                    if let Some(content) = &data.special[index].content { println!("{}", content);}
                },
                false => {
                    println!("{}. - {}:", index.to_string().yellow(),
                        format!("{}", data.common[index].name).yellow()
                    );
                    if let Some(content) = &data.common[index].content { println!("{}", content);}
                }
            }
        },
        Commands::New { name, content, special } => {
            let mut data = get_data().await?;

            match special {
                true => {
                    data.special.push(Task {name: name, content: content});
                },
                false => {
                    data.common.push(Task {name: name, content: content});
                }
            }

            let _ = save_data(data).await?;
        },
        Commands::Delete { index, special, force } => {
            let mut data = get_data().await?;

            match special {
                true => {
                    if force == true { let _ = data.special.remove(index); } else {
                        println!("Задача: {}", data.special[index].name.red());
                        if Confirm::new().with_prompt("Удалить?").interact()? {
                            let _ = data.special.remove(index);
                        }
                    }
                },
                false => {
                    if force == true { let _ = data.common.remove(index); } else {
                        println!("Задача: {}", data.common[index].name.yellow());
                        if Confirm::new().with_prompt("Удалить?").interact()? {
                            let _ = data.common.remove(index);
                        }
                    }
                }
            }

            let _ = save_data(data).await?;
        },
        Commands::PrintJson => {
            let data = get_data().await?;

            println!("{}", serde_json::to_string(&data)?);
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
        Ok(Root{special: Vec::new(), common: Vec::new()})
    }
}

async fn save_data(data: Root) -> Result<()> {
    let mut data_dir = home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    data_dir.push(".clorine/notes");
    if !data_dir.exists() { fs::create_dir_all(&data_dir).await?; }
    let data_file = data_dir.join("data.json");
    fs::write(data_file, ron::ser::to_string_pretty(&data,
            ron::ser::PrettyConfig::new().depth_limit(4).indentor("  ".to_string())
            )?).await?;
    Ok(())
}

async fn get_config() -> Result<Config> {
    let mut data_dir = home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    data_dir.push(".clorine/notes");
    if !data_dir.exists() { fs::create_dir_all(&data_dir).await?; }
    let data_file = data_dir.join("config.toml");
    if data_file.exists() {
        Ok(toml::from_str(&fs::read_to_string(data_file).await?)?)
    } else {
        Ok(Config{style: None})
    }
}
