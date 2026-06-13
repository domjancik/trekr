use image::codecs::jpeg::JpegEncoder;
use image::{ColorType, ImageEncoder};
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, SyncSender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub const DEFAULT_PREVIEW_FPS: u32 = 12;
const MJPEG_BOUNDARY: &str = "trekr-frame";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewStreamOptions {
    pub bind_addr: String,
    pub fps: u32,
}

impl PreviewStreamOptions {
    pub fn endpoint_url(&self) -> String {
        format!("http://{}/preview.mjpg", self.bind_addr)
    }
}

#[derive(Debug)]
struct RawFrame {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

#[derive(Debug, Clone)]
struct EncodedFrame {
    generation: u64,
    jpeg: Arc<Vec<u8>>,
}

#[derive(Debug, Default)]
struct SharedFrameState {
    latest: Option<EncodedFrame>,
}

#[derive(Debug, Default)]
struct PreviewShared {
    state: Mutex<SharedFrameState>,
    changed: Condvar,
    running: AtomicBool,
    generation: AtomicU64,
}

pub(crate) struct PreviewStreamRuntime {
    raw_frame_tx: SyncSender<RawFrame>,
    shared: Arc<PreviewShared>,
    listener_addr: SocketAddr,
    min_capture_interval: Duration,
    last_capture_at: Option<Instant>,
    encoder_thread: Option<JoinHandle<()>>,
    listener_thread: Option<JoinHandle<()>>,
}

impl PreviewStreamRuntime {
    pub(crate) fn start(options: PreviewStreamOptions) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&options.bind_addr)?;
        listener.set_nonblocking(true)?;
        let listener_addr = listener.local_addr()?;
        let shared = Arc::new(PreviewShared {
            state: Mutex::new(SharedFrameState::default()),
            changed: Condvar::new(),
            running: AtomicBool::new(true),
            generation: AtomicU64::new(0),
        });
        let (raw_frame_tx, raw_frame_rx) = mpsc::sync_channel::<RawFrame>(1);

