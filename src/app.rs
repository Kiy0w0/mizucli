use crate::animation::BadAppleState;
use crate::config::settings::Settings;
use crate::metrics::Metrics;
use crate::ui::theme::MizuTheme;
use tokio::sync::mpsc::Receiver;

pub struct App {
    pub metrics: Metrics,
    pub bad_apple: BadAppleState,
    pub metric_rx: Receiver<Metrics>,
    pub history_net_rx: Vec<f64>,
    pub history_net_tx: Vec<f64>,
    pub history_cpu: Vec<f64>,
    pub history_disk_read: Vec<f64>,
    pub history_disk_write: Vec<f64>,
    pub active_tab: u8,
    pub settings: Settings,
    pub theme: MizuTheme,
}

impl App {
    pub fn new(metric_rx: Receiver<Metrics>, settings: Settings) -> Self {
        let theme = MizuTheme::from_name(settings.theme);
        Self {
            metrics: Metrics::default(),
            bad_apple: BadAppleState::new(),
            metric_rx,
            history_net_rx: vec![0.0; 60],
            history_net_tx: vec![0.0; 60],
            history_cpu: vec![0.0; 60],
            history_disk_read: vec![0.0; 60],
            history_disk_write: vec![0.0; 60],
            active_tab: 0,
            settings,
            theme,
        }
    }

    pub fn set_tab(&mut self, tab: u8) {
        self.active_tab = if tab <= 2 { tab } else { 0 };
    }

    pub fn cycle_theme(&mut self) {
        self.settings.theme = self.settings.theme.next();
        self.theme = MizuTheme::from_name(self.settings.theme);
        let _ = self.settings.save();
    }

    pub fn toggle_flow(&mut self) {
        self.settings.flow_enabled = !self.settings.flow_enabled;
        let _ = self.settings.save();
    }

    pub fn tick(&mut self) {
        while let Ok(m) = self.metric_rx.try_recv() {
            push_cap(&mut self.history_cpu, m.cpu_total);
            push_cap(&mut self.history_net_rx, m.net_rx_kbps);
            push_cap(&mut self.history_net_tx, m.net_tx_kbps);
            push_cap(&mut self.history_disk_read, m.disk_read_kbps);
            push_cap(&mut self.history_disk_write, m.disk_write_kbps);
            self.metrics = m;
        }
        if self.settings.flow_enabled {
            self.bad_apple.tick(0.016);
        }
    }
}

fn push_cap(v: &mut Vec<f64>, x: f64) {
    v.push(x);
    if v.len() > 60 {
        v.remove(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::ThemeName;
    use tokio::sync::mpsc;

    fn make_app() -> App {
        let (_, rx) = mpsc::channel(1);
        App::new(rx, Settings::default())
    }

    #[test]
    fn tab_defaults_to_zero() {
        assert_eq!(make_app().active_tab, 0);
    }

    #[test]
    fn set_tab_valid() {
        let mut app = make_app();
        app.set_tab(1);
        assert_eq!(app.active_tab, 1);
        app.set_tab(2);
        assert_eq!(app.active_tab, 2);
    }

    #[test]
    fn set_tab_clamps_out_of_range() {
        let mut app = make_app();
        app.set_tab(3);
        assert_eq!(app.active_tab, 0);
        app.set_tab(255);
        assert_eq!(app.active_tab, 0);
    }

    #[test]
    fn set_tab_zero_resets() {
        let mut app = make_app();
        app.set_tab(2);
        app.set_tab(0);
        assert_eq!(app.active_tab, 0);
    }

    #[test]
    fn cycle_theme_rotates() {
        let mut app = make_app();
        assert_eq!(app.theme.name, ThemeName::Mizu);
        app.cycle_theme();
        assert_eq!(app.theme.name, ThemeName::Abyss);
        app.cycle_theme();
        assert_eq!(app.theme.name, ThemeName::Coral);
        app.cycle_theme();
        assert_eq!(app.theme.name, ThemeName::Mizu);
    }

    #[test]
    fn toggle_flow_persists() {
        let mut app = make_app();
        let initial = app.settings.flow_enabled;
        app.toggle_flow();
        assert_ne!(app.settings.flow_enabled, initial);
    }
}
