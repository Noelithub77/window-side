use crate::config::Config;
use crate::display::{console, looking_glass};
use crate::guest::commands::{self, UpdateAction};
use crate::guest::{metrics, sign_in, ssh};
use crate::host::{self, Profile};
use crate::notify::{self, Urgency};
use crate::process::require;
use crate::vm::{drivers, lifecycle, virsh};
use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

/// Run your dual-boot Windows install as a VM. Without a command, opens the window.
#[derive(Parser)]
#[command(name = "ws", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Action>,
}

#[derive(Subcommand)]
pub enum Action {
    /// Validate the host and define the libvirt domain
    Setup,
    /// Start Windows headlessly, wait for its guest agent and sign in
    Start,
    /// Start Windows if needed and show it (Looking Glass in the GPU profile)
    Launch,
    /// Show running Windows without changing its state
    Console,
    /// Request a clean Windows shutdown
    Stop,
    /// Immediately power off the VM
    ForceStop,
    /// Reboot Linux into the normal or GPU boot profile
    Reboot { profile: BootChoice },
    /// Root side of reboot, run through pkexec
    #[command(hide = true)]
    RebootRoot { entry: String },
    /// Open the Looking Glass client in the GPU profile
    LookingGlass,
    /// Install the signed Looking Glass virtual display in running Windows
    LookingGlassSetup,
    /// Attach the stable Windows guest-driver installer as a CD
    Drivers,
    /// Print the VM state and boot profile
    Status,
    /// Windows CPU, memory, uptime and network as JSON (10-second cache)
    Metrics,
    /// Run PowerShell as SYSTEM in Windows
    Exec {
        /// PowerShell command
        command: Option<String>,
        /// PowerShell script to run instead
        #[arg(long)]
        file: Option<PathBuf>,
        /// Seconds to wait
        #[arg(long, default_value_t = 600)]
        timeout: u64,
        /// Print exitCode, stdout and stderr as JSON
        #[arg(long)]
        json: bool,
    },
    /// Copy a host file into Windows, e.g. copy notes.txt 'C:\Users\Public\notes.txt'
    Copy { source: PathBuf, target: String },
    /// Key-only PowerShell over SSH, or --put/--get files (see ws ssh --help)
    Ssh {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Windows Update status, check or install
    Updates { action: UpdateAction },
    /// Save and apply a resolution on the VirtIO display of signed-in Windows
    Resolution {
        #[arg(long, default_value_t = 2560)]
        width: u32,
        #[arg(long, default_value_t = 1440)]
        height: u32,
        #[arg(long)]
        json: bool,
    },
    /// Type the keyring credential at the Windows sign-in screen
    SignIn {
        #[arg(value_enum, default_value_t = SignInAction::Run)]
        action: SignInAction,
        /// Seconds to wait for the desktop session
        #[arg(long, default_value_t = 40)]
        wait: u64,
    },
}

#[derive(Clone, Copy, ValueEnum)]
pub enum BootChoice {
    Normal,
    Gpu,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SignInAction {
    Run,
    Setup,
    Forget,
}

impl Action {
    /// Lifecycle failures also raise a desktop notification; polled queries stay quiet.
    pub fn notifies(&self) -> bool {
        matches!(
            self,
            Action::Setup | Action::Start | Action::Launch | Action::Console | Action::Stop | Action::ForceStop
                | Action::Reboot { .. } | Action::LookingGlass | Action::LookingGlassSetup | Action::Drivers
        )
    }
}

fn require_running(domain: &str) -> Result<()> {
    if virsh::is_running(domain) { Ok(()) } else { bail!("Start Windows first.") }
}

/// Signs in after start; a failure leaves Windows running at its sign-in screen.
fn sign_in_after_start(config: &Config) {
    let result = sign_in::run(&config.vm.name, &config.windows.user, Duration::from_secs(40));
    if !result.is_ok_and(|report| report["status"] != "not-signed-in") {
        notify::send(Urgency::Normal, "Windows sign-in did not finish. Sign in from the console.");
    }
}

fn show(config: &Config, profile: Profile) -> Result<()> {
    if profile == Profile::Gpu { looking_glass::open(config, profile) } else { console::open(&config.vm.name) }
}

pub fn run(action: Action) -> Result<i32> {
    // pkexec runs this as root, where the user's config is out of reach.
    if let Action::RebootRoot { entry } = &action {
        host::reboot_into(entry)?;
        return Ok(0);
    }
    let config = Config::load()?;
    let profile = host::boot_profile(config.gpu.as_ref());
    let domain = config.vm.name.as_str();
    match action {
        Action::Setup => lifecycle::setup_domain(&config, profile)?,
        Action::Start => {
            lifecycle::start(&config, profile)?;
            lifecycle::wait_for_agent(domain)?;
            sign_in_after_start(&config);
        }
        Action::Launch => {
            require(if profile == Profile::Gpu { "looking-glass-client" } else { "virt-viewer" })?;
            lifecycle::start(&config, profile)?;
            if profile != Profile::Gpu {
                console::open(domain)?;
            }
            lifecycle::wait_for_agent(domain)?;
            sign_in_after_start(&config);
            if profile == Profile::Gpu {
                looking_glass::open(&config, profile)?;
            }
            notify::send(Urgency::Normal, "Windows is running. Closing the viewer will not stop it.");
        }
        Action::Console => show(&config, profile)?,
        Action::Stop => lifecycle::stop(domain)?,
        Action::ForceStop => lifecycle::force_stop(domain)?,
        Action::Reboot { profile: choice } => {
            if virsh::is_running(domain) {
                bail!("Stop Windows before rebooting between GPU profiles.");
            }
            let (name, entry) = match choice {
                BootChoice::Normal => ("normal", &config.boot.normal_entry),
                BootChoice::Gpu => ("GPU", &config.boot.gpu_entry),
            };
            notify::send(Urgency::Critical, &format!("Rebooting into the {name} profile."));
            let program = std::env::current_exe().context("Could not find the ws binary")?;
            return Err(Command::new("pkexec").arg(program).args(["reboot-root", entry]).exec()).context("Could not run pkexec");
        }
        Action::RebootRoot { .. } => bail!("reboot-root is handled before loading the config"),
        Action::LookingGlass => looking_glass::open(&config, profile)?,
        Action::LookingGlassSetup => looking_glass::install(&config)?,
        Action::Drivers => drivers::attach(domain)?,
        Action::Status => {
            let state = virsh::state(domain);
            let state = if state.is_empty() { "not configured".to_owned() } else { state };
            println!("window-side: {state} (boot profile: {})", profile.name());
        }
        Action::Metrics => return metrics::show(&config, profile),
        Action::Exec { command, file, timeout, json } => {
            require_running(domain)?;
            return commands::exec(domain, command.as_deref(), file.as_deref(), timeout, json);
        }
        Action::Copy { source, target } => {
            require_running(domain)?;
            commands::copy(domain, &source, &target)?;
        }
        Action::Ssh { args } => return ssh::run(&config, &args),
        Action::Updates { action } => {
            if action != UpdateAction::Status {
                require_running(domain)?;
            }
            return commands::updates(domain, action);
        }
        Action::Resolution { width, height, json } => {
            require_running(domain)?;
            commands::resolution(domain, width, height, json)?;
        }
        Action::SignIn { action, wait } => match action {
            SignInAction::Setup => {
                sign_in::store_interactively(&config.windows.user)?;
                println!("{}", serde_json::json!({"status": "stored"}));
            }
            SignInAction::Forget => {
                sign_in::forget(&config.windows.user)?;
                println!("{}", serde_json::json!({"status": "forgotten"}));
            }
            SignInAction::Run => {
                require_running(domain)?;
                let report = sign_in::run(domain, &config.windows.user, Duration::from_secs(wait))?;
                println!("{report}");
                return Ok(if report["status"] == "not-signed-in" { 1 } else { 0 });
            }
        },
    }
    Ok(0)
}
