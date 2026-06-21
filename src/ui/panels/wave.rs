use crate::app::App;
use crate::bad_apple_frames::{H as FRAME_H, W as FRAME_W};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let p = &theme.palette;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(p.blue))
        .title(Span::styled(" ~ flow ~ ", Style::default().fg(p.cyan).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(p.deep));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 4 || inner.height < 2 {
        return;
    }

    if !app.settings.flow_enabled {
        let msg = Paragraph::new(Line::from(Span::styled(
            "flow paused — press f to resume",
            Style::default().fg(p.dim),
        )))
        .alignment(Alignment::Center)
        .style(Style::default().bg(p.deep));
        f.render_widget(msg, inner);
        return;
    }

    let pixels = app.bad_apple.decode_current();
    let fw = FRAME_W as usize;
    let fh = FRAME_H as usize;

    let tw = inner.width as usize;
    let th = inner.height as usize;

    let mut lines: Vec<Line> = Vec::with_capacity(th);
    for ty in 0..th {
        let sy = (ty as f64 * fh as f64 / th as f64) as usize;
        let sy = sy.min(fh - 1);
        let mut row = String::with_capacity(tw);
        for tx in 0..tw {
            let sx = (tx as f64 * fw as f64 / tw as f64) as usize;
            let sx = sx.min(fw - 1);
            let px = pixels[sy * fw + sx];
            row.push(if px == 1 { '#' } else { ' ' });
        }
        lines.push(Line::from(Span::styled(row, Style::default().fg(p.foam))));
    }

    let para = Paragraph::new(lines).style(Style::default().bg(p.deep));
    f.render_widget(para, inner);
}
