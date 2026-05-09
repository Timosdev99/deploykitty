use std::sync::mpsc;

use crate::profile::Profile;
use crate::ssh::SshEvent;

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
        page_welcome("DEPLOY", "Deploy your application to a remote server.\nSelect a preset or configure individual targets."),
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
                    self.pages[self.selected_menu].push(Message {
                        sender: Sender::System,
                        content: format!("Command finished with exit code {code}"),
                    });
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
                    content: "Available commands:\n  /help        - show this message\n  /connect <n>  - connect to profile by index\n  /disconnect   - close SSH connection\n  /profiles     - list saved profiles\n  /add-profile  - add a new profile (interactive form)\n  /run <cmd>    - run a shell command over SSH\n  /clear        - clear the log panel".into(),
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
                self.selected_menu = 2; // CONNECT VPS
                let msg = "Add a profile by filling in these fields with /set:\n  /set name <value>\n  /set host <value>\n  /set port <value>\n  /set user <value>\n  /set key <path>\nThen run /save-profile to store it.".into();
                self.add_message(Message {
                    sender: Sender::Assistant,
                    content: msg,
                });
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
            _ => {
                self.add_message(Message {
                    sender: Sender::Assistant,
                    content: format!("Unknown command: /{}. Type /help for available commands.", parts.first().unwrap_or(&"")),
                });
            }
        }
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
