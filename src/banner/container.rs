use super::entry::BannerEntry;
use crate::{
    config::Configuration,
    event_handlers::Command,
    utils::{make_request, status_colorizer},
    VERSION,
};
use anyhow::{bail, Result};
use console::{style, Emoji};
use reqwest::{Client, Url};
use serde_json::Value;
use std::io::Write;
use tokio::sync::mpsc::UnboundedSender;

/// Url used to query github's api; specifically used to look for the latest tagged release name
pub const UPDATE_URL: &str = "https://api.github.com/repos/epi052/feroxbuster/releases/latest";

/// Simple enum to hold three different update states
#[derive(Debug)]
pub(super) enum UpdateStatus {
    /// this version and latest release are the same
    UpToDate,

    /// this version and latest release are not the same
    OutOfDate,

    /// some error occurred during version check
    Unknown,
}

/// Banner object, contains multiple BannerEntry's and knows how to display itself
pub struct Banner {
    /// all live targets
    targets: Vec<BannerEntry>,

    /// represents Configuration.status_codes
    status_codes: BannerEntry,

    /// represents Configuration.filter_status
    filter_status: BannerEntry,

    /// represents Configuration.threads
    threads: BannerEntry,

    /// represents Configuration.wordlist
    wordlist: BannerEntry,

    /// represents Configuration.timeout
    timeout: BannerEntry,

    /// represents Configuration.user_agent
    user_agent: BannerEntry,

    /// represents Configuration.config
    config: BannerEntry,

    /// represents Configuration.proxy
    proxy: BannerEntry,

    /// represents Configuration.replay_proxy
    replay_proxy: BannerEntry,

    /// represents Configuration.replay_codes
    replay_codes: BannerEntry,

    /// represents Configuration.headers
    headers: Vec<BannerEntry>,

    /// represents Configuration.filter_size
    filter_size: Vec<BannerEntry>,

    /// represents Configuration.filter_similar
    filter_similar: Vec<BannerEntry>,

    /// represents Configuration.filter_word_count
    filter_word_count: Vec<BannerEntry>,

    /// represents Configuration.filter_line_count
    filter_line_count: Vec<BannerEntry>,

    /// represents Configuration.filter_regex
    filter_regex: Vec<BannerEntry>,

    /// represents Configuration.extract_links
    extract_links: BannerEntry,

    /// represents Configuration.json
    json: BannerEntry,

    /// represents Configuration.output
    output: BannerEntry,

    /// represents Configuration.debug_log
    debug_log: BannerEntry,

    /// represents Configuration.extensions
    extensions: BannerEntry,

    /// represents Configuration.insecure
    insecure: BannerEntry,

    /// represents Configuration.redirects
    redirects: BannerEntry,

    /// represents Configuration.dont_filter
    dont_filter: BannerEntry,

    /// represents Configuration.queries
    queries: Vec<BannerEntry>,

    /// represents Configuration.verbosity
    verbosity: BannerEntry,

    /// represents Configuration.add_slash
    add_slash: BannerEntry,

    /// represents Configuration.no_recursion
    no_recursion: BannerEntry,

    /// represents Configuration.scan_limit
    scan_limit: BannerEntry,

    /// represents Configuration.time_limit
    time_limit: BannerEntry,

    /// current version of feroxbuster
    pub(super) version: String,

    /// whether or not there is a known new version
    pub(super) update_status: UpdateStatus,
}