        let encoder_shared = Arc::clone(&shared);
        let encoder_thread = thread::spawn(move || {
            loop {
                match raw_frame_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(frame) => {
                        if let Some(encoded) = encode_jpeg(frame) {
                            let generation =
                                encoder_shared.generation.fetch_add(1, Ordering::SeqCst) + 1;
                            let mut state = encoder_shared
                                .state
                                .lock()
                                .unwrap_or_else(|poison| poison.into_inner());
                            state.latest = Some(EncodedFrame {
                                generation,
                                jpeg: Arc::new(encoded),
                            });
                            drop(state);
                            encoder_shared.changed.notify_all();
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        if !encoder_shared.running.load(Ordering::SeqCst) {
                            break;
                        }
                    }
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        let listener_shared = Arc::clone(&shared);
        let listener_thread = thread::spawn(move || {
            loop {
                if !listener_shared.running.load(Ordering::SeqCst) {
                    break;
                }
                match listener.accept() {
                    Ok((stream, _)) => {
                        let connection_shared = Arc::clone(&listener_shared);
                        thread::spawn(move || {
                            let _ = handle_connection(stream, connection_shared);
                        });
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(20));
                    }
                    Err(_) => {
                        if !listener_shared.running.load(Ordering::SeqCst) {
                            break;
                        }
                        thread::sleep(Duration::from_millis(50));
                    }
                }
            }
        });

        Ok(Self {
            raw_frame_tx,
            shared,
            listener_addr,
            min_capture_interval: Duration::from_secs_f64(1.0 / f64::from(options.fps.max(1))),
            last_capture_at: None,
            encoder_thread: Some(encoder_thread),
            listener_thread: Some(listener_thread),
        })
    }

    pub(crate) fn listener_addr(&self) -> SocketAddr {
        self.listener_addr
    }

    pub(crate) fn should_capture_now(&self, now: Instant) -> bool {
        self.last_capture_at
            .is_none_or(|last| now.saturating_duration_since(last) >= self.min_capture_interval)
    }

    pub(crate) fn publish_rgba_frame(&mut self, width: u32, height: u32, rgba: Vec<u8>) {
        self.last_capture_at = Some(Instant::now());
        let frame = RawFrame {
            width,
            height,
            rgba,
        };
        match self.raw_frame_tx.try_send(frame) {
            Ok(()) => {}
            Err(mpsc::TrySendError::Full(_)) => {}
            Err(mpsc::TrySendError::Disconnected(_)) => {}
        }
    }
}

impl Drop for PreviewStreamRuntime {
    fn drop(&mut self) {
        self.shared.running.store(false, Ordering::SeqCst);
        self.shared.changed.notify_all();
        if let Some(handle) = self.listener_thread.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.encoder_thread.take() {
            let _ = handle.join();
        }
    }
}

fn handle_connection(
    mut stream: TcpStream,
    shared: Arc<PreviewShared>,
) -> Result<(), Box<dyn std::error::Error>> {
    stream.set_nodelay(true)?;
    let request_line = {
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut line = String::new();
        let _ = reader.read_line(&mut line)?;
        line
    };
    let path = request_line.split_whitespace().nth(1).unwrap_or("/");

    match path {
        "/" => {
            let body = "<!doctype html><html><head><meta charset=\"utf-8\"><title>trekr preview</title><style>html,body{margin:0;background:#000;height:100%;}img{display:block;width:100%;height:100%;object-fit:contain;}</style></head><body><img src=\"/preview.mjpg\" alt=\"trekr preview\"></body></html>";
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )?;
        }
        "/healthz" => {
            let body = "ok\n";
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )?;
        }
        "/preview.mjpg" => {
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nCache-Control: no-cache\r\nPragma: no-cache\r\nConnection: close\r\nContent-Type: multipart/x-mixed-replace; boundary={}\r\n\r\n",
                MJPEG_BOUNDARY
            )?;
            stream.flush()?;
            serve_mjpeg_stream(stream, shared)?;
        }
        _ => {
            let body = "not found\n";
            write!(
                stream,
                "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )?;
        }
    }

    Ok(())
}

fn serve_mjpeg_stream(
    mut stream: TcpStream,
    shared: Arc<PreviewShared>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_generation = 0_u64;
    loop {
        let next_frame = {
            let mut state = shared
                .state
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            loop {
                if !shared.running.load(Ordering::SeqCst) {
                    return Ok(());
                }
                if let Some(frame) = state
                    .latest
                    .as_ref()
                    .filter(|frame| frame.generation > last_generation)
                    .cloned()
                {
                    break frame;
                }
                state = shared
                    .changed
                    .wait(state)
                    .unwrap_or_else(|poison| poison.into_inner());
            }
        };

        last_generation = next_frame.generation;
        write!(
            stream,
            "--{}\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
            MJPEG_BOUNDARY,
            next_frame.jpeg.len()
        )?;
        stream.write_all(next_frame.jpeg.as_slice())?;
        stream.write_all(b"\r\n")?;
        stream.flush()?;
    }
}

fn encode_jpeg(frame: RawFrame) -> Option<Vec<u8>> {
    let mut rgb = Vec::with_capacity(frame.rgba.len() / 4 * 3);
    for chunk in frame.rgba.chunks_exact(4) {
        rgb.extend_from_slice(&chunk[..3]);
    }
    let mut encoded = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut encoded, 80);
    encoder
        .write_image(&rgb, frame.width, frame.height, ColorType::Rgb8.into())
        .ok()?;
    Some(encoded)
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_PREVIEW_FPS, PreviewStreamOptions};

    #[test]
    fn preview_stream_options_keep_bind_address_and_default_fps_contract() {
        let options = PreviewStreamOptions {
            bind_addr: "0.0.0.0:8090".to_owned(),
            fps: DEFAULT_PREVIEW_FPS,
        };
        assert_eq!(options.endpoint_url(), "http://0.0.0.0:8090/preview.mjpg");
        assert_eq!(options.fps, 12);
    }
}
