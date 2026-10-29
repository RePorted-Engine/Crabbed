//! The phone's half of a Wi-Fi import: one page, one upload, no server.
//!
//! A thread accepts connections while the import screen is up and nothing else does: the listener
//! is created when a game with no files needs someone to bring them, and its port is released when
//! the files arrive. Six digits the phone shows and the sender types are what stand between a
//! stranger on the same network and writing 589 MB into the app's storage, so the code is never put
//! in the page — a person reads it off the device in their hand.
//!
//! This is deliberately smaller than a web server, and does not reuse `horizon-server`'s: that is a
//! sibling binary crate the client may not depend on, and its 1 MiB body limit is exactly the wrong
//! number here. What is needed is a request line, four headers, and a body copied to a file while
//! it arrives, which is what this file is.

use std::{
    fs,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use super::{Installed, MAX_PACKED, Pack, sanitize};

/// The most a sender may put in the request line and headers.
const MAX_HEADER: usize = 16 * 1024;
/// How many sockets this answers at once. One is the upload, a couple are the page and its icons;
/// past that, whatever is connecting is not a person bringing one archive, and a thread per socket
/// on a phone is not something to hand out.
const MAX_CLIENTS: usize = 4;
/// Upload progress is worth a screen update, and nothing more often than this.
const PROGRESS_STEP: u64 = 1 << 20;

/// What the listener tells the game about an upload.
#[derive(Debug, Clone)]
pub enum News {
    /// Bytes accepted so far, and the whole size when the sender announced it.
    Progress { received: u64, total: u64 },
    /// An archive arrived, unpacked and is in place.
    Installed(Installed),
    /// Something was offered and refused, in the words the screen shows.
    Refused { error: String },
}

/// The running listener, and the only handle to it. Dropping this closes the port for good: the
/// accept loop notices the flag within one poll of its sleep and ends, and the drop waits for it, so
/// nothing is left listening once the game has its files and starts reading them.
pub struct Session {
    /// The address to show, `None` on a machine whose network nothing can be reached on.
    address: Option<String>,
    port: u16,
    pin: String,
    news: mpsc::Receiver<News>,
    line: Arc<Line>,
    thread: Option<thread::JoinHandle<()>>,
}

/// What the listener and every socket it answered work from: one archive at a time, in one folder,
/// under one code, and with a count of how often that code has been guessed at.
struct Line {
    /// Where an archive has to arrive: `Documents` on a phone.
    home: PathBuf,
    /// The six digits this session shows.
    pin: String,
    /// Set when the game stops waiting; the accept loop and every body in flight end on it.
    stop: AtomicBool,
    /// Set while an archive is arriving.
    busy: AtomicBool,
    /// Uploads that got the code wrong, which is what runs out, not the port.
    guesses: AtomicUsize,
    /// The sockets answered and thrown away past `MAX_CLIENTS`, counted so a flood cannot make one
    /// thread per socket; counted by the accept loop, before the thread exists.
    clients: AtomicUsize,
    sender: mpsc::Sender<News>,
}

impl Line {
    /// Whether this session has been knocked at enough to stop answering anybody, right code or not.
    /// The screen's own words are the cable and a relaunch, and a relaunch asks with new digits.
    fn guessed_out(&self) -> bool {
        self.guesses.load(Ordering::Relaxed) >= MAX_GUESSES
    }
}

/// How many wrong codes one session will look at. Six digits is a number a stranger can spend an
/// evening on, and the port is open for as long as the screen is, which is minutes.
const MAX_GUESSES: usize = 30;

/// How long the accept loop sleeps when nothing is arriving. It ends on a timer rather than on a
/// connection from its own process, because a loop that only wakes when something knocks could be
/// left holding the port -- and a phone that has started loading its game must not keep a door open.
const POLL: Duration = Duration::from_millis(20);

impl Session {
    /// Listen for an archive in `home` (`Documents` on a phone) until this is dropped.
    pub fn start(home: &Path) -> io::Result<Self> {
        let listener = TcpListener::bind(("0.0.0.0", 0))?;
        let port = listener.local_addr()?.port();
        listener.set_nonblocking(true)?;
        let (sender, news) = mpsc::channel();
        let pin = random_pin()?;
        let line = Arc::new(Line {
            home: home.to_owned(),
            pin: pin.clone(),
            stop: AtomicBool::new(false),
            busy: AtomicBool::new(false),
            guesses: AtomicUsize::new(0),
            clients: AtomicUsize::new(0),
            sender,
        });
        let loop_line = line.clone();
        let thread = thread::Builder::new()
            .name("horizon-import".into())
            .spawn(move || {
                // One thread per connection, because a browser that keeps the socket it used for
                // the page open must not also sit in front of the upload that page sends.
                loop {
                    if loop_line.stop.load(Ordering::Relaxed) {
                        return;
                    }
                    let stream = match listener.accept() {
                        Ok((stream, _)) => stream,
                        // Rust does not hand the listener's nonblocking mode to the socket it
                        // returns, but saying so is cheap and the reads below count on it.
                        Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                            thread::sleep(POLL);
                            continue;
                        }
                        Err(_) => continue,
                    };
                    if loop_line.clients.load(Ordering::Relaxed) >= MAX_CLIENTS {
                        // Closing it is the answer: nothing is read, no thread is spent on it.
                        continue;
                    }
                    let line = loop_line.clone();
                    loop_line.clients.fetch_add(1, Ordering::Relaxed);
                    if let Err(error) = thread::Builder::new()
                        .name("horizon-import-client".into())
                        .spawn(move || {
                            let _ = serve(stream, &line);
                            line.clients.fetch_sub(1, Ordering::Relaxed);
                        })
                    {
                        log::warn!("Horizon: no thread for an upload: {error}");
                        loop_line.clients.fetch_sub(1, Ordering::Relaxed);
                    }
                }
            })?;
        Ok(Self {
            address: local_address(),
            port,
            pin,
            news,
            line,
            thread: Some(thread),
        })
    }

    /// The port the page is served on.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The six digits a sender has to type.
    pub fn pin(&self) -> &str {
        &self.pin
    }

    /// The address to show beside the port, when this device is on a network worth naming. `None`
    /// leaves the Wi-Fi page off the screen, with the cable as the only door.
    pub fn address(&self) -> Option<&str> {
        self.address.as_deref()
    }

    /// Whether an upload is arriving right now: the screen shows a bar rather than a promise, and a
    /// second sender gets `503`.
    pub fn uploading(&self) -> bool {
        self.line.busy.load(Ordering::Relaxed)
    }

    /// Everything the listener has said since the last frame.
    pub fn poll(&mut self) -> Vec<News> {
        let mut news = Vec::new();
        while let Ok(item) = self.news.try_recv() {
            news.push(item);
        }
        news
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.line.stop.store(true, Ordering::Relaxed);
        // The loop notices within one poll, so this returns in milliseconds and the port is shut --
        // not merely meant to be shut sometime soon.
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// The first address a phone could be reached on. Enumerating interfaces is what
/// `horizon-friends` already does to decide whether a share is reachable, including which names to
/// ignore (VPN tunnels, link-local, loopback), so the same answer is used here rather than a second
/// guess about `en0`.
fn local_address() -> Option<String> {
    horizon_friends::reach::interface_addresses(false)
        .into_iter()
        .find(|ip| ip.is_ipv4())
        .map(|ip| ip.to_string())
}

/// Six digits, from the same source the login encryption uses.
fn random_pin() -> io::Result<String> {
    let mut bytes = [0u8; 4];
    horizon_protocol::crypto::random_bytes(&mut bytes)
        .map_err(|error| io::Error::other(error.to_string()))?;
    let digits = u32::from_be_bytes(bytes) % 1_000_000;
    Ok(format!("{digits:06}"))
}

/// Read the request line and headers. `None` when the sender closed, or sent something that is not
/// a request at all.
fn read_request(stream: &mut TcpStream) -> io::Result<Option<Request>> {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while head.len() < MAX_HEADER {
        if stream.read(&mut byte)? != 1 {
            return Ok(None);
        }
        head.push(byte[0]);
        if head.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    if !head.ends_with(b"\r\n\r\n") {
        return Ok(None);
    }
    let head = String::from_utf8_lossy(&head);
    let mut lines = head.lines().map(str::trim);
    let mut words = lines.next().unwrap_or_default().split_whitespace();
    let (Some(method), Some(target)) = (words.next(), words.next()) else {
        return Ok(None);
    };
    let (path, query) = match target.split_once('?') {
        Some((path, query)) => (path, query),
        None => (target, ""),
    };
    let mut request = Request {
        method: method.to_ascii_uppercase(),
        path: path.to_owned(),
        query: query.to_owned(),
        name: String::new(),
        pin: String::new(),
        expect: String::new(),
        length: 0,
    };
    for line in lines {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        match key.trim().to_ascii_lowercase().as_str() {
            "content-length" => request.length = value.trim().parse().unwrap_or_default(),
            "x-horizon-pin" => request.pin = value.trim().to_owned(),
            "x-horizon-name" => request.name = value.trim().to_owned(),
            // A terminal asks before sending 589 MB, and waits a second for an answer if it is not
            // given one; saying "go ahead" is one line and saves the wait.
            "expect" => request.expect = value.trim().to_ascii_lowercase(),
            _ => {}
        }
    }
    // `?pin=` is what a terminal sends (`curl --upload-file …?pin=123456`), the header what the page does.
    for pair in request.query.split('&') {
        match pair.split_once('=') {
            Some(("pin", value)) if request.pin.is_empty() => request.pin = value.to_owned(),
            Some(("name", value)) if request.name.is_empty() => {
                request.name = value.split('/').next_back().unwrap_or_default().to_owned();
            }
            _ => {}
        }
    }
    Ok(Some(request))
}

/// What a sender asked for. Only the fields this port can act on are kept, and the path is already
/// separated from its query.
struct Request {
    method: String,
    path: String,
    query: String,
    name: String,
    pin: String,
    expect: String,
    length: u64,
}

/// Answer one connection.
fn serve(mut stream: TcpStream, line: &Line) -> io::Result<()> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let Some(request) = read_request(&mut stream)? else {
        return Ok(());
    };
    if line.stop.load(Ordering::Relaxed) {
        return Ok(());
    }
    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/" | "/index.html") => respond(&mut stream, 200, "text/html", PAGE)?,
        ("GET", _) => respond(&mut stream, 404, "text/plain", "not found\n")?,
        ("PUT" | "POST", "/upload") => upload(&mut stream, line, request)?,
        _ => respond(&mut stream, 405, "text/plain", "use PUT /upload\n")?,
    }
    stream.flush()
}

/// Take the body to a file, then let it follow the same road as one dropped in over a cable.
fn upload(stream: &mut TcpStream, line: &Line, request: Request) -> io::Result<()> {
    if line.guessed_out() {
        // Right code or not, this session has been knocked at too often to keep answering. The
        // screen still says what the cable route is, and a relaunch asks with new digits.
        log::warn!("Horizon: too many wrong codes; the import page has stopped answering");
        return respond(
            stream,
            403,
            "text/plain",
            "too many wrong codes. Use the folder over a cable, or restart the app for new ones.\n",
        );
    }
    if !same_code(&line.pin, &request.pin) {
        line.guesses.fetch_add(1, Ordering::Relaxed);
        log::warn!("Horizon: an upload was refused for the wrong code");
        return respond(
            stream,
            403,
            "text/plain",
            "the six digits on the phone did not match\n",
        );
    }
    if request.length > MAX_PACKED {
        return respond(
            stream,
            413,
            "text/plain",
            "that is larger than a phone will hold\n",
        );
    }
    if line.busy.swap(true, Ordering::Relaxed) {
        return respond(
            stream,
            503,
            "text/plain",
            "the phone is taking another archive already\n",
        );
    }
    struct Busy<'a>(&'a AtomicBool);
    impl Drop for Busy<'_> {
        fn drop(&mut self) {
            self.0.store(false, Ordering::Relaxed);
        }
    }
    let _busy = Busy(&line.busy);

    let name = sanitize(
        request
            .name
            .split('/')
            .next_back()
            .unwrap_or("upload")
            .trim_end_matches(".zip"),
    );
    let part = line.home.join(".import").join(format!("{name}.zip.part"));
    let archive = line.home.join(format!("{name}.zip"));
    // Everything that could refuse this has been refused, so the body is wanted: say so.
    if request.expect == "100-continue" {
        stream.write_all(b"HTTP/1.1 100 Continue\r\n\r\n")?;
    }
    if let Err(error) = stream_body(stream, &part, request.length, line) {
        let _ = fs::remove_file(&part);
        let message = format!("the upload did not finish: {error}");
        let _ = line.sender.send(News::Refused {
            error: message.clone(),
        });
        return respond(stream, 400, "text/plain", &format!("{message}\n"));
    }
    // Land it where the cable route lands it, and let one path unpack it: a phone that dies between
    // these two lines still leaves an archive in `Documents` for the next launch to install.
    let installed = fs::rename(&part, &archive)
        .and_then(|()| {
            Pack::open(&archive)
                .map_err(|error| io::Error::other(error.to_string()))?
                .install(&line.home)
                .map_err(|error| io::Error::other(error.to_string()))
        })
        .inspect(|installed| {
            let _ = line.sender.send(News::Installed(installed.clone()));
        });
    match installed {
        Ok(installed) => respond(
            stream,
            200,
            "text/plain",
            &format!(
                "the phone has the {}; you can close this page.\n",
                installed.kind.label()
            ),
        ),
        Err(error) => {
            // Park it the way the startup scan does, so the same failure is not retried every launch.
            let _ = fs::rename(&archive, archive.with_extension("zip.failed"));
            let message = format!("the phone could not use that archive: {error}");
            let _ = line.sender.send(News::Refused {
                error: message.clone(),
            });
            respond(stream, 422, "text/plain", &format!("{message}\n"))
        }
    }
}

