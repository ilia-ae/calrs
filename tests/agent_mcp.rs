//! The MCP agent from the outside (fork addition): the `calrs mcp` binary as
//! sshd runs it, with a cleared environment, and an end-to-end session through
//! it to a real `calrs serve`.

use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_calrs");
const OWNER: &str = "owner@example.test";

/// A directory under the system temp dir, removed when dropped. Names stay
/// short: a Unix socket path must fit in 108 bytes.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let id = uuid::Uuid::new_v4().simple().to_string();
        let path = std::env::temp_dir().join(format!("calrs-mcp-{}", &id[..12]));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn dir(&self, name: &str) -> PathBuf {
        let path = self.0.join(name);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn file(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Every file and directory below `dir`.
fn entries_under(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            found.extend(entries_under(&path));
        }
        found.push(path);
    }
    found
}

/// `calrs mcp --socket <socket>` with nothing in its environment but HOME
/// and a trace-level RUST_LOG, run from `cwd`.
fn start_pump(socket: &Path, home: &Path, cwd: &Path) -> Child {
    Command::new(BIN)
        .arg("mcp")
        .arg("--socket")
        .arg(socket)
        .env_clear()
        .env("HOME", home)
        .env("RUST_LOG", "trace")
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

/// The lines of a child's output, read on a thread; the channel closes at EOF.
fn lines_of(output: impl Read + Send + 'static) -> Receiver<String> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(output).lines() {
            let Ok(line) = line else { break };
            if tx.send(line).is_err() {
                break;
            }
        }
    });
    rx
}

fn send(stdin: &mut ChildStdin, message: Value) {
    writeln!(stdin, "{message}").unwrap();
    stdin.flush().unwrap();
}

/// The next stdout line as JSON-RPC 2.0, or `None` at EOF.
fn receive(stdout: &Receiver<String>) -> Option<Value> {
    match stdout.recv_timeout(Duration::from_secs(30)) {
        Ok(line) => {
            let message: Value = serde_json::from_str(&line)
                .unwrap_or_else(|e| panic!("stdout line is not JSON ({e}): {line}"));
            assert_eq!(message["jsonrpc"], "2.0", "{line}");
            Some(message)
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => None,
        Err(mpsc::RecvTimeoutError::Timeout) => panic!("no answer on stdout in time"),
    }
}

fn wait_for_exit(child: &mut Child, limit: Duration) -> ExitStatus {
    let deadline = Instant::now() + limit;
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!("the process did not exit within {limit:?}");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn read_all(output: Option<impl Read>) -> String {
    let mut text = String::new();
    output.unwrap().read_to_string(&mut text).unwrap();
    text
}

#[test]
fn pump_passes_json_rpc_and_leaves_no_files() {
    let tmp = TempDir::new();
    let (home, cwd, socket) = (tmp.dir("home"), tmp.dir("cwd"), tmp.file("s"));
    let listener = UnixListener::bind(&socket).unwrap();
    let server = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let request: Value = serde_json::from_str(&line).unwrap();
        let mut writer = stream;
        writeln!(
            writer,
            "{}",
            json!({"jsonrpc": "2.0", "id": request["id"], "result": {}})
        )
        .unwrap();
        let mut rest = Vec::new();
        reader.read_to_end(&mut rest).unwrap();
        (request, rest)
    });

    let mut pump = start_pump(&socket, &home, &cwd);
    let stdout = lines_of(pump.stdout.take().unwrap());
    let mut stdin = pump.stdin.take().unwrap();
    send(
        &mut stdin,
        json!({"jsonrpc": "2.0", "id": 1, "method": "ping"}),
    );
    let answer = receive(&stdout).expect("the answer is passed on");
    assert_eq!(answer, json!({"jsonrpc": "2.0", "id": 1, "result": {}}));

    drop(stdin);
    let status = wait_for_exit(&mut pump, Duration::from_secs(10));
    assert!(status.success(), "{status:?}");
    while receive(&stdout).is_some() {}

    let (request, rest) = server.join().unwrap();
    assert_eq!(request["method"], "ping");
    assert!(rest.is_empty());
    assert_eq!(entries_under(&home), Vec::<PathBuf>::new());
    assert_eq!(entries_under(&cwd), Vec::<PathBuf>::new());
}

#[test]
fn pump_without_a_socket_fails_on_stderr_only() {
    let tmp = TempDir::new();
    let (home, cwd) = (tmp.dir("home"), tmp.dir("cwd"));
    let mut pump = start_pump(&tmp.file("missing"), &home, &cwd);
    drop(pump.stdin.take());
    let status = wait_for_exit(&mut pump, Duration::from_secs(10));
    assert!(!status.success());
    assert_eq!(read_all(pump.stdout.take()), "");
    let stderr = read_all(pump.stderr.take());
    assert!(stderr.contains("cannot connect"), "{stderr}");
    assert_eq!(entries_under(&home), Vec::<PathBuf>::new());
    assert_eq!(entries_under(&cwd), Vec::<PathBuf>::new());
}

#[test]
fn pump_exits_when_the_server_closes_while_stdin_is_open() {
    let tmp = TempDir::new();
    let (home, cwd, socket) = (tmp.dir("home"), tmp.dir("cwd"), tmp.file("s"));
    let listener = UnixListener::bind(&socket).unwrap();
    let server = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        drop(stream);
    });
    let mut pump = start_pump(&socket, &home, &cwd);
    let _stdin = pump.stdin.take().unwrap();
    server.join().unwrap();
    let status = wait_for_exit(&mut pump, Duration::from_secs(2));
    assert!(!status.success());
    assert_eq!(read_all(pump.stdout.take()), "");
}

// --- End to end with `calrs serve` ---