/// implementation of Banner
impl Banner {
    /// Create a new Banner from a Configuration and live targets
    pub fn new(tgts: &[String], config: &Configuration) -> Self {
        let mut targets = Vec::new();
        let mut code_filters = Vec::new();
        let mut replay_codes = Vec::new();
        let mut headers = Vec::new();
        let mut filter_size = Vec::new();
        let mut filter_similar = Vec::new();
        let mut filter_word_count = Vec::new();
        let mut filter_line_count = Vec::new();
        let mut filter_regex = Vec::new();
        let mut queries = Vec::new();

        for target in tgts {
            targets.push(BannerEntry::new("🎯", "Target Url", target));
        }

        let mut codes = vec![];
        for code in &config.status_codes {
            codes.push(status_colorizer(&code.to_string()))
        }
        let status_codes =
            BannerEntry::new("👌", "Status Codes", &format!("[{}]", codes.join(", ")));

        for code in &config.filter_status {
            code_filters.push(status_colorizer(&code.to_string()))
        }
        let filter_status = BannerEntry::new(
            "🗑",
            "Status Code Filters",
            &format!("[{}]", code_filters.join(", ")),
        );

        for code in &config.replay_codes {
            replay_codes.push(status_colorizer(&code.to_string()))
        }
        let replay_codes = BannerEntry::new(
            "📼",
            "Replay Proxy Codes",
            &format!("[{}]", replay_codes.join(", ")),
        );

        for (name, value) in &config.headers {
            headers.push(BannerEntry::new(
                "🤯",
                "Header",
                &format!("{}: {}", name, value),
            ));
        }

        for filter in &config.filter_size {
            filter_size.push(BannerEntry::new("💢", "Size Filter", &filter.to_string()));
        }

        for filter in &config.filter_similar {
            filter_similar.push(BannerEntry::new("💢", "Similarity Filter", filter));
        }

        for filter in &config.filter_word_count {
            filter_word_count.push(BannerEntry::new(
                "💢",
                "Word Count Filter",
                &filter.to_string(),
            ));
        }

        for filter in &config.filter_line_count {
            filter_line_count.push(BannerEntry::new(
                "💢",
                "Line Count Filter",
                &filter.to_string(),
            ));
        }

        for filter in &config.filter_regex {
            filter_regex.push(BannerEntry::new("💢", "Regex Filter", filter));
        }

        for query in &config.queries {
            queries.push(BannerEntry::new(
                "🤔",
                "Query Parameter",
                &format!("{}={}", query.0, query.1),
            ));
        }

        let volume = ["🔈", "🔉", "🔊", "📢"];
        let verbosity = if let 1..=4 = config.verbosity {
            //speaker medium volume (increasing with verbosity to loudspeaker)
            BannerEntry::new(
                volume[config.verbosity as usize - 1],
                "Verbosity",
                &config.verbosity.to_string(),
            )
        } else {
            BannerEntry::default()
        };

        let no_recursion = if !config.no_recursion {
            let depth = if config.depth == 0 {
                "INFINITE".to_string()
            } else {
                config.depth.to_string()
            };

            BannerEntry::new("🔃", "Recursion Depth", &depth)
        } else {
            BannerEntry::new("🚫", "Do Not Recurse", &config.no_recursion.to_string())
        };

        let scan_limit = BannerEntry::new(
            "🦥",
            "Concurrent Scan Limit",
            &config.scan_limit.to_string(),
        );

        let replay_proxy = BannerEntry::new("🎥", "Replay Proxy", &config.replay_proxy);
        let cfg = BannerEntry::new("💉", "Config File", &config.config);
        let proxy = BannerEntry::new("💎", "Proxy", &config.proxy);
        let threads = BannerEntry::new("🚀", "Threads", &config.threads.to_string());
        let wordlist = BannerEntry::new("📖", "Wordlist", &config.wordlist);
        let timeout = BannerEntry::new("💥", "Timeout (secs)", &config.timeout.to_string());
        let user_agent = BannerEntry::new("🦡", "User-Agent", &config.user_agent);
        let extract_links =
            BannerEntry::new("🔎", "Extract Links", &config.extract_links.to_string());
        let json = BannerEntry::new("🧔", "JSON Output", &config.json.to_string());
        let output = BannerEntry::new("💾", "Output File", &config.output);
        let debug_log = BannerEntry::new("🪲", "Debugging Log", &config.debug_log);
        let extensions = BannerEntry::new(
            "💲",
            "Extensions",
            &format!("[{}]", config.extensions.join(", ")),
        );
        let insecure = BannerEntry::new("🔓", "Insecure", &config.insecure.to_string());
        let redirects = BannerEntry::new("📍", "Follow Redirects", &config.redirects.to_string());
        let dont_filter =
            BannerEntry::new("🤪", "Filter Wildcards", &(!config.dont_filter).to_string());
        let add_slash = BannerEntry::new("🪓", "Add Slash", &config.add_slash.to_string());
        let time_limit = BannerEntry::new("🕖", "Time Limit", &config.time_limit);

        Self {
            targets,
            status_codes,
            threads,
            wordlist,
            filter_status,
            timeout,
            user_agent,
            proxy,
            replay_codes,
            replay_proxy,
            headers,
            filter_size,
            filter_similar,
            filter_word_count,
            filter_line_count,
            filter_regex,
            extract_links,
            json,
            queries,
            output,
            debug_log,
            extensions,
            insecure,
            dont_filter,
            redirects,
            verbosity,
            add_slash,
            no_recursion,
            scan_limit,
            time_limit,
            config: cfg,
            version: VERSION.to_string(),
            update_status: UpdateStatus::Unknown,
        }
    }

