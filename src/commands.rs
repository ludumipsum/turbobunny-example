use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

const CMD_NOTFOUND_URL: &str = "/404";

const GOOGLE_SEARCH_URL: &str = "https://www.google.com/search";
const LI_BUILDKITE_BASE: &str = "https://buildkite.com/ludumipsum";

const LI_GITHUB_BASE: &str = "https://www.github.com/ludumipsum";
const LI_MAIN_REPO: &str = "repo";

fn gen_google_cmd() -> BunnyCommand {
    BunnyCommand {
        matchers: vec!["g".to_string(), "google".to_string()],
        description: "search google with your arguments".to_string(),
        example: "g red rex rabbits".to_string(),
        destination: Destination::RedirectArgsString {
            url: GOOGLE_SEARCH_URL.to_string(),
            url_with_args: format!("{}?q=", GOOGLE_SEARCH_URL),
        },
    }
}

fn gen_cmd_list() -> Vec<BunnyCommand> {
    vec![
        gen_google_cmd(),
        BunnyCommand {
            matchers: vec!["bunny".to_string()],
            description: "open turbobunny's homepage".to_string(),
            example: "bunny".to_string(),
            destination: Destination::RedirectNoArgs {
                url: "/index".to_string(),
            },
        },
        BunnyCommand {
            matchers: vec!["gh".to_string()],
            description: "open the corresponding LI git repo".to_string(),
            example: "gh repo".to_string(),
            destination: Destination::RedirectArgsString {
                url: LI_GITHUB_BASE.to_string(),
                url_with_args: format!("{}/", LI_GITHUB_BASE),
            },
        },
        BunnyCommand {
            matchers: vec!["ghi".to_string()],
            description: "open the given github issue in ludumipsum/repo"
                .to_string(),
            example: "ghi 192".to_string(),
            destination: Destination::RedirectArgsString {
                url: format!(
                    "{}/{}/{}/",
                    LI_GITHUB_BASE, LI_MAIN_REPO, "issues"
                ),
                url_with_args: format!(
                    "{}/{}/{}/",
                    LI_GITHUB_BASE, LI_MAIN_REPO, "issues"
                ),
            },
        },
        BunnyCommand {
            matchers: vec!["pr".to_string(), "prs".to_string()],
            description: "open the given pr in ludumipsum/repo".to_string(),
            example: "ghi 192".to_string(),
            destination: Destination::RedirectArgsString {
                url: format!("{}/{}/{}", LI_GITHUB_BASE, LI_MAIN_REPO, "pulls"),
                url_with_args: format!(
                    "{}/{}/{}/",
                    LI_GITHUB_BASE, LI_MAIN_REPO, "pull"
                ),
            },
        },
        BunnyCommand {
            matchers: vec!["bk".to_string()],
            description: "open our buildkite dashboard".to_string(),
            example: "bk global-ci".to_string(),
            destination: Destination::RedirectArgsString {
                url: LI_BUILDKITE_BASE.to_string(),
                url_with_args: format!("{}/", LI_BUILDKITE_BASE),
            },
        },
    ]
}

#[derive(Serialize, Deserialize)]
pub struct BunnyCommandTable {
    pub commands: Vec<BunnyCommand>,
    pub fallback: Option<BunnyCommand>,
    pub fqdn: String,
    pub resources_path: PathBuf,
}
impl BunnyCommandTable {
    pub fn new<T: AsRef<str>>(fqdn: T, resources_path: &PathBuf) -> Self {
        Self {
            commands: gen_cmd_list(),
            fallback: Some(gen_google_cmd()),
            fqdn: fqdn.as_ref().to_string(),
            resources_path: resources_path.clone(),
        }
    }
    /// Get the first command in the table which matches the given string,
    /// along with the matcher pattern used to select it
    pub fn match_query<T: AsRef<str>>(
        &self,
        query: T,
    ) -> Option<(&BunnyCommand, String)> {
        for command in &self.commands {
            for matcher in &command.matchers {
                let query_str = query.as_ref();
                // Exact match on the query = call with no args
                if query_str == *matcher {
                    return Some((&command, "".to_string()));
                }
                // Partial match-plus-space = call with args
                let matcher_with_space = format!("{} ", matcher);
                if query_str.starts_with(&matcher_with_space) {
                    let leftover_args = query_str
                        .get(matcher_with_space.len()..)
                        .unwrap()
                        .to_string();
                    return Some((&command, leftover_args));
                }
            }
        }
        None
    }
    /// Get all the matchers which could potentially match the given string
    pub fn completions<T: AsRef<str>>(
        &self,
        query: T,
    ) -> Vec<(&BunnyCommand, &str)> {
        let mut ret: Vec<(&BunnyCommand, &str)> = vec![];
        for command in &self.commands {
            for matcher in &command.matchers {
                if matcher.starts_with(query.as_ref()) {
                    ret.push((command, matcher));
                }
            }
        }
        ret
    }
}

#[derive(Serialize, Deserialize)]
pub enum Destination {
    None,
    RedirectNoArgs { url: String },
    RedirectArgsString { url: String, url_with_args: String },
}
impl Default for Destination {
    fn default() -> Self {
        Destination::None
    }
}

#[derive(Serialize, Deserialize)]
pub struct BunnyCommand {
    pub matchers: Vec<String>,
    pub example: String,
    pub description: String,
    pub destination: Destination,
}
impl Default for BunnyCommand {
    fn default() -> Self {
        Self {
            matchers: vec![],
            example: "[missing_example]".to_string(),
            description: "[missing_description]".to_string(),
            destination: Destination::None,
        }
    }
}
impl BunnyCommand {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn run<T: AsRef<str>>(&self, args: T) -> String {
        let args: &str = args.as_ref();
        match &self.destination {
            Destination::None => CMD_NOTFOUND_URL.to_string(),
            Destination::RedirectNoArgs { url } => url.clone(),
            Destination::RedirectArgsString { url, url_with_args } => {
                if args.is_empty() {
                    url.clone()
                } else {
                    format!("{}{}", url_with_args, args)
                }
            }
        }
    }
}
