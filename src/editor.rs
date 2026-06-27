use crossterm::event::{Event, KeyEvent, KeyEventKind, read};
use std::{
    env,
    io::Error,
    panic::{set_hook, take_hook},
};
mod editorcommand;
mod terminal;
mod view;
use terminal::Terminal;
use view::View;

use editorcommand::EditorCommand;

pub struct Editor {
    should_quit: bool,
    view: View,
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
        let mut view = View::default();
        let args: Vec<String> = env::args().collect();

        if let Some(file_name) = args.get(1) {
            view.load(file_name);
        }

        return Ok(Self {
            should_quit: false,
            view,
        });
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
                } else {
                    self.view.handle_command(command);
                }
            }
        } else {
            #[cfg(debug_assertions)]
            panic!("Received and discarded unsupported or non-press event. {event:?}");
        }
    }

    fn refresh_screen(&mut self) {
        let _ = Terminal::hide_caret();
        self.view.render();
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
  ,--.'|_                         ,--.'|_                  ,---,  ,--,   ,--.'|_                     
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