    /// get a fancy header for the banner
    fn header(&self) -> String {
        let artwork = format!(
            r#"
 ___  ___  __   __     __      __         __   ___
|__  |__  |__) |__) | /  `    /  \ \_/ | |  \ |__
|    |___ |  \ |  \ | \__,    \__/ / \ | |__/ |___
by Ben "epi" Risher {}                 ver: {}"#,
            Emoji("🤓", &format!("{:<2}", "\u{0020}")),
            self.version
        );

        let top = "───────────────────────────┬──────────────────────";

        format!("{}\n{}", artwork, top)
    }

    /// get a fancy footer for the banner
    fn footer(&self) -> String {
        let addl_section = "──────────────────────────────────────────────────";
        let bottom = "───────────────────────────┴──────────────────────";

        let instructions = format!(
            " 🏁  Press [{}] to use the {}™",
            style("ENTER").yellow(),
            style("Scan Cancel Menu").bright().yellow(),
        );

        format!("{}\n{}\n{}", bottom, instructions, addl_section)
    }

    /// Makes a request to the given url, expecting to receive a JSON response that contains a field
    /// named `tag_name` that holds a value representing the latest tagged release of this tool.
    ///
    /// ex: v1.1.0
    pub async fn check_for_updates(
        &mut self,
        client: &Client,
        url: &str,
        tx_stats: UnboundedSender<Command>,
    ) -> Result<()> {
        log::trace!("enter: needs_update({:?}, {}, {:?})", client, url, tx_stats);

        let api_url = Url::parse(url)?;

        let response = make_request(&client, &api_url, tx_stats.clone()).await?;

        let body = response.text().await?;

        let json_response: Value = serde_json::from_str(&body)?;

        let latest_version = match json_response["tag_name"].as_str() {
            Some(tag) => tag.trim_start_matches('v'),
            None => {
                bail!("JSON has no tag_name: {}", json_response);
            }
        };

        // if we've gotten this far, we have a string in the form of X.X.X where X is a number
        // all that's left is to compare the current version with the version found above

        return if latest_version == self.version {
            // there's really only two possible outcomes if we accept that the tag conforms to
            // the X.X.X pattern:
            //   1. the version strings match, meaning we're up to date
            //   2. the version strings do not match, meaning we're out of date
            //
            // except for developers working on this code, nobody should ever be in a situation
            // where they have a version greater than the latest tagged release
            self.update_status = UpdateStatus::UpToDate;
            Ok(())
        } else {
            self.update_status = UpdateStatus::OutOfDate;
            Ok(())
        };
    }

    /// display the banner on Write writer
    pub fn print_to<W>(&self, mut writer: W, config: &Configuration) -> Result<()>
    where
        W: Write,
    {
        writeln!(&mut writer, "{}", self.header())?;

        // begin with always printed items
        for target in &self.targets {
            writeln!(&mut writer, "{}", target)?;
        }

        writeln!(&mut writer, "{}", self.threads)?;
        writeln!(&mut writer, "{}", self.wordlist)?;
        writeln!(&mut writer, "{}", self.status_codes)?;

        if !config.filter_status.is_empty() {
            // exception here for an optional print in the middle of always printed values is due
            // to me wanting the allows and denys to be printed one after the other
            writeln!(&mut writer, "{}", self.filter_status)?;
        }

        writeln!(&mut writer, "{}", self.timeout)?;
        writeln!(&mut writer, "{}", self.user_agent)?;

        // followed by the maybe printed or variably displayed values
        if !config.config.is_empty() {
            writeln!(&mut writer, "{}", self.config)?;
        }

        if !config.proxy.is_empty() {
            writeln!(&mut writer, "{}", self.proxy)?;
        }

        if !config.replay_proxy.is_empty() {
            // i include replay codes logic here because in config.rs, replay codes are set to the
            // value in status codes, meaning it's never empty
            writeln!(&mut writer, "{}", self.replay_proxy)?;
            writeln!(&mut writer, "{}", self.replay_codes)?;
        }

        for header in &self.headers {
            writeln!(&mut writer, "{}", header)?;
        }

        for filter in &self.filter_size {
            writeln!(&mut writer, "{}", filter)?;
        }

        for filter in &self.filter_similar {
            writeln!(&mut writer, "{}", filter)?;
        }

        for filter in &self.filter_word_count {
            writeln!(&mut writer, "{}", filter)?;
        }

        for filter in &self.filter_line_count {
            writeln!(&mut writer, "{}", filter)?;
        }

        for filter in &self.filter_regex {
            writeln!(&mut writer, "{}", filter)?;
        }

        if config.extract_links {
            writeln!(&mut writer, "{}", self.extract_links)?;
        }

        if config.json {
            writeln!(&mut writer, "{}", self.json)?;
        }

        for query in &self.queries {
            writeln!(&mut writer, "{}", query)?;
        }

        if !config.output.is_empty() {
            writeln!(&mut writer, "{}", self.output)?;
        }

        if !config.debug_log.is_empty() {
            writeln!(&mut writer, "{}", self.debug_log)?;
        }

        if !config.extensions.is_empty() {
            writeln!(&mut writer, "{}", self.extensions)?;
        }

        if config.insecure {
            writeln!(&mut writer, "{}", self.insecure)?;
        }

        if config.redirects {
            writeln!(&mut writer, "{}", self.redirects)?;
        }

        if config.dont_filter {
            writeln!(&mut writer, "{}", self.dont_filter)?;
        }

        if let 1..=4 = config.verbosity {
            writeln!(&mut writer, "{}", self.verbosity)?;
        }

        if config.add_slash {
            writeln!(&mut writer, "{}", self.add_slash)?;
        }

        writeln!(&mut writer, "{}", self.no_recursion)?;

        if config.scan_limit > 0 {
            writeln!(&mut writer, "{}", self.scan_limit)?;
        }
        if !config.time_limit.is_empty() {
            writeln!(&mut writer, "{}", self.time_limit)?;
        }

        if matches!(self.update_status, UpdateStatus::OutOfDate) {
            let update = BannerEntry::new(
                "🎉",
                "New Version Available",
                "https://github.com/epi052/feroxbuster/releases/latest",
            );
            writeln!(&mut writer, "{}", update)?;
        }

        writeln!(&mut writer, "{}", self.footer())?;

        Ok(())
    }
}