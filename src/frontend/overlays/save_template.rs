// --- File: src/frontend/overlays/save_template.rs ---
// --- Purpose: Text input popup for naming a template ---

use ratatui::{prelude::*, widgets::*};
use crate::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let area = crate::frontend::overlays::help::centered_rect(40, 20, area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Save Template As... ")
        .borders(Borders::ALL)
        .border_style(app.theme.focused_border)
        .style(app.theme.root);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = format!("{}\n\n[Enter] Save  [Esc] Cancel", app.input_buffer);
    let input = Paragraph::new(text)
        .style(app.theme.text_highlight)
        .alignment(Alignment::Center);

    f.render_widget(input, inner);
}