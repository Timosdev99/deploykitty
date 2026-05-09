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

#[derive(PartialEq)]
pub enum Focus {
    Sidebar,
    Logs,
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
pub const VERSION: &str = "0.1.0";

pub struct AppState {
    pub pages: Vec<Vec<Message>>,
    pub selected_menu: usize,
    pub logs: Vec<String>,
    pub scroll_offsets: Vec<usize>,
    pub focus: Focus,
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
        page_welcome("DEPLOY", "Deploy your application to a remote server."),
        page_welcome("SETUP AGENT", "Configure and set up a new agent."),
        page_welcome("CONNECT VPS", "Connect to a remote VPS over SSH."),
        page_welcome("HISTORY", "Browse your previous sessions and commands."),
        page_welcome("CONNECTION", "Manage active connections."),
        page_welcome("SESSION", "View and manage current sessions."),
    ]
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            pages: make_pages(),
            selected_menu: 0,
            logs: Vec::new(),
            scroll_offsets: vec![0; MENU_ITEMS.len()],
            focus: Focus::Logs,
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
}
