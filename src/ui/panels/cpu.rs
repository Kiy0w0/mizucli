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
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border())
        .title(Span::styled(
            format!(" CPU  {} ", app.metrics.cpu_brand),
            theme.panel_title(),
        ))
        .style(theme.panel_bg());
    let inner = block.inner(area);
    f.render_widget(block, area);

    let cores = &app.metrics.cpu_cores;
    if cores.is_empty() || inner.height == 0 {
        return;
    }

    let core_count = cores.len();
    let cols = if core_count <= 4 {
        2
    } else if core_count <= 16 {
        4
    } else {
        8
    };
    let rows = core_count.div_ceil(cols);
    let row_h = (inner.height as usize / rows.max(1)).max(1) as u16;
    let col_w = inner.width / cols as u16;

    for (i, usage) in cores.iter().enumerate() {
        let r = (i / cols) as u16;
        let c = (i % cols) as u16;
        let x = inner.x + c * col_w;
        let y = inner.y + r * row_h;
        if y >= inner.y + inner.height {
            break;
        }
        let cell = Rect {
            x,
            y,
            width: col_w.saturating_sub(1).max(1),
            height: row_h.min(inner.y + inner.height - y),
        };
        let pct = (*usage as u16).min(100);
        let color = theme.gauge_color(pct as f64 / 100.0);
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .title(Span::styled(format!("c{:02}", i), Style::default().fg(p.dim))),
            )
            .gauge_style(Style::default().fg(color).bg(p.abyss))
            .percent(pct)
            .label(Span::styled(format!("{:>3}%", pct), theme.label_style()));
        f.render_widget(gauge, cell);
    }
}
