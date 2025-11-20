use crate::cmd::console::app::{ChannelsFocus, SelectedTab};
use ratatui::{
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph},
    Frame,
};

/// Renders the bottom controls bar showing context-aware keybindings
pub(crate) fn render_help_bar(
    frame: &mut Frame,
    area: Rect,
    selected_tab: SelectedTab,
    focus: ChannelsFocus,
) {
    let controls_line = if selected_tab == SelectedTab::Channels {
        match focus {
            ChannelsFocus::Channels => Line::from(vec![
                " Navigate ".into(),
                "<←↑↓→/hjkl> ".blue().bold(),
                " | Toggle Logs ".into(),
                "<o> ".blue().bold(),
                " | Pause ".into(),
                "<p> ".blue().bold(),
                " | Quit ".into(),
                "<q> ".blue().bold(),
            ]),
            ChannelsFocus::Logs => Line::from(vec![
                " Navigate ".into(),
                "<←↑↓→/hjkl> ".blue().bold(),
                " | Toggle Logs ".into(),
                "<o> ".blue().bold(),
                " | Pause ".into(),
                "<p> ".blue().bold(),
                " | Inspect ".into(),
                "<i> ".blue().bold(),
                " | Quit ".into(),
                "<q> ".blue().bold(),
            ]),
            ChannelsFocus::Inspect => Line::from(vec![
                " Navigate ".into(),
                "<←↑↓→/hjkl> ".blue().bold(),
                " | Toggle Logs ".into(),
                "<o> ".blue().bold(),
                " | Pause ".into(),
                "<p> ".blue().bold(),
                " | Close ".into(),
                "<i/o/h> ".blue().bold(),
                " | Quit ".into(),
                "<q> ".blue().bold(),
            ]),
        }
    } else {
        Line::from(vec![
            " Navigate ".into(),
            "<↑/k ↓/j> ".blue().bold(),
            " | Toggle Logs ".into(),
            "<o> ".blue().bold(),
            " | Pause ".into(),
            "<p> ".blue().bold(),
            " | Quit ".into(),
            "<q> ".blue().bold(),
        ])
    };

    let block = Block::bordered()
        .title(" Controls ")
        .border_set(border::PLAIN);

    let paragraph = Paragraph::new(controls_line).block(block).left_aligned();

    frame.render_widget(paragraph, area);
}
