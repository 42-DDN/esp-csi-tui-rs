use ratatui::{prelude::*, widgets::*};
use crate::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let area = crate::frontend::overlays::help::centered_rect(50, 50, area);
    f.render_widget(Clear, area);

    let items: Vec<ListItem> = app.available_replays
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if i == app.replay_selector_index {
                app.theme.sidebar_selected
            } else {
                app.theme.text_normal
            };
            ListItem::new(format!(" {} ", name)).style(style)
        })
        .collect();

    let title = " Select Replay File ";

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(app.theme.focused_border)
        .style(app.theme.root);

    let list = List::new(items)
        .block(block);

    f.render_widget(list, area);
}
