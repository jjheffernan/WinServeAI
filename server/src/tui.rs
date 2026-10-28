//! Terminal monitor. Strata's launch TUI is the reference; this is the
//! slice that drives `ServerManager` (status, start/stop, URL, logs).
//!
//! `1` overview, `2` logs, `3` the URL to paste into a client.
//! `s` start, `x` stop, `r` restart, `q` quit.

use std::io::{stdout, Write};
use std::path::PathBuf;
use std::time::Duration;

use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use tokio::sync::{mpsc, oneshot};

use crate::ipc::pipe::{self, Response};
use crate::server::logs;
use crate::server::resident::{Command, CommandKind, Reply};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Start,
    Stop,
    Restart,
    Quit,
    View(u8),
    LogNext,
}

pub fn action_for(key: KeyEvent) -> Option<Action> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Some(Action::Quit);
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),
        KeyCode::Char('s') => Some(Action::Start),
        KeyCode::Char('x') => Some(Action::Stop),
        KeyCode::Char('r') => Some(Action::Restart),
        KeyCode::Char('1') => Some(Action::View(0)),
        KeyCode::Char('2') => Some(Action::View(1)),
        KeyCode::Char('3') => Some(Action::View(2)),
        KeyCode::Char('l') | KeyCode::Tab => Some(Action::LogNext),
        _ => None,
    }
}

#[derive(Clone)]
pub struct ScreenInfo {
    pub model: String,
    pub context: u32,
    pub gpu_layers: String,
    pub log_dir: PathBuf,
    /// This process owns the manager. Quit stops llama-server.
    pub owns: bool,
}

enum Control {
    Local(mpsc::Sender<Command>),
    Remote(String),
}

struct Snap {
    status: String,
    endpoint: String,
    error: String,
}

impl Control {
    async fn call(&self, kind: CommandKind) -> Result<Snap, String> {
        match self {
            Self::Local(tx) => {
                let (cmd, ack) = Command::new(kind);
                tx.send(cmd).await.map_err(|_| "owner closed".to_string())?;
                reply_snap(ack).await
            }
            Self::Remote(pipe) => {
                let resp = pipe::request(pipe, kind_name(kind))
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(response_snap(resp))
            }
        }
    }
}

fn kind_name(kind: CommandKind) -> &'static str {
    match kind {
        CommandKind::Status => "status",
        CommandKind::Start => "start",
        CommandKind::Stop => "stop",
        CommandKind::Restart => "restart",
        CommandKind::Health => "health",
        CommandKind::Endpoint => "endpoint",
        CommandKind::Show => "show",
    }
}

async fn reply_snap(ack: oneshot::Receiver<Reply>) -> Result<Snap, String> {
    let reply = ack.await.map_err(|_| "no reply".to_string())?;
    Ok(Snap {
        status: reply.status.as_str().to_string(),
        endpoint: reply.endpoint,
        error: reply.error.unwrap_or_default(),
    })
}

fn response_snap(resp: Response) -> Snap {
    Snap {
        status: resp.status,
        endpoint: resp.endpoint,
        error: resp.error.unwrap_or_default(),
    }
}

struct RawGuard;

impl Drop for RawGuard {
    fn drop(&mut self) {
        let mut out = stdout();
        let _ = disable_raw_mode();
        let _ = execute!(out, LeaveAlternateScreen, Show);
        let _ = out.flush();
    }
}

const LOGS: [&str; 3] = ["server", "llama", "error"];

