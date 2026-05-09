use std::sync::mpsc;

use crate::pipeline::Pipeline;
use crate::profile::{AiAgent, Database, DeploymentTarget, Profile, ReverseProxy};
use crate::ssh::{SshClient, SshEvent};

#[derive(Clone)]
pub enum Sender {
    System,
    #[allow(dead_code)]
    User,
    Assistant,
    #[allow(dead_code)]
    Tool { name: String },
}

#[derive(Clone)]
pub struct Message {
    pub sender: Sender,
    pub content: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Focus {
    Sidebar,
    Content,
    Input,
    Logs,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Failed,
}

pub const MENU_ITEMS: &[&str] = &[
    "DEPLOY",
    "SETUP AGENT",
    "CONNECT VPS",
    "HISTORY",
    "CONNECTION",
    "SESSION",
];

pub const SIDEBAR_W: u16 = 22;
pub const SIDEBAR_ITEM_Y0: u16 = 2;
pub const SIDEBAR_ITEM_STRIDE: u16 = 4;
pub const VERSION: &str = "0.1.0";
pub const SCROLL_PAGE_SIZE: usize = 10;

pub fn sidebar_hit_test(row: u16) -> Option<usize> {
    for i in 0..MENU_ITEMS.len() {
        let y0 = SIDEBAR_ITEM_Y0 + (i as u16) * SIDEBAR_ITEM_STRIDE;
        if row >= y0 && row <= y0 + 3 {
            return Some(i);
        }
    }
    None
}

#[derive(Clone)]
pub struct DeployTargetState {
    pub target: DeploymentTarget,
    pub selected: bool,
}

pub struct ProfileForm {
    pub active: bool,
    pub name: String,
    pub host: String,
    pub port: String,
    pub username: String,
    pub key_path: String,
    pub cursor: usize,
}

impl ProfileForm {
    pub fn open() -> Self {
        Self {
            active: true,
            name: String::new(),
            host: String::new(),
            port: String::from("22"),
            username: String::from("root"),
            key_path: std::env::var("HOME")
                .map(|h| format!("{h}/.ssh/id_rsa"))
                .unwrap_or_default(),
            cursor: 0,
        }
    }

    pub fn field_count(&self) -> usize {
        6
    }

    pub fn field_mut(&mut self, i: usize) -> Option<&mut String> {
        match i {
            0 => Some(&mut self.name),
            1 => Some(&mut self.host),
            2 => Some(&mut self.port),
            3 => Some(&mut self.username),
            4 => Some(&mut self.key_path),
            _ => None,
        }
    }

    pub fn field_label(i: usize) -> &'static str {
        match i {
            0 => "Name",
            1 => "Host",
            2 => "Port",
            3 => "User",
            4 => "Key Path",
            _ => "",
        }
    }
}

const ALL_TARGETS: &[DeploymentTarget] = &[
    DeploymentTarget::Hardening,
    DeploymentTarget::Database(Database::Postgres),
    DeploymentTarget::Database(Database::Redis),
    DeploymentTarget::Database(Database::MongoDB),
    DeploymentTarget::ReverseProxy(ReverseProxy::Caddy),
    DeploymentTarget::ReverseProxy(ReverseProxy::Nginx),
    DeploymentTarget::AiAgent(AiAgent::Hermes),
    DeploymentTarget::AiAgent(AiAgent::OpenClaw),
    DeploymentTarget::DockerCompose,
    DeploymentTarget::Binary,
];

fn default_deploy_targets() -> Vec<DeployTargetState> {
    ALL_TARGETS.iter().map(|t| DeployTargetState {
        target: t.clone(),
        selected: matches!(t, DeploymentTarget::Hardening),
    }).collect()
}

pub struct AppState {
    pub pages: Vec<Vec<Message>>,
    pub selected_menu: usize,
    pub logs: Vec<String>,
    pub scroll_offsets: Vec<usize>,
    pub focus: Focus,
    pub following_chat: bool,
    pub logs_scroll_offset: usize,

    pub profiles: Vec<Profile>,
    pub presets: Vec<crate::profile::Preset>,
    pub selected_profile: Option<usize>,
    pub connection_state: ConnectionState,
    pub ssh_rx: Option<mpsc::Receiver<SshEvent>>,
    pub input: String,

    pub deploy_targets: Vec<DeployTargetState>,
    pub deploy_running: bool,
    pub content_cursor: usize,
    pub profile_form: ProfileForm,
}

fn page_welcome(name: &str, desc: &str) -> Vec<Message> {
    vec![
        Message {
            sender: Sender::System,
            content: format!("You are in the {} view.\n\n{}", name, desc),
        },
        Message {
            sender: Sender::Assistant,
            content: format!(
                "Welcome to {}.\n\nType a message or use /help for available commands.",
                name
            ),
        },
    ]
}

