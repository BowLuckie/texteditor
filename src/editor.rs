use crossterm::event::{Event, KeyEvent, KeyEventKind, read};
use std::{
    env,
    io::Error,
    panic::{set_hook, take_hook},
};

mod editorcommand;
use editorcommand::EditorCommand;

mod terminal;
use crate::editor::terminal::Size;
use terminal::Terminal;

mod view;
use view::View;

mod documentstatus;
use documentstatus::DocumentStatus;

mod statusbar;
use statusbar::StatusBar;

mod fileinfo;

mod uicomponent;
use uicomponent::UiComponent;

mod messagebar;
use messagebar::MessageBar;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    terminal_size: Size,
    title: String,
}

impl Editor {
    /// creates a new [`Editor`].
    ///
    /// # Errors
    ///
    /// This function will return an error if terminal initialization fails.
    pub fn new() -> Result<Self, Error> {
        let current_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));

        Terminal::initialize()?;

        let mut editor = Self::default();
        let size = Terminal::size().unwrap_or_default();
        editor.resize(size);

        let args: Vec<String> = env::args().collect();
        if let Some(file_name) = args.get(1) {
            editor.view.load(file_name);
        }

        editor.refresh_status();
        return Ok(editor);
    }

    fn resize(&mut self, size: Size) {
        self.terminal_size = size;
        self.view.resize(Size {
            height: size.height.saturating_sub(2),
            width: size.width,
        });
        self.status_bar.resize(Size {
            height: 1,
            width: size.width,
        });
    }

    pub fn refresh_status(&mut self) {
        let status = self.view.get_status();
        let title = format!("{} - {}", status.file_name, NAME);
        self.status_bar.update_status(status);

        if title != self.title && Terminal::set_title(&title).is_ok() {
            self.title = title;
        }
    }

    pub fn run(&mut self) {
        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }
            match read() {
                Ok(event) => self.evaluate_event(&event),
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not read event: {err:?}");
                    }
                }
            }
            self.status_bar.update_status(self.view.get_status());
        }
    }

    fn evaluate_event(&mut self, event: &Event) {
        let should_process = match event {
            Event::Key(KeyEvent { kind, .. }) => {
                if *kind == KeyEventKind::Release {
                    return;
                }
                true
            }
            Event::Resize(_, _) => true,
            _ => false,
        };

        if should_process {
            if let Ok(command) = EditorCommand::try_from(event) {
                if let EditorCommand::Quit = command {
                    self.should_quit = true;
                } else if let EditorCommand::Resize(size) = command {
                    self.resize(size);
                } else {
                    self.view.handle_command(command);
                }
            }
        } else {
            #[cfg(debug_assertions)]
            panic!("received and discarded unsupported or non-press event {event:?}");
        }
    }

    fn refresh_screen(&mut self) {
        if self.terminal_size.height == 0 || self.terminal_size.width == 0 {
            return;
        }

        let _ = Terminal::hide_caret();
        self.status_bar
            .render(self.terminal_size.height.saturating_sub(2));
        if self.terminal_size.height > 2 {
            self.view.render(0);
        }
        let _ = Terminal::move_caret_to(self.view.caret_pos());
        let _ = Terminal::show_caret();
        let _ = Terminal::flush();
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print("\r\nthank you for using the\r\n");
            let _ = Terminal::print(
                r"
    ___                             ___                                    ___                       
  ,--.'|_                         ,--.'|_                  ,---,  ,--,   ,--.'|_          tm         
  |  | :,'                        |  | :,'               ,---.'|,--.'|   |  | :,'   ,---.    __  ,-. 
  :  : ' :            ,--,  ,--,  :  : ' :               |   | :|  |,    :  : ' :  '   ,'\ ,' ,'/ /| 
.;__,'  /     ,---.   |'. \/ .`|.;__,'  /     ,---.      |   | |`--'_  .;__,'  /  /   /   |'  | |' | 
|  |   |     /     \  '  \/  / ;|  |   |     /     \   ,--.__| |,' ,'| |  |   |  .   ; ,. :|  |   ,' 
:__,'| :    /    /  |  \  \.' / :__,'| :    /    /  | /   ,'   |'  | | :__,'| :  '   | |: :'  :  /   
  '  : |__ .    ' / |   \  ;  ;   '  : |__ .    ' / |.   '  /  ||  | :   '  : |__'   | .; :|  | '    
  |  | '.'|'   ;   /|  / \  \  \  |  | '.'|'   ;   /|'   ; |:  |'  : |__ |  | '.'|   :    |;  : |    
  ;  :    ;'   |  / |./__;   ;  \ ;  :    ;'   |  / ||   | '/  '|  | '.'|;  :    ;\   \  / |  , ;    
  |  ,   / |   :    ||   :/\  \ ; |  ,   / |   :    ||   :    :|;  :    ;|  ,   /  `----'   ---'     
   ---`-'   \   \  / `---'  `--`   ---`-'   \   \  /  \   \  /  |  ,   /  ---`-'                     
             `----'                          `----'    `----'    ---`-'                              

",
            );
        }
    }
}
