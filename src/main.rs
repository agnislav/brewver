use std::fmt;
use std::io::{Read, Write};
use clap::Parser;
use clap_derive::Parser;
use tempfile::{Builder, NamedTempFile, TempDir};
use log::{info, debug, error};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    show_github_token_info();

    let args = Args::parse();
    Formula::new(args.formula_name, args.formula_version).init();
}

fn show_github_token_info() {
    if std::env::var("GITHUB_TOKEN").is_ok() {
        info!("Personal Access Token is used.");
    } else {
        info!("This program uses the GitHub API to fetch data. To increase the rate limit, you can set a GITHUB_TOKEN environment variable.");
        info!("To set the GITHUB_TOKEN, use the following command in your terminal:");
        info!("export GITHUB_TOKEN=your_personal_access_token");
        info!("You can create a personal access token at https://github.com/settings/tokens");
    }
}

#[derive(Parser)]
#[clap(version = "0.1", author = "Agnislav Onufriichuk", about = "Installs a specific version of a Homebrew formula")]
struct Args {
    #[clap(help = "The name of the formula")]
    formula_name: String,

    #[clap(help = "The version of the formula")]
    formula_version: String,
}

struct Formula {
    name: String,
    version: String,
    repo_path: Option<String>,
    commit: Option<String>,
    url: Option<String>,
    temp_dir: Option<TempDir>,
    bottle_file: Option<NamedTempFile>,
}

impl fmt::Debug for Formula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Formula: {}\nVersion: {}\nCommit: {:?}\nURL: {:?}", self.name, self.version, self.commit, self.url)
    }
}

impl Formula {
    fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            repo_path: None,
            commit: None,
            url: None,
            temp_dir: None,
            bottle_file: None,
        }
    }

    fn init(&mut self) -> &mut Self {
        if let Err(e) = self.get_commit_hash() {
            error!("Failed to get commit hash: {:?}", e);
        } else if let Err(e) = self.download() {
            error!("Failed to download: {:?}", e);
        } else if let Err(e) = self.install() {
            error!("Failed to install: {:?}", e);
        } else {
            info!("Formula {}@{} was installed successfully", self.name, self.version);
            debug!("Formula: {:?}", self);
        }
        self
    }

    fn get_commit_hash(&mut self) -> Result<&mut Self, Box<dyn std::error::Error>> {
        info!("Looking for {}@{}", self.name, self.version);
        
        for file_path in get_file_path(&self.name) {
            let url = format_gh_api_commits_url(&file_path);
            debug!("URL: {:?}", &url);

            let request = create_client(&url);
            let response = request.send()?;

            let json: serde_json::Value = response.json()?;

            if let Some(commit) = json.as_array().and_then(
                |arr| arr.iter().find(|commit| self.is_matching_commit(commit))
            ) {
                info!("Found Commit: {}", commit.get("sha").and_then(|s| s.as_str()).unwrap_or_default());
                self.commit = commit.get("sha").and_then(|s| s.as_str()).map(String::from);
                self.url = self.commit.as_ref().map(|commit| format_gh_api_raw_file_url(commit, &file_path));
                self.repo_path = Some(file_path.clone());
                return Ok(self);
            }
        }
        Err("Commit not found".into())
    }

    fn is_matching_commit(&self, commit: &serde_json::Value) -> bool {
        commit.get("commit")
            .and_then(|c| c.get("message"))
            .and_then(|m| m.as_str())
            .map_or(false, |msg| msg.contains(&self.commit_message()))
    }

    fn commit_message(&self) -> String {
        format!("{}: update {} bottle", self.name, self.version)
    }

    fn download(&mut self) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let request = create_client(self.url.as_ref().unwrap());
        let response = request.send()?;
        let file_content = response.text()?;

        // create temp file
        let tmp_dir = Builder::new().tempdir()?;
        let mut temp_file = Builder::new()
            .prefix(&self.name)
            .suffix(".rb")
            .rand_bytes(0)
            .tempfile_in(tmp_dir.path())?;

        debug!("Temp File: {:?}", &temp_file.path());

        temp_file.write_all(file_content.as_bytes())?;
        self.temp_dir = Some(tmp_dir);
        self.bottle_file = Some(temp_file);
        Ok(self)
    }

    fn install(&mut self) -> Result<&mut Self, Box<dyn std::error::Error>> {
        self.run_command("brew", &["remove", &self.name])?;
        debug!("Install from File: {:?}", &self.bottle_file.as_ref().unwrap().path());

        let mut file = std::fs::File::open(self.bottle_file.as_ref().unwrap().path())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        debug!("Bottle File Content: {}", contents);

        self.run_command("brew", &["install", self.bottle_file.as_ref().unwrap().path().to_str().unwrap()])?;
        Ok(self)
    }

    fn run_command(&self, command: &str, args: &[&str]) -> Result<std::process::Output, Box<dyn std::error::Error>> {
        let output = std::process::Command::new(command)
            .args(args)
            .output()?;
        debug!("Command output: {:?}", output);
        Ok(output)
    }
}

fn create_client(url: &str) -> reqwest::blocking::RequestBuilder {
    let client = reqwest::blocking::Client::new();
    let mut request_builder = client.get(url)
        .header("User-Agent", "BrewVer/0.1");

    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
    }

    request_builder
}

fn format_gh_api_commits_url(file_path: &str) -> String {
    format!("https://api.github.com/repos/Homebrew/homebrew-core/commits?path={}&per_page=100", file_path)
}

fn format_gh_api_raw_file_url(commit: &str, file_path: &str) -> String {
    format!("https://raw.githubusercontent.com/Homebrew/homebrew-core/{}{}", commit, file_path)
}

fn get_file_path(name: &str) -> [String; 2] {
    let first_letter = name.chars().next().unwrap();
    [
        format!("/Formula/{}/{}.rb", first_letter, name),
        format!("/Formula/{}.rb", name),
    ]
}
