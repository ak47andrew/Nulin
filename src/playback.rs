use std::{
    fs::File, path::PathBuf,
    sync::{atomic::{AtomicBool, AtomicU64, Ordering}, mpsc, Arc},
    thread, time::Duration,
};
use std::sync::Mutex;
use rodio::Source;

pub enum Command {
    Play(PathBuf), // duration comes from your library DB, see below
    Pause,
    Resume,
    Seek(Duration),
    SetVolume(f32),
    Stop,
}

pub enum Event {
    SongLoaded,
    Finished,
}

#[derive(Default)]
pub struct Shared {
    pub position_ms: AtomicU64,
    pub duration_ms: AtomicU64,
    pub volume: Arc<Mutex<f32>>,
    pub playing: AtomicBool,
    pub track_loaded: AtomicBool,
}

pub struct PlayerHandle {
    pub tx: mpsc::Sender<Command>,
    pub events: mpsc::Receiver<Event>,
    pub shared: Arc<Shared>,
}

pub fn spawn_player() -> PlayerHandle {
    let (tx, rx) = mpsc::channel();
    let (ev_tx, events) = mpsc::channel();
    let shared = Arc::new(Shared::default());
    let sh = shared.clone();
    *sh.volume.lock().unwrap() = 0.5;

    thread::spawn(move || {
        // The stream must be created and kept alive on this thread.
        let mut handle = rodio::DeviceSinkBuilder::open_default_sink()
            .expect("open default audio stream");
        handle.log_on_drop(false);
        let player = rodio::Player::connect_new(&handle.mixer());
        player.set_volume(0.5);

        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Command::Play(path)) => {
                    player.clear();
                    let file = File::open(&path).unwrap();
                    let len = file.metadata().unwrap().len();
                    let src = rodio::Decoder::builder()
                        .with_data(file)
                        .with_byte_len(len)
                        .with_seekable(true)
                        .build().unwrap();
                    let drt = src.total_duration().unwrap().as_millis() as u64;
                    player.append(src);
                    player.play();
                    sh.duration_ms.store(drt, Ordering::Relaxed);
                    sh.track_loaded.store(true, Ordering::Relaxed);
                    let _ = ev_tx.send(Event::SongLoaded);
                }
                Ok(Command::Pause) => player.pause(),
                Ok(Command::Resume) => player.play(),
                Ok(Command::Seek(t)) => { let _ = player.try_seek(t); }
                Ok(Command::SetVolume(v)) => { player.set_volume(v); *sh.volume.lock().unwrap() = v; },
                Ok(Command::Stop) => { player.clear(); sh.track_loaded.store(false, Ordering::Relaxed); }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }

            sh.position_ms.store(player.get_pos().as_millis() as u64, Ordering::Relaxed);
            sh.playing.store(sh.track_loaded.load(Ordering::Relaxed) && !player.is_paused() && !player.empty(), Ordering::Relaxed);

            if sh.track_loaded.load(Ordering::Relaxed) && player.empty() {
                sh.track_loaded.store(false, Ordering::Relaxed);
                let _ = ev_tx.send(Event::Finished);
            }
        }
    });

    PlayerHandle { tx, events, shared }
}
