use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let p = &theme.palette;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border())
        .title(Span::styled(" Network ", theme.panel_title()))
        .style(theme.panel_bg());
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 4 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(inner);

    let rx_now = app.metrics.net_rx_kbps;
    let tx_now = app.metrics.net_tx_kbps;

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("▼ rx  ", Style::default().fg(p.cyan)),
            Span::styled(
                format!("{:>8.1} KB/s", rx_now),
                Style::default().fg(p.foam).add_modifier(Modifier::BOLD),
            ),
        ])),
        chunks[0],
    );

    let rx_data: Vec<u64> = app.history_net_rx.iter().map(|v| v.max(0.0) as u64).collect();
    f.render_widget(
        Sparkline::default()
            .data(&rx_data)
            .style(Style::default().fg(p.surf))
            .bar_set(symbols::bar::NINE_LEVELS),
        chunks[1],
    );

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("▲ tx  ", Style::default().fg(p.cyan)),
            Span::styled(
                format!("{:>8.1} KB/s", tx_now),
                Style::default().fg(p.foam).add_modifier(Modifier::BOLD),
            ),
        ])),
        chunks[2],
    );

    let tx_data: Vec<u64> = app.history_net_tx.iter().map(|v| v.max(0.0) as u64).collect();
    f.render_widget(
        Sparkline::default()
            .data(&tx_data)
            .style(Style::default().fg(p.wave))
            .bar_set(symbols::bar::NINE_LEVELS),
        chunks[3],
    );
}