/// Copy exactly `length` bytes of the body into `part`, reporting each megabyte as it arrives.
fn stream_body(stream: &mut TcpStream, part: &Path, length: u64, line: &Line) -> io::Result<()> {
    if let Some(parent) = part.parent() {
        fs::create_dir_all(parent)?;
    }
    // A long timeout for the body: 589 MB over a phone's Wi-Fi takes minutes, and one slow packet
    // in that time is not a broken connection.
    stream.set_read_timeout(Some(Duration::from_secs(120)))?;
    let mut file = fs::File::create(part)?;
    let mut buffer = vec![0u8; 64 * 1024];
    let mut received = 0u64;
    let mut reported = 0u64;
    while received < length {
        if line.stop.load(Ordering::Relaxed) {
            // The game stopped waiting -- it has its files, or it is going away -- so this upload is
            // now writing into a folder nobody is about to read.
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "the phone stopped listening",
            ));
        }
        let want = (length - received).min(buffer.len() as u64) as usize;
        let read = stream.read(&mut buffer[..want])?;
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "the sender stopped mid-upload",
            ));
        }
        file.write_all(&buffer[..read])?;
        received += read as u64;
        if received - reported >= PROGRESS_STEP || received == length {
            reported = received;
            let _ = line.sender.send(News::Progress {
                received,
                total: length,
            });
        }
    }
    file.flush()
}