fn make_pages() -> Vec<Vec<Message>> {
    vec![
        Vec::new(),
        page_welcome("SETUP AGENT", "Configure and set up a new AI agent."),
        page_welcome("CONNECT VPS", "Manage SSH connections.\nSelect a profile and press Enter to connect."),
        page_welcome("HISTORY", "Browse your previous sessions and commands."),
        page_welcome("CONNECTION", "View active connection details."),
        page_welcome("SESSION", "View and manage current sessions."),
    ]
}

impl Default for AppState {
    fn default() -> Self {
        let profiles = crate::profile::load_profiles().unwrap_or_default();
        let presets = crate::profile::load_presets().unwrap_or_default();

        Self {
            pages: make_pages(),
            selected_menu: 0,
            logs: Vec::new(),
            scroll_offsets: vec![0; MENU_ITEMS.len()],
            focus: Focus::Input,
            following_chat: true,
            logs_scroll_offset: 0,

            profiles,
            presets,
            selected_profile: None,
            connection_state: ConnectionState::Disconnected,
            ssh_rx: None,
            input: String::new(),

            deploy_targets: default_deploy_targets(),
            deploy_running: false,
            content_cursor: 0,
            profile_form: ProfileForm {
                active: false,
                name: String::new(),
                host: String::new(),
                port: String::new(),
                username: String::new(),
                key_path: String::new(),
                cursor: 0,
            },
        }
    }
}

impl AppState {
    pub fn messages(&self) -> &[Message] {
        &self.pages[self.selected_menu]
    }

    #[allow(dead_code)]
    pub fn messages_mut(&mut self) -> &mut Vec<Message> {
        &mut self.pages[self.selected_menu]
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offsets[self.selected_menu]
    }

    pub fn set_scroll_offset(&mut self, v: usize) {
        self.scroll_offsets[self.selected_menu] = v;
    }

    pub fn add_log(&mut self, line: String) {
        self.logs.push(line);
    }

    pub fn add_message(&mut self, msg: Message) {
        self.pages[self.selected_menu].push(msg);
    }

    pub fn refresh_deploy_page(&mut self) {
        let mut msgs = vec![
            Message {
                sender: Sender::System,
                content: "Select deployment targets, then run /deploy to execute.".into(),
            },
            Message {
                sender: Sender::Assistant,
                content: self.deploy_target_list_text(),
            },
        ];
        if self.connection_state != ConnectionState::Connected {
            msgs.push(Message {
                sender: Sender::System,
                content: "Not connected to any VPS. Use CONNECT VPS first.".into(),
            });
        } else if self.deploy_running {
            msgs.push(Message {
                sender: Sender::System,
                content: "Deployment in progress... Check the logs panel for output.".into(),
            });
        }
        self.pages[0] = msgs;
    }

    fn deploy_target_list_text(&self) -> String {
        let mut s = "Available targets:\n".to_string();
        for dt in &self.deploy_targets {
            let check = if dt.selected { "[x]" } else { "[ ]" };
            let label = Pipeline::target_label(&dt.target);
            s.push_str(&format!("  {} {}\n", check, label));
        }
        s.push_str("\nCommands:\n  /targets       - show this list\n  /select <name> - toggle a target by name\n  /deploy        - run all selected targets");
        s
    }

    pub fn toggle_target(&mut self, name: &str) {
        let idx = self.deploy_targets.iter().position(|dt| {
            let label = Pipeline::target_label(&dt.target);
            label.to_lowercase().contains(&name.to_lowercase())
        });
        match idx {
            Some(i) => {
                self.deploy_targets[i].selected = !self.deploy_targets[i].selected;
                let state = self.deploy_targets[i].selected;
                let label = Pipeline::target_label(&self.deploy_targets[i].target);
                self.refresh_deploy_page();
                self.add_message(Message {
                    sender: Sender::System,
                    content: format!("Toggled {} {}", label, if state { "on" } else { "off" }),
                });
            }
            None => {
                self.add_message(Message {
                    sender: Sender::System,
                    content: format!("No target matching '{name}'. Use /targets to list available targets."),
                });
            }
        }
    }

