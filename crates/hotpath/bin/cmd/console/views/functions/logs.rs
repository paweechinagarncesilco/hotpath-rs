use super::super::super::widgets::formatters::format_time_ago;
use hotpath::{FunctionLogsJson, ProfilingMode};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Cell, HighlightSpacing, List, ListItem, Row, Table, TableState},
    Frame,
};

pub(crate) fn render_function_logs_panel(
    current_function_logs: Option<&FunctionLogsJson>,
    selected_function_name: Option<&str>,
    profiling_mode: &ProfilingMode,
    total_elapsed: u64,
    area: Rect,
    frame: &mut Frame,
    table_state: &mut TableState,
    is_focused: bool,
) {
    let title = if let Some(ref function_logs) = current_function_logs {
        format!(" {} ", function_logs.function_name)
    } else if selected_function_name.is_some() {
        " Loading... ".to_string()
    } else {
        " Recent Logs ".to_string()
    };

    let border_set = if is_focused {
        border::THICK
    } else {
        border::PLAIN
    };

    let block = Block::bordered()
        .border_set(border_set)
        .border_style(if is_focused {
            Style::default()
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));

    if let Some(ref function_logs_data) = current_function_logs {
        let is_alloc_mode = matches!(profiling_mode, &ProfilingMode::Alloc);

        let headers = if is_alloc_mode {
            Row::new(vec![
                Cell::from("Index").style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from("Mem").style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from("Objects").style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from("Ago").style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from("TID").style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        } else {
            Row::new(vec![
                Cell::from("Index").style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from("Latency").style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from("Ago").style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from("TID").style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        };

        let rows: Vec<Row> = function_logs_data
            .logs
            .iter()
            .enumerate()
            .map(|(idx, &(value, elapsed_nanos, count, tid))| {
                let time_ago_str = if total_elapsed >= elapsed_nanos {
                    let nanos_ago = total_elapsed - elapsed_nanos;
                    format_time_ago(nanos_ago)
                } else {
                    "now".to_string()
                };

                if is_alloc_mode {
                    let mem_str = hotpath::format_bytes(value);
                    let obj_str = count.map_or("0".to_string(), |c| c.to_string());

                    Row::new(vec![
                        Cell::from(format!("{}", idx + 1)),
                        Cell::from(mem_str),
                        Cell::from(obj_str),
                        Cell::from(time_ago_str),
                        Cell::from(tid.to_string()),
                    ])
                } else {
                    let time_str = hotpath::format_duration(value);

                    Row::new(vec![
                        Cell::from(format!("{}", idx + 1)),
                        Cell::from(time_str),
                        Cell::from(time_ago_str),
                        Cell::from(tid.to_string()),
                    ])
                }
            })
            .collect();

        let widths = if is_alloc_mode {
            [
                Constraint::Length(7),  // Index column
                Constraint::Min(10),    // Mem column
                Constraint::Length(9),  // Objects column
                Constraint::Length(12), // Ago column
                Constraint::Length(10), // TID column
            ]
            .as_slice()
        } else {
            [
                Constraint::Length(7),  // Index column
                Constraint::Min(15),    // Latency column (flexible)
                Constraint::Length(12), // Ago column
                Constraint::Length(10), // TID column
            ]
            .as_slice()
        };

        let selected_row_style = Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD);

        let table = Table::new(rows, widths)
            .header(headers)
            .block(block)
            .column_spacing(2)
            .row_highlight_style(selected_row_style)
            .highlight_symbol(">> ")
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(table, area, table_state);
    } else if selected_function_name.is_some() {
        // No logs yet
        let items = vec![
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(Span::styled(
                "  Loading logs...",
                Style::default().fg(Color::Gray),
            ))),
        ];
        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    } else {
        // No function selected
        let items = vec![
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(Span::styled(
                "  No function selected",
                Style::default().fg(Color::Gray),
            ))),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(Span::styled(
                "  Navigate the function list to see logs.",
                Style::default().fg(Color::DarkGray),
            ))),
        ];
        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}
