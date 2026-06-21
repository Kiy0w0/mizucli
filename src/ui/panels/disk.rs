use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let p = &theme.palette;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border())
        .title(Span::styled(" Disk ", theme.panel_title()))
        .style(theme.panel_bg());
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    let disk_rows = app.metrics.disks.len().max(1);
    let io_height = 4u16;
    let disk_area_h = (inner.height).saturating_sub(io_height);
    let per_disk = if disk_rows > 0 {
        (disk_area_h / disk_rows as u16).max(1)
    } else {
        1
    };

    let mut constraints: Vec<Constraint> = app
        .metrics
        .disks
        .iter()
        .map(|_| Constraint::Length(per_disk))
        .collect();
    constraints.push(Constraint::Min(io_height));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, disk) in app.metrics.disks.iter().enumerate() {
        if i >= chunks.len().saturating_sub(1) {
            break;
        }
        let total = disk.total_bytes.max(1);
        let ratio = disk.used_bytes as f64 / total as f64;
        let pct = (ratio * 100.0).clamp(0.0, 100.0) as u16;
        let used_h = humansize::format_size(disk.used_bytes, humansize::BINARY);
        let total_h = humansize::format_size(disk.total_bytes, humansize::BINARY);
        let label = format!("{} {} {}/{} {}%", disk.name, disk.mount, used_h, total_h, pct);
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(theme.size_color(ratio).bg(p.abyss))
            .percent(pct)
            .label(Span::styled(label, Style::default().fg(p.foam)));
        f.render_widget(gauge, chunks[i]);
    }

    let io_chunk = chunks[chunks.len() - 1];
    if io_chunk.height < 4 {
        return;
    }
    let io_parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(io_chunk);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("▼ rd  ", Style::default().fg(p.cyan)),
            Span::styled(
                format!("{:>8.1} KB/s", app.metrics.disk_read_kbps),
                Style::default().fg(p.foam).add_modifier(Modifier::BOLD),
            ),
        ])),
        io_parts[0],
    );

    let rd_data: Vec<u64> = app
        .history_disk_read
        .iter()
        .map(|v| v.max(0.0) as u64)
        .collect();
    f.render_widget(
        Sparkline::default()
            .data(&rd_data)
            .style(Style::default().fg(p.surf))
            .bar_set(symbols::bar::NINE_LEVELS),
        io_parts[1],
    );

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("▲ wr  ", Style::default().fg(p.cyan)),
            Span::styled(
                format!("{:>8.1} KB/s", app.metrics.disk_write_kbps),
                Style::default().fg(p.foam).add_modifier(Modifier::BOLD),
            ),
        ])),
        io_parts[2],
    );

    let wr_data: Vec<u64> = app
        .history_disk_write
        .iter()
        .map(|v| v.max(0.0) as u64)
        .collect();
    f.render_widget(
        Sparkline::default()
            .data(&wr_data)
            .style(Style::default().fg(p.wave))
            .bar_set(symbols::bar::NINE_LEVELS),
        io_parts[3],
    );
}
