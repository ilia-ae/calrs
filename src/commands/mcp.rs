//! `calrs mcp`: the SSH forced command of the MCP agent (fork addition, see
//! ilia-booking `docs/mcp.md`). A byte pump between stdio and serve's admin
//! socket, nothing more: the MCP server, the tools, the role check and the
//! limits all live in `calrs serve` (`src/web/agent/`), so there is nothing
//! here to bypass.
//!
//! It runs before calrs's usual startup: it opens no database, reads no key or
//! settings, creates no files, and ignores `SSH_ORIGINAL_COMMAND`. Its stdout
//! carries only what serve sent; its own messages (one line on failure, plus
//! `RUST_LOG` logging) go to stderr, and they never contain the bytes pumped.

use crate::web::agent::MAX_FRAME_BYTES;
use std::io;
use std::path::PathBuf;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::UnixStream;

/// The socket systemd creates for serve (`calrs-admin.socket`).
pub const DEFAULT_SOCKET: &str = "/run/calrs-admin.sock";

/// After stdin closes, how long the pump still passes serve's last answers
/// on before it exits.
const DRAIN_AFTER_EOF: std::time::Duration = std::time::Duration::from_secs(5);

/// How the pump ended.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PumpEnd {
    /// The client closed stdin: a normal end (exit 0).
    ClientClosed,
    /// Serve closed the socket while the client was still connected: refused,
    /// shut down, or timed out (exit 1).
    ServerClosed,
    /// The client sent a line longer than `MAX_FRAME_BYTES` (exit 1).
    LineTooLong,
    /// Reading or writing failed (exit 1).
    Io(io::ErrorKind),
}

/// Connect to the admin socket and pump until either side is done. Never
/// returns: it ends the process, because a read from tokio's stdin cannot be
/// cancelled and would otherwise keep the runtime from shutting down.
pub async fn run(socket: Option<PathBuf>) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_ansi(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "calrs=warn".into()),
        )
        .init();

    let path = socket.unwrap_or_else(|| PathBuf::from(DEFAULT_SOCKET));
    let stream = match UnixStream::connect(&path).await {
        Ok(stream) => stream,
        Err(error) => {
            eprintln!(
                "calrs mcp: cannot connect to {}: {}",
                path.display(),
                error.kind()
            );
            std::process::exit(1);
        }
    };
    let end = pump(tokio::io::stdin(), tokio::io::stdout(), stream).await;
    let code = match &end {
        PumpEnd::ClientClosed => 0,
        PumpEnd::ServerClosed => {
            eprintln!("calrs mcp: the server closed the connection");
            1
        }
        PumpEnd::LineTooLong => {
            eprintln!("calrs mcp: message longer than {MAX_FRAME_BYTES} bytes, closing");
            1
        }
        PumpEnd::Io(kind) => {
            eprintln!("calrs mcp: {kind}");
            1
        }
    };
    tracing::debug!(target: "calrs::mcp", ?end, "pump finished");
    std::process::exit(code)
}

/// Copy `stdin` to the socket and the socket to `stdout` until one side ends.
/// On stdin EOF the socket's write side is shut down and serve's remaining
/// answers are still passed on for a moment; on socket EOF the pump ends at
/// once, whatever stdin is doing.
pub(crate) async fn pump<I, O>(stdin: I, stdout: O, socket: UnixStream) -> PumpEnd
where
    I: AsyncRead + Unpin,
    O: AsyncWrite + Unpin,
{
    let (from_server, to_server) = socket.into_split();
    let upstream = upstream(stdin, to_server);
    let downstream = downstream(from_server, stdout);
    tokio::pin!(upstream, downstream);
    tokio::select! {
        end = &mut upstream => match end {
            PumpEnd::ClientClosed => {
                let _ = tokio::time::timeout(DRAIN_AFTER_EOF, &mut downstream).await;
                PumpEnd::ClientClosed
            }
            other => other,
        },
        end = &mut downstream => end,
    }
}

/// stdin to the socket, refusing lines over `MAX_FRAME_BYTES`: nothing past
/// that limit is forwarded.
async fn upstream<I: AsyncRead + Unpin>(mut stdin: I, mut socket: OwnedWriteHalf) -> PumpEnd {
    let mut buf = vec![0u8; 16 * 1024];
    let mut since_newline = 0usize;
    loop {
        let n = match stdin.read(&mut buf).await {
            Ok(0) => {
                let _ = socket.shutdown().await;
                return PumpEnd::ClientClosed;
            }
            Ok(n) => n,
            Err(error) => return PumpEnd::Io(error.kind()),
        };
        let chunk = &buf[..n];
        let mut allowed = n;
        let mut too_long = false;
        for (i, &byte) in chunk.iter().enumerate() {
            if byte == b'\n' {
                since_newline = 0;
            } else {
                since_newline += 1;
                if since_newline > MAX_FRAME_BYTES {
                    allowed = i;
                    too_long = true;
                    break;
                }
            }
        }
        if let Err(error) = socket.write_all(&chunk[..allowed]).await {
            return PumpEnd::Io(error.kind());
        }
        if too_long {
            return PumpEnd::LineTooLong;
        }
    }
}

