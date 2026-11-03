use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use souvlaki::{MediaControlEvent, MediaControls, PlatformConfig};

pub struct MediaController {
    pub media_controls: MediaControls,
    pub events: Receiver<MediaControlEvent>,
}

pub fn start_souvlaki_thread() -> Option<MediaController> {
    let (ev_tx, events) = mpsc::channel::<MediaControlEvent>();

    let config = PlatformConfig {
        dbus_name: "nulin_music_player",
        display_name: "Nulin",
        hwnd: None
    };
    let mut controls = MediaControls::new(config).ok()?;

    controls.attach(
        move |event| {
            let _ = ev_tx.send(event);
        }
    ).ok()?;

    Some(MediaController { media_controls: controls, events })
}
