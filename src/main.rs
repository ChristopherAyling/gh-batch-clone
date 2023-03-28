use clap::Parser;
use octocrab::models;
use std::{thread, time};
use std::fs;
use std::path;
use std::process::Command;


#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    org: String,
    #[clap(short, long)]
    token: Option<String>,
    #[clap(short, long)]
    clonedir: String,
}

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let args = Args::parse();
    let token = args.token;
    let org = args.org;
    let clonedir = path::PathBuf::from(args.clonedir).join(org.clone());


    let octocrab = match  token {
        Some(token) => octocrab::Octocrab::builder()
            .personal_token(token)
            .build()?,
        None => octocrab::Octocrab::builder()
            .build()?,
    };

    let mut page = octocrab.orgs(org).list_repos().per_page(50).send().await?;

    loop {
        for repo in &page {
            let name = repo.name.clone();
            let local_path = clonedir.join(name);
            let url = repo.clone_url.clone().unwrap();
            let url = url.as_str();
            if !local_path.exists() {
                fs::create_dir_all(local_path.clone().parent().unwrap()).unwrap();
                match Command::new("git")
                    .arg("clone")
                    .arg(url)
                    .arg(local_path)
                    .output() {
                        Ok(output) => {
                            println!("Cloned {}", url);
                            println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                            println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                        },
                        Err(e) => println!("Failed to clone: {}", e),
                    }
            } else {
                println!("{} already exists", local_path.display());
                match Command::new("git")
                    .arg("-C")
                    .arg(local_path.clone())
                    .arg("pull")
                    .output() {
                        Ok(output) => {
                            println!("Pulled {}", url);
                            println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                            println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                        },
                        Err(e) => println!("Failed to pull: {}", e),
                    }
            }
        }    
        page = match octocrab.get_page::<models::Repository>(&page.next).await? {
            Some(next_page) => next_page,
            None => break
        };
        thread::sleep(time::Duration::from_secs(2));
    }
        
    Ok(())
}
