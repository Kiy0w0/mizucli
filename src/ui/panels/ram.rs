use crate::app::App;
use ratatui::{
    layout::Rect,
    style::Style,
    text::Span,
    widgets::{Block, Borders, Gauge},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let p = &theme.palette;
    let used = app.metrics.ram_used_mb;
    let total = app.metrics.ram_total_mb.max(1);
    let pct = ((used as f64 / total as f64) * 100.0).clamp(0.0, 100.0) as u16;
    let used_h = humansize::format_size(used as u64 * 1024 * 1024, humansize::BINARY);
    let total_h = humansize::format_size(total as u64 * 1024 * 1024, humansize::BINARY);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border())
        .title(Span::styled(" RAM ", theme.panel_title()))
        .style(theme.panel_bg());

    let gauge = Gauge::default()
        .block(block)
        .gauge_style(Style::default().fg(theme.gauge_color(pct as f64 / 100.0)).bg(p.abyss))
        .percent(pct)
        .label(Span::styled(
            format!("{} / {} ({}%)", used_h, total_h, pct),
            theme.label_style(),
        ));

    f.render_widget(gauge, area);
}