/// The socket to stdout, verbatim, flushed after every write.
async fn downstream<O: AsyncWrite + Unpin>(mut socket: OwnedReadHalf, mut stdout: O) -> PumpEnd {
    let mut buf = vec![0u8; 16 * 1024];
    loop {
        let n = match socket.read(&mut buf).await {
            Ok(0) => return PumpEnd::ServerClosed,
            Ok(n) => n,
            Err(error) => return PumpEnd::Io(error.kind()),
        };
        if let Err(error) = stdout.write_all(&buf[..n]).await {
            return PumpEnd::Io(error.kind());
        }
        if let Err(error) = stdout.flush().await {
            return PumpEnd::Io(error.kind());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::io::{duplex, AsyncBufReadExt, BufReader};

    /// Read from `reader` until `n` bytes or EOF.
    async fn read_n<R: AsyncRead + Unpin>(reader: &mut R, n: usize) -> Vec<u8> {
        let mut out = vec![0u8; n];
        let mut filled = 0;
        while filled < n {
            match reader.read(&mut out[filled..]).await.unwrap() {
                0 => break,
                read => filled += read,
            }
        }
        out.truncate(filled);
        out
    }

    #[tokio::test]
    async fn bytes_pass_verbatim_both_ways() {
        let (mut client_in, pump_in) = duplex(1 << 20);
        let (pump_out, mut client_out) = duplex(1 << 20);
        let (pump_socket, server) = UnixStream::pair().unwrap();
        let pumping = tokio::spawn(pump(pump_in, pump_out, pump_socket));
        let (server_read, mut server_write) = server.into_split();
        let mut server_lines = BufReader::new(server_read).lines();

        let request = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\",\"x\":\"é\\n\"}\n";
        client_in.write_all(request.as_bytes()).await.unwrap();
        let received = server_lines.next_line().await.unwrap().unwrap();
        assert_eq!(format!("{received}\n"), request);

        let answer = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}\n\x00\xff";
        server_write.write_all(answer).await.unwrap();
        assert_eq!(read_n(&mut client_out, answer.len()).await, answer);

        drop(client_in);
        assert_eq!(
            server_lines.next_line().await.unwrap(),
            None,
            "EOF passed on"
        );
        drop(server_write);
        assert_eq!(pumping.await.unwrap(), PumpEnd::ClientClosed);
    }

    #[tokio::test]
    async fn stdin_eof_still_passes_the_last_answer() {
        let (mut client_in, pump_in) = duplex(4096);
        let (pump_out, mut client_out) = duplex(4096);
        let (pump_socket, server) = UnixStream::pair().unwrap();
        let pumping = tokio::spawn(pump(pump_in, pump_out, pump_socket));

        client_in.write_all(b"{\"id\":7}\n").await.unwrap();
        drop(client_in);
        let (server_read, mut server_write) = server.into_split();
        let mut received = String::new();
        BufReader::new(server_read)
            .read_line(&mut received)
            .await
            .unwrap();
        assert_eq!(received, "{\"id\":7}\n");
        server_write
            .write_all(b"{\"id\":7,\"result\":{}}\n")
            .await
            .unwrap();
        drop(server_write);
        assert_eq!(pumping.await.unwrap(), PumpEnd::ClientClosed);
        let mut rest = Vec::new();
        client_out.read_to_end(&mut rest).await.unwrap();
        assert_eq!(rest, b"{\"id\":7,\"result\":{}}\n");
    }

    #[tokio::test]
    async fn server_eof_ends_the_pump_while_stdin_stays_open() {
        let (_client_in, pump_in) = duplex(4096);
        let (pump_out, mut client_out) = duplex(4096);
        let (pump_socket, server) = UnixStream::pair().unwrap();
        let pumping = tokio::spawn(pump(pump_in, pump_out, pump_socket));
        drop(server);
        let end = tokio::time::timeout(Duration::from_secs(2), pumping)
            .await
            .expect("the pump ends promptly")
            .unwrap();
        assert_eq!(end, PumpEnd::ServerClosed);
        let mut rest = Vec::new();
        client_out.read_to_end(&mut rest).await.unwrap();
        assert!(rest.is_empty(), "nothing invented on stdout");
    }

    #[tokio::test]
    async fn an_overlong_line_is_cut_at_the_cap() {
        let (mut client_in, pump_in) = duplex(1 << 20);
        let (pump_out, _client_out) = duplex(4096);
        let (pump_socket, mut server) = UnixStream::pair().unwrap();
        let pumping = tokio::spawn(pump(pump_in, pump_out, pump_socket));

        let writer = tokio::spawn(async move {
            client_in.write_all(b"{\"ok\":1}\n").await.unwrap();
            let long = vec![b'a'; MAX_FRAME_BYTES + 4096];
            let _ = client_in.write_all(&long).await;
            client_in
        });
        // Read while the pump writes: the socket buffer is smaller than a frame.
        let reader = tokio::spawn(async move {
            let mut forwarded = Vec::new();
            server.read_to_end(&mut forwarded).await.unwrap();
            forwarded
        });
        assert_eq!(pumping.await.unwrap(), PumpEnd::LineTooLong);
        let forwarded = reader.await.unwrap();
        assert!(forwarded.starts_with(b"{\"ok\":1}\n"));
        let tail = forwarded.len() - b"{\"ok\":1}\n".len();
        assert!(
            tail <= MAX_FRAME_BYTES,
            "forwarded {tail} bytes of the long line"
        );
        drop(writer.await.unwrap());
    }

    #[tokio::test]
    async fn a_line_at_the_cap_passes() {
        let (mut client_in, pump_in) = duplex(1 << 20);
        let (pump_out, _client_out) = duplex(4096);
        let (pump_socket, server) = UnixStream::pair().unwrap();
        let pumping = tokio::spawn(pump(pump_in, pump_out, pump_socket));
        let mut line = vec![b'b'; MAX_FRAME_BYTES];
        line.push(b'\n');
        let sent = line.clone();
        let writer = tokio::spawn(async move {
            client_in.write_all(&sent).await.unwrap();
            client_in
        });
        let (mut server_read, server_write) = server.into_split();
        assert_eq!(read_n(&mut server_read, line.len()).await, line);
        drop(writer.await.unwrap());
        drop(server_write);
        assert_eq!(pumping.await.unwrap(), PumpEnd::ClientClosed);
    }
}