    pub fn start_deployment(&mut self) {
        if self.connection_state != ConnectionState::Connected {
            self.add_message(Message {
                sender: Sender::System,
                content: "Not connected. Use /connect <n> first.".into(),
            });
            return;
        }
        if self.deploy_running {
            self.add_message(Message {
                sender: Sender::System,
                content: "Deployment already in progress.".into(),
            });
            return;
        }
        let profile = match self.selected_profile
            .and_then(|i| self.profiles.get(i))
        {
            Some(p) => p.clone(),
            None => {
                self.add_message(Message {
                    sender: Sender::System,
                    content: "No profile selected. Use /connect <n> first.".into(),
                });
                return;
            }
        };

        let selected: Vec<DeploymentTarget> = self.deploy_targets
            .iter()
            .filter(|dt| dt.selected)
            .map(|dt| dt.target.clone())
            .collect();

        if selected.is_empty() {
            self.add_message(Message {
                sender: Sender::System,
                content: "No targets selected. Use /select <name> to toggle targets.".into(),
            });
            return;
        }

        self.deploy_running = true;
        self.selected_menu = 0;
        self.refresh_deploy_page();

        let (tx, rx) = mpsc::channel();
        self.ssh_rx = Some(rx);

        let names: Vec<String> = selected.iter()
            .map(|t| Pipeline::target_label(t).to_string())
            .collect();
        self.add_message(Message {
            sender: Sender::System,
            content: format!("Starting deployment: {}", names.join(", ")),
        });
        self.add_log("=== DEPLOYMENT STARTED ===".into());

        for target in &selected {
            let label = Pipeline::target_label(target);
            self.add_log(format!(">>> {label}"));
        }

        std::thread::spawn(move || {
            for target in selected {
                let label = Pipeline::target_label(&target);
                let _ = tx.send(SshEvent::Line(format!("\n>>> {label}")));
                match SshClient::exec_script_sync(&profile, Pipeline::script_for(&target), tx.clone()) {
                    Ok(code) => {
                        let _ = tx.send(SshEvent::Line(format!("[ {label} done, exit: {code} ]")));
                    }
                    Err(e) => {
                        let _ = tx.send(SshEvent::Error(format!("[ {label} failed: {e} ]")));
                        break;
                    }
                }
            }
            let _ = tx.send(SshEvent::Done(0));
            let _ = tx.send(SshEvent::Line("=== DEPLOYMENT COMPLETE ===".into()));
        });

        self.focus = Focus::Logs;
    }

    pub fn set_page_messages(&mut self, index: usize, msgs: Vec<Message>) {
        if index < self.pages.len() {
            self.pages[index] = msgs;
        }
    }

    pub fn drain_ssh_events(&mut self) {
        let (events, disconnected) = {
            let rx = match self.ssh_rx.as_ref() {
                Some(rx) => rx,
                None => return,
            };
            let mut events = Vec::new();
            let mut disconnected = false;
            loop {
                match rx.try_recv() {
                    Ok(evt) => events.push(evt),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        disconnected = true;
                        break;
                    }
                }
            }
            (events, disconnected)
        };

        for event in events {
            match event {
                SshEvent::Line(line) => {
                    self.logs.push(line);
                }
                SshEvent::Error(e) => {
                    self.logs.push(format!("ERROR: {e}"));
                    self.connection_state = ConnectionState::Failed;
                    self.pages[self.selected_menu].push(Message {
                        sender: Sender::System,
                        content: format!("SSH error: {e}"),
                    });
                }
                SshEvent::Done(code) => {
                    self.logs.push(format!("[exit code: {code}]"));
                    if self.deploy_running {
                        self.deploy_running = false;
                        self.pages[0].push(Message {
                            sender: Sender::System,
                            content: "Deployment finished.".into(),
                        });
                        self.refresh_deploy_page();
                    } else {
                        self.pages[self.selected_menu].push(Message {
                            sender: Sender::System,
                            content: format!("Command finished with exit code {code}"),
                        });
                    }
                }
                SshEvent::Connected => {
                    self.connection_state = ConnectionState::Connected;
                    self.logs.push("SSH connected".into());
                    self.pages[self.selected_menu].push(Message {
                        sender: Sender::System,
                        content: "Connected to VPS.".into(),
                    });
                    self.pages[self.selected_menu].push(Message {
                        sender: Sender::Assistant,
                        content: "Connection established. You can now run commands.\nType a command or select a target from DEPLOY.".into(),
                    });
                }
            }
        }