/// Compare the code without stopping at the first difference, which is what makes the time it takes
/// say nothing about how much of a guess was right.
fn same_code(wanted: &str, given: &str) -> bool {
    let (wanted, given) = (wanted.as_bytes(), given.as_bytes());
    wanted.len() == given.len()
        && !wanted.is_empty()
        && wanted
            .iter()
            .zip(given)
            .fold(0u8, |difference, (a, b)| difference | (a ^ b))
            == 0
}

/// One finished response, never chunked and never kept alive.
fn respond(stream: &mut TcpStream, status: u16, kind: &str, body: &str) -> io::Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        413 => "Content Too Large",
        422 => "Unprocessable Content",
        _ => "Service Unavailable",
    };
    // Build the response before writing: formatting directly into the socket
    // can emit a partial status line before an early-refused upload is closed.
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {kind}; charset=utf-8\r\nContent-Length: \
         {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())
}

/// The page the phone's address serves: choose the archive, type the code, watch it arrive. The
/// digits are not in it, which is the whole point of showing them on the phone instead.
const PAGE: &str = r#"<!doctype html>
<html lang=en><head>
<meta charset=utf-8>
<meta name=viewport content="width=device-width,initial-scale=1">
<title>Send to Horizon</title>
<style>
 body{background:#111a21;color:#e8eef2;font:16px/1.55 system-ui,sans-serif;margin:40px auto;max-width:620px;padding:0 20px}
 h1{font-size:22px;margin-bottom:4px}
 p{color:#9db0bb}
 code{background:#18232c;padding:2px 6px;border-radius:6px}
 input,button{font:inherit;color:#e8eef2;background:#18232c;border:1px solid #3b4a54;border-radius:8px;padding:10px 12px;margin:6px 8px 6px 0}
 button{background:#4c8bf5;border-color:#4c8bf5;cursor:pointer}
 #pin{width:9ch;letter-spacing:2px}
 progress{width:100%;height:10px;margin-top:12px;display:none}
 pre{white-space:pre-wrap;color:#9db0bb}
</style></head><body>
<h1>Send game files to the phone</h1>
<p>Choose the archive made with <code>horizon-client --pack-resources</code>, then type the six
digits the phone shows. A world and a resource pack go in the same way.</p>
<input id=file type=file accept=".zip,application/zip">
<p>Code <input id=pin inputmode=numeric autocomplete=off maxlength=6 spellcheck=false></p>
<button id=send>Send</button>
<progress id=bar max=100 value=0></progress>
<pre id=out></pre>
<script>
document.getElementById("send").onclick = () => {
  const file = document.getElementById("file").files[0];
  const out = document.getElementById("out");
  if (!file) { out.textContent = "Choose the .zip first."; return; }
  const pin = document.getElementById("pin").value.trim();
  if (!/^[0-9]{6}$/.test(pin)) { out.textContent = "Type the six digits on the phone."; return; }
  const bar = document.getElementById("bar");
  const xhr = new XMLHttpRequest();
  xhr.open("PUT", "/upload");
  xhr.setRequestHeader("X-Horizon-Pin", pin);
  xhr.setRequestHeader("X-Horizon-Name", file.name);
  bar.style.display = "block"; bar.value = 0;
  out.textContent = "Sending " + file.name + "…";
  xhr.upload.onprogress = (e) => { if (e.lengthComputable) bar.value = 100 * e.loaded / e.total; };
  xhr.onload = () => { bar.value = 100; out.textContent = xhr.responseText; };
  xhr.onerror = () => { out.textContent = "The phone closed the connection."; };
  xhr.send(file);
};
</script></body></html>
"#;

#[cfg(test)]
mod tests {
    use super::super::Kind;
    use super::*;
    use std::net::Shutdown;

    fn temp_home(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("horizon-http-test-{}-{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// One request, answered in full: the status line and the body, with the socket closed after.
    fn ask(port: u16, request: String, body: &[u8]) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        stream.write_all(request.as_bytes()).unwrap();
        stream.write_all(body).unwrap();
        stream.flush().unwrap();
        let mut reply = String::new();
        // The answer ends where the sender stops; a refused upload still closes its own half.
        stream.shutdown(Shutdown::Write).unwrap();
        let mut read = [0u8; 4096];
        loop {
            match stream.read(&mut read) {
                Ok(0) | Err(_) => break,
                Ok(n) => reply.push_str(String::from_utf8_lossy(&read[..n]).as_ref()),
            }
        }
        reply
    }

    fn header(status: u16) -> String {
        format!("HTTP/1.1 {status} ")
    }

    /// A prepared tree small enough to send in a test, made by the same writer a player uses. It is
    /// built and left outside `home`, because a test that finds a loose zip in `Documents` cannot
    /// then tell what the phone put there.
    fn archive(tag: &str) -> Vec<u8> {
        let work =
            std::env::temp_dir().join(format!("horizon-http-source-{}-{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&work);
        let root = work.join("Horizon-26.3");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("horizon-resources.json"), r#"{"schema":7}"#).unwrap();
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::write(root.join("assets/grass.png"), "png").unwrap();
        let out = work.join("Horizon-26.3.zip");
        super::super::pack(&root, &out, |_, _| {}).unwrap();
        let bytes = fs::read(&out).unwrap();
        let _ = fs::remove_dir_all(&work);
        bytes
    }

    #[test]
    fn the_page_promises_nothing_it_cannot_keep() {
        let home = temp_home("page");
        let session = Session::start(&home).unwrap();
        let reply = ask(session.port(), "GET / HTTP/1.1\r\n\r\n".into(), &[]);
        assert!(reply.starts_with(&header(200)), "{reply}");
        assert!(reply.contains("Send game files to the phone"));
        // The digits are shown on the phone, never in what a stranger can read.
        assert!(
            !reply.contains(session.pin()),
            "the page hands out the code it asks for"
        );
        assert!(
            ask(
                session.port(),
                "GET /elsewhere HTTP/1.1\r\n\r\n".into(),
                &[]
            )
            .starts_with(&header(404))
        );
        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn the_code_is_the_only_way_in() {
        let home = temp_home("code");
        let session = Session::start(&home).unwrap();
        let body = archive("wrong-code");
        for pin in ["", "000000", "12345", session.pin().repeat(2).as_str()] {
            let reply = ask(
                session.port(),
                format!(
                    "PUT /upload?name=game HTTP/1.1\r\nHost: phone\r\nX-Horizon-Pin: {pin}\r\n\
                     Content-Length: {}\r\n\r\n",
                    body.len()
                ),
                &body,
            );
            assert!(reply.starts_with(&header(403)), "{pin:?} got {reply}");
        }
        assert!(!home.join("game.zip").exists());
        assert!(!home.join("resources").exists());
        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn a_session_that_has_been_guessed_at_answers_nobody() {
        // Driven through `serve` on a socket of its own, because what is being tested is the budget:
        // thirty knocks are already spent, and the thirty-first sender has the right code.
        let home = temp_home("guessed-out");
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let client = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let (server, _) = listener.accept().unwrap();
        let line = Line {
            home: home.clone(),
            pin: "481930".into(),
            stop: AtomicBool::new(false),
            busy: AtomicBool::new(false),
            guesses: AtomicUsize::new(MAX_GUESSES),
            clients: AtomicUsize::new(0),
            sender: mpsc::channel().0,
        };
        // No body: the code is what is being spent here, and a socket closed after refusing one
        // without reading what followed it would be a reset rather than an answer.
        let request = "PUT /upload HTTP/1.1\r\nHost: phone\r\nX-Horizon-Pin: 481930\r\n\
                       Content-Length: 0\r\n\r\n"
            .as_bytes()
            .to_vec();
        thread::spawn(move || {
            let _ = serve(server, &line);
        });
        let mut client = client;
        client.write_all(&request).unwrap();
        client.flush().unwrap();
        let mut reply = String::new();
        client.read_to_string(&mut reply).unwrap();
        assert!(reply.starts_with(&header(403)), "{reply}");
        assert!(reply.contains("too many wrong codes"), "{reply}");
        assert!(!home.join("upload.zip").exists());
        assert!(!home.join("resources").exists());
        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn an_archive_posted_over_loopback_installs_itself() {
        let home = temp_home("installed");
        let mut session = Session::start(&home).unwrap();
        let body = archive("horizon-26.3");
        let reply = ask(
            session.port(),
            format!(
                "PUT /upload HTTP/1.1\r\nHost: phone\r\nX-Horizon-Pin: {}\r\n\
                 X-Horizon-Name: horizon-26.3.zip\r\nExpect: 100-continue\r\nContent-Length: {}\r\n\r\n",
                session.pin(),
                body.len()
            ),
            &body,
        );
        // The terminal's shape: an interim answer, then the real one, once the body is taken.
        assert!(reply.starts_with("HTTP/1.1 100 "), "{reply}");
        assert!(reply.contains(&header(200)), "{reply}");
        assert!(home.join("resources/horizon-resources.json").is_file());
        // The archive was a copy of what the computer still holds, so it does not stay on the phone.
        assert!(!home.join("horizon-26.3.zip").exists());
        assert!(!home.join(".import/horizon-26.3.zip.part").exists());
        let news = session.poll();
        assert!(
            news.iter().any(|news| matches!(
                news,
                News::Installed(placed) if placed.kind == Kind::Resources
            )),
            "{news:?}"
        );
        assert!(
            news.iter().any(|news| matches!(
                news,
                News::Progress {
                    received,
                    total,
                } if *received == *total && *total == body.len() as u64
            )),
            "{news:?}"
        );
        drop(session);
        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn an_archive_that_is_not_horizons_is_answered_and_parked() {
        let home = temp_home("foreign");
        let session = Session::start(&home).unwrap();
        let mut zip = zip::ZipWriter::new(fs::File::create(home.join("photos.zip")).unwrap());
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("holiday.jpg", options).unwrap();
        zip.write_all(b"jpeg").unwrap();
        zip.finish().unwrap();
        let body = fs::read(home.join("photos.zip")).unwrap();
        let reply = ask(
            session.port(),
            format!(
                "PUT /upload?name=photos HTTP/1.1\r\nHost: phone\r\nX-Horizon-Pin: {}\r\n\
                 Content-Length: {}\r\n\r\n",
                session.pin(),
                body.len()
            ),
            &body,
        );
        assert!(reply.starts_with(&header(422)), "{reply}");
        assert!(!home.join("photos.zip").exists());
        assert!(home.join("photos.zip.failed").is_file());
        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn a_sender_that_stops_mid_upload_leaves_no_half_file() {
        let home = temp_home("stopped");
        let session = Session::start(&home).unwrap();
        let body = archive("short");
        // Announce the whole archive and send a third of it.
        let _ = ask(
            session.port(),
            format!(
                "PUT /upload?name=short HTTP/1.1\r\nHost: phone\r\nX-Horizon-Pin: {}\r\n\
                 Content-Length: {}\r\n\r\n",
                session.pin(),
                body.len() * 4
            ),
            &body[..body.len() / 3],
        );
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if !home.join(".import/short.zip.part").exists() && !home.join("short.zip").exists() {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "the half file was never removed"
            );
            thread::sleep(Duration::from_millis(20));
        }
        assert!(!home.join("resources").exists());
        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn the_code_is_six_digits_and_each_session_has_its_own() {
        let home = temp_home("pin");
        let first = Session::start(&home).unwrap();
        let second = Session::start(&home).unwrap();
        assert_eq!(first.pin().len(), 6);
        assert!(
            first.pin().bytes().all(|byte| byte.is_ascii_digit()),
            "{}",
            first.pin()
        );
        assert_ne!(first.pin(), second.pin());
        assert!(first.port() != second.port());
        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn a_closed_session_stops_answering() {
        let home = temp_home("closed");
        let session = Session::start(&home).unwrap();
        let port = session.port();
        assert!(ask(port, "GET / HTTP/1.1\r\n\r\n".into(), &[]).starts_with(&header(200)));
        drop(session);
        // The port is released, so nothing is listening for a stranger to find.
        assert!(TcpStream::connect(("127.0.0.1", port)).is_err());
        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn a_name_cannot_walk_out_of_the_folder() {
        let home = temp_home("escape");
        let session = Session::start(&home).unwrap();
        let body = archive("walk");
        let reply = ask(
            session.port(),
            format!(
                "PUT /upload?name=../../escape HTTP/1.1\r\nHost: phone\r\nX-Horizon-Pin: {}\r\n\
                 Content-Length: {}\r\n\r\n",
                session.pin(),
                body.len()
            ),
            &body,
        );
        assert!(reply.starts_with(&header(200)), "{reply}");
        // Reduced to a label, and installed where the game reads: nothing of the name is a path.
        assert!(home.join("resources/horizon-resources.json").is_file());
        assert!(
            !home.parent().unwrap().join("escape.zip").exists(),
            "the name walked out of Documents"
        );
        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn codes_are_matched_whole_or_not_at_all() {
        assert!(same_code("481930", "481930"));
        assert!(!same_code("481930", "48193"));
        assert!(!same_code("481930", "481931"));
        assert!(!same_code("481930", ""));
        assert!(!same_code("", ""));
    }
}
