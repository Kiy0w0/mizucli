mod panels;
pub mod theme;

use crate::app::App;
use crate::config::settings::ThemeName;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Block,
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(f, root[0], app);
    render_body(f, root[1], app);
    render_footer(f, root[2], app);
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let p = &app.theme.palette;
    let cpu = app.metrics.cpu_total;
    let ram_used = humansize::format_size(app.metrics.ram_used_mb as u64 * 1024 * 1024, humansize::BINARY);
    let ram_total = humansize::format_size(app.metrics.ram_total_mb as u64 * 1024 * 1024, humansize::BINARY);
    let header = Block::default()
        .style(Style::default().bg(p.abyss))
        .title(Span::styled(
            format!(
                " 水 mizu  {}  cpu {:.0}%  ram {}/{}  gpu {} ",
                app.metrics.cpu_brand,
                cpu,
                ram_used,
                ram_total,
                app.metrics.gpu_name,
            ),
            Style::default().fg(p.cyan).add_modifier(Modifier::BOLD),
        ));
    f.render_widget(header, area);
}

fn render_body(f: &mut Frame, area: Rect, app: &App) {
    match app.active_tab {
        1 => render_cpu_tab(f, area, app),
        2 => render_net_tab(f, area, app),
        _ => render_overview(f, area, app),
    }
}

fn render_overview(f: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    let left = cols[0];
    let right = cols[1];

    let left_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(left);

    panels::cpu::render(f, left_rows[0], app);
    panels::wave::render(f, left_rows[1], app);

    let right_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Percentage(50),
            Constraint::Min(5),
        ])
        .split(right);

    panels::ram::render(f, right_rows[0], app);
    panels::disk::render(f, right_rows[1], app);
    panels::net::render(f, right_rows[2], app);
}

fn render_cpu_tab(f: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);
    panels::cpu::render(f, rows[0], app);
    panels::ram::render(f, rows[1], app);
}

fn render_net_tab(f: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    panels::net::render(f, rows[0], app);
    panels::disk::render(f, rows[1], app);
}

fn theme_label(name: ThemeName) -> &'static str {
    match name {
        ThemeName::Mizu => "mizu",
        ThemeName::Abyss => "abyss",
        ThemeName::Coral => "coral",
    }
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let p = &app.theme.palette;
    let active = app.active_tab;
    let tabs = [(0u8, "overview"), (1u8, "cpu"), (2u8, "net/disk")];

    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::raw(" "));
    for (i, (idx, label)) in tabs.iter().enumerate() {
        let key = format!("[{}]", idx + 1);
        let label_text = format!(" {} ", label);
        let trailing = if i + 1 < tabs.len() { "  " } else { "" };

        let key_style = if *idx == active {
            Style::default().fg(p.foam).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(p.ripple)
        };
        let label_style = if *idx == active {
            Style::default().fg(p.cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(p.ripple)
        };

        spans.push(Span::styled(key, key_style));
        spans.push(Span::styled(label_text, label_style));
        spans.push(Span::raw(trailing));
    }
    spans.push(Span::styled(
        format!("t:{}  f:flow  ", theme_label(app.theme.name)),
        Style::default().fg(p.ripple),
    ));
    spans.push(Span::styled("q quit", Style::default().fg(p.ripple)));
    spans.push(Span::raw(" "));

    let footer = Block::default()
        .style(Style::default().bg(p.abyss))
        .title(Line::from(spans));
    f.render_widget(footer, area);
}

#[cfg(test)]
mod tests {
    use super::{theme_label, ThemeName};

    #[test]
    fn theme_label_covers_all_variants() {
        assert_eq!(theme_label(ThemeName::Mizu), "mizu");
        assert_eq!(theme_label(ThemeName::Abyss), "abyss");
        assert_eq!(theme_label(ThemeName::Coral), "coral");
    }
}