        if disconnected {
            self.ssh_rx = None;
        }
    }

    pub fn submit_input(&mut self) {
        let text = self.input.trim().to_string();
        if text.is_empty() {
            return;
        }
        self.input.clear();

        self.add_message(Message {
            sender: Sender::User,
            content: text.clone(),
        });

        if let Some(cmd) = text.strip_prefix('/') {
            self.handle_command(cmd);
        }
    }

    fn handle_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.first() {
            Some(&"help") => {
                self.add_message(Message {
                    sender: Sender::Assistant,
                    content: "Available commands:\n  /help          - show this message\n  /connect <n>   - connect to profile by index\n  /disconnect    - close SSH connection\n  /profiles      - list saved profiles\n  /add-profile   - open profile form\n  /targets       - list deployment targets\n  /select <name> - toggle a deployment target\n  /deploy        - run all selected targets\n  /clear         - clear log panel".into(),
                });
            }
            Some(&"connect") => {
                let idx = parts.get(1).and_then(|s| s.parse::<usize>().ok());
                if let Some(i) = idx {
                    if i < self.profiles.len() {
                        self.connect_to(i);
                    } else {
                        self.add_message(Message {
                            sender: Sender::System,
                            content: format!("Profile {i} not found. Use /profiles to list them."),
                        });
                    }
                } else {
                    self.add_message(Message {
                        sender: Sender::System,
                        content: "Usage: /connect <profile_index>".into(),
                    });
                }
            }
            Some(&"disconnect") => {
                self.connection_state = ConnectionState::Disconnected;
                self.ssh_rx = None;
                self.add_log("Disconnected.".into());
                self.add_message(Message {
                    sender: Sender::System,
                    content: "SSH connection closed.".into(),
                });
            }
            Some(&"profiles") => {
                if self.profiles.is_empty() {
                    self.add_message(Message {
                        sender: Sender::Assistant,
                        content: "No profiles saved. Use /add-profile to create one.".into(),
                    });
                } else {
                    let mut msg = "Saved profiles:\n".to_string();
                    for (i, p) in self.profiles.iter().enumerate() {
                        msg.push_str(&format!("  {}. {}@{}:{}\n", i, p.username, p.host, p.port));
                    }
                    self.add_message(Message {
                        sender: Sender::Assistant,
                        content: msg,
                    });
                }
            }
            Some(&"add-profile") => {
                self.open_profile_form();
            }
            Some(&"save-profile") => {
                let p = Profile {
                    name: String::new(),
                    host: String::new(),
                    port: 22,
                    username: String::from("root"),
                    key_path: format!("{}/.ssh/id_rsa", std::env::var("HOME").unwrap_or_default()),
                };
                self.profiles.push(p);
                let _ = crate::profile::save_profiles(&self.profiles);
                self.add_message(Message {
                    sender: Sender::System,
                    content: "Blank profile saved (fill in details manually in ~/.config/deploykit/profiles.toml)".into(),
                });
            }
            Some(&"run") => {
                self.add_message(Message {
                    sender: Sender::Assistant,
                    content: "Use a deployment target instead (navigate to DEPLOY and select targets).\nFor one-off commands, /connect first then type the command directly.".into(),
                });
            }
            Some(&"clear") => {
                self.logs.clear();
            }
            Some(&"targets") => {
                self.selected_menu = 0;
                self.refresh_deploy_page();
            }
            Some(&"select") => {
                let name = parts.get(1).copied().unwrap_or("");
                self.selected_menu = 0;
                self.toggle_target(name);
            }
            Some(&"deploy") => {
                self.start_deployment();
            }
            _ => {
                self.add_message(Message {
                    sender: Sender::Assistant,
                    content: format!("Unknown command: /{}. Type /help for available commands.", parts.first().unwrap_or(&"")),
                });
            }
        }
    }

    pub fn open_profile_form(&mut self) {
        self.selected_menu = 2;
        self.content_cursor = 0;
        self.profile_form = ProfileForm::open();
        self.focus = Focus::Content;
    }

    pub fn save_profile_form(&mut self) {
        let host = std::mem::take(&mut self.profile_form.host);
        if host.is_empty() {
            return;
        }
        let name = std::mem::take(&mut self.profile_form.name);
        let name = if name.is_empty() {
            format!("{}@{}", self.profile_form.username, host)
        } else {
            name
        };
        let port: u16 = self.profile_form.port.parse().unwrap_or(22);
        let username = std::mem::take(&mut self.profile_form.username);
        let key_path = std::mem::take(&mut self.profile_form.key_path);
        let p = Profile { name, host, port, username, key_path };
        self.profiles.push(p);
        let _ = crate::profile::save_profiles(&self.profiles);
        self.profile_form.active = false;
        self.content_cursor = self.profiles.len().saturating_sub(1);
    }

    pub fn connect_to(&mut self, profile_idx: usize) {
        if profile_idx >= self.profiles.len() {
            return;
        }
        self.selected_profile = Some(profile_idx);
        self.connection_state = ConnectionState::Connecting;
        self.selected_menu = 2;

        let (tx, rx) = mpsc::channel();
        self.ssh_rx = Some(rx);
        let profile = self.profiles[profile_idx].clone();

        self.add_log(format!("Connecting to {}@{}...", profile.username, profile.host));
        self.add_message(Message {
            sender: Sender::System,
            content: format!("Connecting to {}@{}:{}...", profile.username, profile.host, profile.port),
        });

        let _ = crate::ssh::SshClient::connect(&profile, tx);
    }
}