/// The name of the user running the tests, from /etc/passwd.
fn own_user_name(tmp: &TempDir) -> String {
    let uid = std::fs::metadata(&tmp.0).unwrap().uid();
    let passwd = std::fs::read_to_string("/etc/passwd").unwrap();
    passwd
        .lines()
        .find_map(|line| {
            let fields: Vec<&str> = line.split(':').collect();
            (fields.len() > 2 && fields[2] == uid.to_string()).then(|| fields[0].to_string())
        })
        .unwrap_or_else(|| panic!("uid {uid} is not in /etc/passwd"))
}

/// A running `calrs serve`, killed when dropped.
struct Serve {
    child: Child,
    log: PathBuf,
}

impl Serve {
    fn start(tmp: &TempDir, data: &Path, socket: &Path, users: &str) -> Self {
        let _ = std::fs::remove_file(socket);
        let log = tmp.file(&format!("serve-{}.log", uuid::Uuid::new_v4().simple()));
        let out = std::fs::File::create(&log).unwrap();
        let child = Command::new(BIN)
            .args(["serve", "--port", "0"])
            .env_clear()
            .env("HOME", data)
            .env("CALRS_DATA_DIR", data)
            .env("CALRS_AGENT_SOCKET", socket)
            .env("CALRS_AGENT_USERS", users)
            .env("CALRS_AGENT_ADMIN_EMAIL", OWNER)
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .stdin(Stdio::null())
            .stdout(out.try_clone().unwrap())
            .stderr(out)
            .spawn()
            .unwrap();
        let mut serve = Self { child, log };
        let deadline = Instant::now() + Duration::from_secs(60);
        while !socket.exists() {
            if let Some(status) = serve.child.try_wait().unwrap() {
                panic!("serve exited ({status:?}):\n{}", serve.log());
            }
            assert!(
                Instant::now() < deadline,
                "no admin socket:\n{}",
                serve.log()
            );
            std::thread::sleep(Duration::from_millis(50));
        }
        serve
    }

    fn log(&self) -> String {
        std::fs::read_to_string(&self.log).unwrap_or_default()
    }
}

impl Drop for Serve {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn pump_and_serve_run_a_read_only_session_end_to_end() {
    let tmp = TempDir::new();
    let (data, home, socket) = (tmp.dir("data"), tmp.dir("home"), tmp.file("agent.sock"));
    let mut create = Command::new(BIN)
        .args(["user", "create", "--email", OWNER, "--name", "Owner"])
        .args(["--admin", "--password-stdin"])
        .env_clear()
        .env("HOME", &data)
        .env("CALRS_DATA_DIR", &data)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    create
        .stdin
        .take()
        .unwrap()
        .write_all(b"correct horse battery staple\n")
        .unwrap();
    let created = create.wait_with_output().unwrap();
    assert!(
        created.status.success(),
        "{}{}",
        String::from_utf8_lossy(&created.stdout),
        String::from_utf8_lossy(&created.stderr)
    );

    let me = own_user_name(&tmp);
    let serve = Serve::start(&tmp, &data, &socket, &format!("{me}=read"));
    let mut pump = start_pump(&socket, &home, &home);
    let stdout = lines_of(pump.stdout.take().unwrap());
    let mut stdin = pump.stdin.take().unwrap();

    send(
        &mut stdin,
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "calrs-e2e", "version": "0"}
        }}),
    );
    let init = receive(&stdout).unwrap_or_else(|| panic!("no answer:\n{}", serve.log()));
    assert_eq!(init["id"], 1, "{init}");
    assert_eq!(init["result"]["serverInfo"]["name"], "calrs");
    let instructions = init["result"]["instructions"].as_str().unwrap();
    assert!(instructions.contains("This connection is read_only."));
    send(
        &mut stdin,
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
    );

    send(
        &mut stdin,
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
    );
    let list = receive(&stdout).unwrap();
    assert_eq!(list["id"], 2, "{list}");
    let names: BTreeSet<&str> = list["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    let read_tools: BTreeSet<&str> = [
        "whoami",
        "get_overview",
        "get_settings",
        "list_event_types",
        "get_event_type",
        "get_slots",
        "list_sources",
        "list_bookings",
    ]
    .into_iter()
    .collect();
    assert_eq!(names, read_tools);

    send(
        &mut stdin,
        json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call",
               "params": {"name": "whoami", "arguments": {}}}),
    );
    let whoami = receive(&stdout).unwrap();
    assert_eq!(whoami["id"], 3, "{whoami}");
    let me_out = &whoami["result"]["structuredContent"];
    assert_eq!(me_out["connection"]["role"], "read_only", "{whoami}");
    assert_eq!(me_out["acting_user"]["email"], OWNER);

    drop(stdin);
    let status = wait_for_exit(&mut pump, Duration::from_secs(15));
    assert!(status.success(), "{status:?}");
    let log = serve.log();
    assert!(log.contains("agent call"), "the call is audited:\n{log}");
    assert!(!log.contains("correct horse"), "{log}");
    drop(serve);

    // The same uid, now not in the role map: closed unread, nothing on stdout.
    let serve = Serve::start(&tmp, &data, &socket, "nobody=read");
    let mut pump = start_pump(&socket, &home, &home);
    let stdout = lines_of(pump.stdout.take().unwrap());
    let mut stdin = pump.stdin.take().unwrap();
    // The pump may already be gone, so a failed write is fine here.
    let _ = writeln!(stdin, r#"{{"jsonrpc":"2.0","id":1,"method":"ping"}}"#);
    assert!(receive(&stdout).is_none(), "no answer for an unknown uid");
    let status = wait_for_exit(&mut pump, Duration::from_secs(10));
    assert!(!status.success());
    assert!(serve.log().contains("unknown_uid"), "{}", serve.log());
}
