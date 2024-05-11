/*
Sources;
         https://ratatui.rs/,
         https://github.com/ratatui-org/ratatui/blob/main/examples/user_input.rs,
         https://github.com/ratatui-org/ratatui/blob/main/examples/README.md,
         https://ratatui.rs/tutorials/hello-world/,


Events: 3 main ways to handle events; Centrealized event hadnling on event::read(),
    centralized catching, message passing i.e main polling loop,
    Distributed event loops / segmented apps i.e sub looping from main.

Crossterm: https://docs.rs/crossterm/0.27.0/crossterm/
    pure rust, terminal manipulation library. UNIX + windows support.

Raw_Mode: Disable terminal processing and allows us to handle the 'processing'
    kestroks, keyboard control), crossterm is one backend that enables this
    allows for cursor, drawing, and clearing term screen. It is a wrapper of impl write.

Alternate Screen: Swaps from main to alt, self explanatory. Essentially pauses main,
    creates alt, resumes main on alt termination.

Backends: Crossterm, Termion, Termwiz. Crossterm being the 'simplest' and comes with
    windows support. And is currently most commonly used with ratatui.

Ratatui:

Terminal: main interface of lib, hadnles drawing and maint of diff widgets

Frame: consistent view into terminal state for rendering,
    obtained via closure of Terminal::draw, used to render widgets, and control cursor.

Stylize: used for any tyipe that implements Stylize.
    pub trait Stylize<'a, T>: Sized {...}
    let text = "Hello".red().on_blue();
    isntead of
    let text = Span::styled("Hello", Style::default().fg(Color::Red).bg(Color::Blue));

Paragraph: Used to disaplay text
    pub struct Paragraph<'a> { /* private fields */ }

Style shorthands:
    Strings and string slices when styled return a Span
    Spans can be styled again, which will merge the styles.
    Many widget types can be styled directly rather than calling their style() method.

Hello World Guide:
    1. Enter alt screen
    2. Enable Raw mode
    3. create backend and clear screen.
    ...
    n. Terminate Alt Screen, disable raw mode.

*/

/*
To Modify:
    mode picker on: 
        1. Allow for image selection
        2. Re-size image
        3. Crop image
        4. Image conversion
        5. Exiff (Stripper / editor / viewer)
    
    Change bottom window of previous enter strings to be previous comamnds, 
        make it scrollable,
        size it down
    
    add new window for potential options from mode_picker.
    run string on enter through image/ exiff functions.
    start out with file selecter.

*/
use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};

use std::io::{stdout, Result};

enum InputMode {
    Normal,
    Editing,
}
enum ImageMode {
    SelectMode,
    ImagePicker,
    ReSize,
    Crop,
    Conversion,
}

struct App {
    input: String,
    character_index: usize,
    input_mode: InputMode,
    messages: String,
    current_mode: ImageMode,
}

impl App {
    const fn new() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: String::new(),
            character_index: 0,
            current_mode: ImageMode::SelectMode,
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&mut self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);

            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn submit_message(&mut self) {
        self.messages = self.input.clone();
        self.input.clear();
        self.reset_cursor();
    }

}

fn main() -> Result<()> {
    // setup terminal
    //TODO handle err on raw_mode
    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('e') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                },
                InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => app.submit_message(),
                    KeyCode::Char(to_insert) => {
                        app.enter_char(to_insert);
                    }
                    KeyCode::Backspace => {
                        app.delete_char();
                    }
                    KeyCode::Left => {
                        app.move_cursor_left();
                    }
                    KeyCode::Right => {
                        app.move_cursor_right();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                InputMode::Editing => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let vertical = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Min(1),
    ]);
    let [help_area, input_area, messages_area] = vertical.areas(f.size());

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                "Press ".into(),
                "q".bold(),
                " to exit, ".into(),
                "e".bold(),
                " to start editing.".bold(),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                "Press ".into(),
                "Esc".bold(),
                " to stop editing, ".into(),
                "Enter".bold(),
                " to record the message".into(),
            ],
            Style::default(),
        ),
    };

    let text = Text::from(Line::from(msg)).patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, help_area);

    let input = Paragraph::new(app.input.as_str())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::bordered().title("Input"));
    f.render_widget(input, input_area);
    match app.input_mode {
        InputMode::Normal => {}
        InputMode::Editing => {
            #[allow(clippy::cast_possible_truncation)]
            f.set_cursor(
                input_area.x + app.character_index as u16 + 1,
                input_area.y + 1,
            );
        }
    }
    //let message = Paragraph::new(app.messages).block(Block::bordered().title("Messages"));
    let (msg, style) = match app.current_mode {
        ImageMode::SelectMode => (
            vec![
                "Press ".into(),
                "q".bold(),
                " to exit, ".into(),
                "e".bold(),
                " to start editing.".bold(),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        ImageMode::ImagePicker => (
            vec![
                "Press ".into(),
                "Esc".bold(),
                " to stop editing, ".into(),
                "Enter".bold(),
                " to record the message".into(),
            ],
            Style::default(),
        ),
        ImageMode::ReSize => (
            vec![
                "Press ".into(),
                "Esc".bold(),
                " to stop editing, ".into(),
                "Enter".bold(),
                " to record the message".into(),
            ],
            Style::default(),
        ),
        ImageMode::Crop => (
            vec![
                "Press ".into(),
                "Esc".bold(),
                " to stop editing, ".into(),
                "Enter".bold(),
                " to record the message".into(),
            ],
            Style::default(),
        ),
        ImageMode::Conversion => (
            vec![
                "Press ".into(),
                "Esc".bold(),
                " to stop editing, ".into(),
                "Enter".bold(),
                " to record the message".into(),
            ],
            Style::default(),
        ),

    };
    let text = Text::from(Line::from(msg)).patch_style(style);
    f.render_widget(text , messages_area)
}
