//! A scripted, simulated agent session used to demo the dashboard.
//!
//! It emits the exact same [`AgentEvent`] stream the real agent loop would, so
//! the dashboard code never knows (or cares) that the work is fake. The pacing
//! is deliberately tuned to look good on screen-capture: stages light up, files
//! change color as they're touched, and the diff fills in character-by-character.

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use super::events::{AgentEvent, DiffKind, FileStatus, LogEntry};

/// Spawn the simulated agent on a background thread and return the event stream.
pub fn spawn() -> Receiver<AgentEvent> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        // If the UI is closed early the send fails; just stop quietly.
        let _ = script(&tx);
    });
    rx
}

fn log(tx: &Sender<AgentEvent>, stage: &str, detail: &str) -> Result<(), ()> {
    send(
        tx,
        AgentEvent::Log(LogEntry {
            stage: stage.to_string(),
            detail: detail.to_string(),
        }),
    )
}

fn send(tx: &Sender<AgentEvent>, event: AgentEvent) -> Result<(), ()> {
    tx.send(event).map_err(|_| ())
}

fn pause(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

/// Stream one full edit line into the diff, character-by-character.
fn type_line(tx: &Sender<AgentEvent>, kind: DiffKind, text: &str) -> Result<(), ()> {
    send(tx, AgentEvent::DiffNewLine(kind))?;
    // Removed lines and context appear instantly; added lines "type" in.
    if matches!(kind, DiffKind::Added) {
        for ch in text.chars() {
            send(tx, AgentEvent::DiffPush(ch))?;
            pause(12);
        }
    } else {
        for ch in text.chars() {
            send(tx, AgentEvent::DiffPush(ch))?;
        }
        pause(20);
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn script(tx: &Sender<AgentEvent>) -> Result<(), ()> {
    let files = [
        "src/main.rs",
        "src/auth/mod.rs",
        "src/auth/token.rs",
        "src/server/handler.rs",
        "src/server/router.rs",
        "src/config.rs",
        "tests/auth_test.rs",
    ];

    send(tx, AgentEvent::Status("Planning: refactor token validation".into()))?;
    for f in files {
        send(
            tx,
            AgentEvent::File {
                path: f.to_string(),
                status: FileStatus::Pending,
            },
        )?;
    }
    log(tx, "Plan", "Refactor expired-token handling in the auth layer")?;
    pause(500);

    // Scan phase ---------------------------------------------------------
    send(tx, AgentEvent::Status("Analyzing workspace".into()))?;
    log(tx, "Analyzing AST", "Parsing crate module graph")?;
    send(tx, AgentEvent::Progress(8))?;
    pause(350);

    for (i, f) in files.iter().enumerate() {
        send(
            tx,
            AgentEvent::File {
                path: (*f).to_string(),
                status: FileStatus::Scanning,
            },
        )?;
        log(tx, "Running Grep", &format!("`validate_token` in {f}"))?;
        pause(220);
        send(
            tx,
            AgentEvent::File {
                path: (*f).to_string(),
                status: FileStatus::Read,
            },
        )?;
        let scanned = u16::try_from(i + 1).unwrap_or(u16::MAX);
        send(tx, AgentEvent::Progress(8 + scanned * 5))?;
    }

    log(tx, "Reasoning", "Token expiry checked in 2 places; unify it")?;
    pause(400);

    // Edit phase: src/auth/token.rs -------------------------------------
    send(tx, AgentEvent::Status("Editing src/auth/token.rs".into()))?;
    send(
        tx,
        AgentEvent::File {
            path: "src/auth/token.rs".into(),
            status: FileStatus::Modified,
        },
    )?;
    send(tx, AgentEvent::Progress(58))?;
    log(tx, "Generating Diff", "src/auth/token.rs")?;
    send(
        tx,
        AgentEvent::DiffBegin {
            file: "src/auth/token.rs".into(),
            language: "rust".into(),
        },
    )?;
    pause(250);

    type_line(tx, DiffKind::Hunk, "@@ -14,9 +14,14 @@ impl Token {")?;
    type_line(tx, DiffKind::Context, "pub fn validate(&self, now: SystemTime) -> Result<Claims, AuthError> {")?;
    type_line(tx, DiffKind::Removed, "let expired = now > self.expires_at;")?;
    type_line(tx, DiffKind::Removed, "if expired {")?;
    type_line(tx, DiffKind::Removed, "    return Err(AuthError::Expired);")?;
    type_line(tx, DiffKind::Removed, "}")?;
    type_line(tx, DiffKind::Added, "if self.is_expired(now) {")?;
    type_line(tx, DiffKind::Added, "    return Err(AuthError::Expired);")?;
    type_line(tx, DiffKind::Added, "}")?;
    type_line(tx, DiffKind::Context, "    let claims = self.decode_claims()?;")?;
    type_line(tx, DiffKind::Added, "    // Centralized expiry keeps refresh logic in one place.")?;
    type_line(tx, DiffKind::Context, "    Ok(claims)")?;
    type_line(tx, DiffKind::Context, "}")?;
    type_line(tx, DiffKind::Added, "")?;
    type_line(tx, DiffKind::Added, "fn is_expired(&self, now: SystemTime) -> bool {")?;
    type_line(tx, DiffKind::Added, "    now > self.expires_at")?;
    type_line(tx, DiffKind::Added, "}")?;
    pause(400);

    // Edit phase: src/server/handler.rs ---------------------------------
    send(tx, AgentEvent::Progress(82))?;
    send(tx, AgentEvent::Status("Editing src/server/handler.rs".into()))?;
    send(
        tx,
        AgentEvent::File {
            path: "src/server/handler.rs".into(),
            status: FileStatus::Modified,
        },
    )?;
    log(tx, "Generating Diff", "src/server/handler.rs")?;
    send(
        tx,
        AgentEvent::DiffBegin {
            file: "src/server/handler.rs".into(),
            language: "rust".into(),
        },
    )?;
    pause(250);

    type_line(tx, DiffKind::Hunk, "@@ -42,8 +42,7 @@ async fn handle(req: Request) -> Response {")?;
    type_line(tx, DiffKind::Context, "    let token = bearer(&req)?;")?;
    type_line(tx, DiffKind::Removed, "let now = SystemTime::now();")?;
    type_line(tx, DiffKind::Removed, "if now > token.expires_at {")?;
    type_line(tx, DiffKind::Removed, "    return unauthorized();")?;
    type_line(tx, DiffKind::Removed, "}")?;
    type_line(tx, DiffKind::Added, "let claims = token.validate(SystemTime::now())")?;
    type_line(tx, DiffKind::Added, "    .map_err(|_| unauthorized())?;")?;
    type_line(tx, DiffKind::Context, "    dispatch(claims, req).await")?;
    type_line(tx, DiffKind::Context, "}")?;
    pause(400);

    // Verify -------------------------------------------------------------
    send(tx, AgentEvent::Status("Verifying changes".into()))?;
    log(tx, "Running Tests", "cargo test -p auth")?;
    pause(700);
    log(tx, "Tests", "7 passed, 0 failed")?;
    send(tx, AgentEvent::Progress(100))?;
    send(
        tx,
        AgentEvent::Status("Done — 2 files changed, tests green".into()),
    )?;
    log(tx, "Done", "Unified token expiry into Token::is_expired")?;
    send(tx, AgentEvent::Done)?;
    Ok(())
}