/// Run the monitor until quit. When `owns` is set, quit sends stop first.
pub async fn run(local: Option<mpsc::Sender<Command>>, pipe: Option<String>, info: ScreenInfo) -> Result<(), String> {
    let control = match (local, pipe) {
        (Some(tx), _) => Control::Local(tx),
        (None, Some(pipe)) => Control::Remote(pipe),
        (None, None) => return Err("tui has no manager".into()),
    };

    enable_raw_mode().map_err(|e| e.to_string())?;
    execute!(stdout(), EnterAlternateScreen, Hide).map_err(|e| e.to_string())?;
    let _raw = RawGuard;

    if info.owns && !info.model.trim().is_empty() {
        let _ = control.call(CommandKind::Start).await;
    }

    let mut view: u8 = 0;
    let mut log_i: usize = 0;
    let mut snap = control.call(CommandKind::Status).await.unwrap_or(Snap {
        status: "Unknown".into(),
        endpoint: String::new(),
        error: String::new(),
    });

    loop {
        draw(view, log_i, &info, &snap)?;
        if !event::poll(Duration::from_millis(200)).map_err(|e| e.to_string())? {
            if let Ok(next) = control.call(CommandKind::Status).await {
                snap = next;
            }
            continue;
        }
        let ev = event::read().map_err(|e| e.to_string())?;
        let Event::Key(key) = ev else {
            continue;
        };
        match action_for(key) {
            Some(Action::Quit) => {
                if info.owns {
                    let _ = control.call(CommandKind::Stop).await;
                }
                return Ok(());
            }
            Some(Action::Start) => snap = control.call(CommandKind::Start).await?,
            Some(Action::Stop) => snap = control.call(CommandKind::Stop).await?,
            Some(Action::Restart) => snap = control.call(CommandKind::Restart).await?,
            Some(Action::View(n)) => view = n,
            Some(Action::LogNext) => {
                view = 1;
                log_i = (log_i + 1) % LOGS.len();
            }
            None => {}
        }
    }
}

fn draw(view: u8, log_i: usize, info: &ScreenInfo, snap: &Snap) -> Result<(), String> {
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let cols = cols as usize;
    let mut out = stdout();
    execute!(out, crossterm::cursor::MoveTo(0, 0), crossterm::terminal::Clear(crossterm::terminal::ClearType::All))
        .map_err(|e| e.to_string())?;

    let quit = if info.owns {
        "q stops the server"
    } else {
        "q leaves the server running"
    };
    let mut lines = vec![
        "WinServeAI".to_string(),
        format!("status {}   {}", snap.status, snap.endpoint),
        String::new(),
    ];
    match view {
        0 => {
            lines.push(format!("model    {}", info.model));
            lines.push(format!("context  {}", info.context));
            lines.push(format!("gpu      {}", info.gpu_layers));
            if info.model.trim().is_empty() {
                lines.push("set model.path in config, or use the tray Browse, then press s".into());
            }
        }
        1 => {
            lines.push(format!("log {}   (l cycles)", LOGS[log_i]));
            let tail = logs::tail_dir(&info.log_dir, 40).unwrap_or_default();
            let body = match log_i {
                0 => tail.server,
                1 => tail.llama,
                _ => tail.error,
            };
            if body.is_empty() {
                lines.push("(no lines yet)".into());
            } else {
                lines.extend(body);
            }
        }
        _ => {
            lines.push("Paste this URL into Cursor or any OpenAI client.".into());
            lines.push(snap.endpoint.clone());
        }
    }
    if !snap.error.is_empty() {
        lines.push(String::new());
        lines.push(snap.error.clone());
    }
    lines.push(String::new());
    lines.push(format!("1 overview  2 logs  3 url    s start  x stop  r restart    {quit}"));

    let max_rows = rows.saturating_sub(1) as usize;
    for line in lines.into_iter().take(max_rows) {
        let mut text = line.replace('\n', " ");
        if text.chars().count() > cols {
            text = text.chars().take(cols.saturating_sub(1)).collect();
        }
        writeln!(out, "{text}\r").map_err(|e| e.to_string())?;
    }
    out.flush().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn monitor_keys() {
        assert_eq!(action_for(press(KeyCode::Char('s'))), Some(Action::Start));
        assert_eq!(action_for(press(KeyCode::Char('x'))), Some(Action::Stop));
        assert_eq!(action_for(press(KeyCode::Char('q'))), Some(Action::Quit));
        assert_eq!(action_for(press(KeyCode::Char('2'))), Some(Action::View(1)));
        assert_eq!(action_for(press(KeyCode::Char('l'))), Some(Action::LogNext));
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(action_for(ctrl_c), Some(Action::Quit));
    }
}
