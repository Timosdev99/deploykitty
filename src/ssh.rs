use std::io::{BufRead, BufReader};
use std::sync::mpsc;
use std::thread;

use color_eyre::eyre::{Result, eyre};
use ssh2::Session;

use crate::profile::Profile;

pub enum SshEvent {
    Line(String),
    Error(String),
    Done(i32),
    Connected,
}

pub struct SshClient;

impl SshClient {
    pub fn connect(profile: &Profile, tx: mpsc::Sender<SshEvent>) -> Result<()> {
        let host = profile.host.clone();
        let port = profile.port;
        let username = profile.username.clone();
        let key_path = profile.key_path.clone();

        thread::spawn(move || {
            if let Err(e) = Self::run_session(host, port, username, key_path, tx.clone()) {
                let _ = tx.send(SshEvent::Error(format!("SSH error: {e}")));
            }
        });

        Ok(())
    }

    fn run_session(
        host: String,
        port: u16,
        username: String,
        key_path: String,
        tx: mpsc::Sender<SshEvent>,
    ) -> Result<()> {
        let addr = format!("{host}:{port}");
        let tcp = std::net::TcpStream::connect(&addr)
            .map_err(|e| eyre!("failed to connect to {addr}: {e}"))?;

        let mut session = Session::new()
            .map_err(|e| eyre!("failed to create SSH session: {e}"))?;
        session.set_tcp_stream(tcp);
        session
            .handshake()
            .map_err(|e| eyre!("SSH handshake failed: {e}"))?;

        session
            .userauth_pubkey_file(&username, None, std::path::Path::new(&key_path), None)
            .map_err(|e| eyre!("authentication failed for {username} @ {host}: {e}"))?;

        if !session.authenticated() {
            return Err(eyre!("authentication failed (not authenticated)"));
        }

        let _ = tx.send(SshEvent::Connected);
        Ok(())
    }

    pub fn exec(
        session: &mut Session,
        cmd: &str,
        tx: &mpsc::Sender<SshEvent>,
    ) -> Result<i32> {
        let mut channel = session
            .channel_session()
            .map_err(|e| eyre!("failed to open channel: {e}"))?;

        channel
            .exec(cmd)
            .map_err(|e| eyre!("failed to exec '{cmd}': {e}"))?;

        let mut reader = BufReader::new(&mut channel);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let trimmed = line.trim_end().to_string();
                    let _ = tx.send(SshEvent::Line(trimmed));
                }
                Err(e) => {
                    let _ = tx.send(SshEvent::Error(format!("read error: {e}")));
                    break;
                }
            }
        }

        channel.wait_close()?;
        let exit_code = channel.exit_status()?;
        Ok(exit_code)
    }

    pub fn exec_script(
        profile: &Profile,
        script: &str,
        tx: mpsc::Sender<SshEvent>,
    ) -> Result<()> {
        let p = profile.clone();
        let s = script.to_string();
        thread::spawn(move || {
            match Self::exec_script_sync(&p, &s, tx.clone()) {
                Ok(code) => { let _ = tx.send(SshEvent::Done(code)); }
                Err(e) => { let _ = tx.send(SshEvent::Error(format!("script error: {e}"))); }
            }
        });
        Ok(())
    }

    pub fn exec_script_sync(
        profile: &Profile,
        script: &str,
        tx: mpsc::Sender<SshEvent>,
    ) -> Result<i32> {
        let addr = format!("{}:{}", profile.host, profile.port);
        let tcp = std::net::TcpStream::connect(&addr)
            .map_err(|e| eyre!("failed to connect to {addr}: {e}"))?;

        let mut session = Session::new()
            .map_err(|e| eyre!("failed to create SSH session: {e}"))?;
        session.set_tcp_stream(tcp);
        session
            .handshake()
            .map_err(|e| eyre!("SSH handshake failed: {e}"))?;

        session
            .userauth_pubkey_file(
                &profile.username,
                None,
                std::path::Path::new(&profile.key_path),
                None,
            )
            .map_err(|e| eyre!("authentication failed: {e}"))?;

        if !session.authenticated() {
            return Err(eyre!("authentication failed (not authenticated)"));
        }

        let _ = tx.send(SshEvent::Connected);

        let exit_code = Self::exec(&mut session, script, &tx)?;
        Ok(exit_code)
    }
}
