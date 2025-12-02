// --- File: src/frontend/overlays/export_data.rs ---
// --- Purpose: Text input popup for exporting data ---

use ratatui::{prelude::*, widgets::*};
use crate::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let area = crate::frontend::overlays::help::centered_rect(50, 20, area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Export Data (CSV) ")
        .borders(Borders::ALL)
        .border_style(app.theme.focused_border)
        .style(app.theme.root);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let instructions = "Enter filename prefix (e.g. 'capture_01')\n\
                        Will be saved as: [prefix]_[timestamp].csv\n\n\
                        [Enter] Export  [Esc] Cancel";

    let text = format!("{}\n\n{}", app.export_input_buffer, instructions);
    let input = Paragraph::new(text)
        .style(app.theme.text_highlight)
        .alignment(Alignment::Center);

    f.render_widget(input, inner);
}
