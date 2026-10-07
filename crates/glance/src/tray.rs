//! System tray implementation using StatusNotifierItem (ksni).

use ksni::blocking::TrayMethods;
use ksni::{MenuItem, Tray};

#[derive(Debug, Clone, Copy)]
pub enum TrayEvent {
    ToggleWindow,
    Quit,
}

pub struct GlanceTray {
    event_tx: async_channel::Sender<TrayEvent>,
}

impl Tray for GlanceTray {
    fn id(&self) -> String {
        "io.github.maycon.Glance".into()
    }

    fn title(&self) -> String {
        "Glance".into()
    }

    fn icon_name(&self) -> String {
        "utilities-system-monitor".into()
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.event_tx.try_send(TrayEvent::ToggleWindow);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        use ksni::menu::*;
        let tx = self.event_tx.clone();
        let tx_quit = self.event_tx.clone();
        vec![
            StandardItem {
                label: "Show / Hide Glance".into(),
                activate: Box::new(move |_| {
                    let _ = tx.try_send(TrayEvent::ToggleWindow);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit Glance".into(),
                activate: Box::new(move |_| {
                    let _ = tx_quit.try_send(TrayEvent::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub fn spawn_tray(
    event_tx: async_channel::Sender<TrayEvent>,
) -> Option<ksni::blocking::Handle<GlanceTray>> {
    let tray = GlanceTray { event_tx };
    match tray.assume_sni_available(true).spawn() {
        Ok(handle) => Some(handle),
        Err(err) => {
            eprintln!("Glance: Note - system tray unavailable: {err:?}");
            None
        }
    }
}
