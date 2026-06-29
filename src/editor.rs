use crossterm::{
    event::{Event, KeyEvent, KeyEventKind, poll, read},
    style::Attribute,
};
use std::{
    env, io,
    panic::{set_hook, take_hook},
    time::Duration,
};

mod command;
use command::Command;

mod terminal;
use crate::editor::{
    command::System::{self, Quit, Resize},
    terminal::{IoResult, Size},
};
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

const EDITOR_FRAMERATE: u64 = 60;
#[allow(clippy::integer_division)]
const FRAME_DURATION: Duration = Duration::from_millis(1000 / EDITOR_FRAMERATE);

const QUIT_CONFIRM_TIMES: u8 = 3;

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    terminal_size: Size,
    title: String,
    quit_confirm: u8,
}

impl Editor {
    /// creates a new [`Editor`].
    ///
    /// # Errors
    ///
    /// This function will return an error if terminal initialization fails.
    pub fn new() -> io::Result<Editor> {
        let current_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));

        Terminal::initialize()?;

        let mut editor = Self::default();
        let size = Terminal::size().unwrap_or_default();
        editor.resize(size);

        let mut load_result: IoResult = Ok(());
        let args: Vec<String> = env::args().collect();
        if let Some(file_name) = args.get(1) {
            load_result = editor.view.load(file_name);
        }

        editor.message_bar.update_message(&format!(
            "{rev}<C-s>{res} save{sep}{rev}<C-q>{res} quit",
            rev = Attribute::Reverse,
            res = Attribute::Reset,
            sep = " ".repeat(5)
        ));

        if let Err(e) = load_result {
            editor.message_bar.update_error(&format!(
                "Failed to load file {}! {} {}",
                args.get(1).unwrap_or(&String::new()),
                e,
                editor.view.get_file_name()
            ));
        }

        editor.refresh_status();
        return Ok(editor);
    }

    fn resize(&mut self, size: Size) {
        self.terminal_size = size;
        self.status_bar.resize(Size {
            height: 1,
            width: size.width,
        });
        self.message_bar.resize(Size {
            height: 1,
            width: size.width,
        });
        self.view.resize(Size {
            height: size.height.saturating_sub(2),
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
            if poll(FRAME_DURATION).unwrap_or(false) {
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
            if let Ok(command) = Command::try_from(event) {
                self.process_command(command);
            }
        } else {
            #[cfg(debug_assertions)]
            panic!("received and discarded unsupported or non-press event {event:?}");
        }
    }

    fn process_command(&mut self, command: Command) {
        match command {
            Command::System(Quit) => self.handle_quit(),
            Command::System(Resize(size)) => self.resize(size),
            _ => self.reset_quit_times(),
        }

        match command {
            Command::System(Quit | Resize(_)) => {}
            Command::System(System::Save) => self.handle_save(),
            Command::Move(direction) => self.view.handle_move_command(direction),
            Command::Edit(edit) => self.view.handle_edit_command(edit),
        }
    }

    fn refresh_screen(&mut self) {
        if self.terminal_size.height == 0 || self.terminal_size.width == 0 {
            return;
        }

        let _ = Terminal::hide_caret();

        self.message_bar
            .render(self.terminal_size.height.saturating_sub(1));

        if self.terminal_size.height > 1 {
            self.status_bar
                .render(self.terminal_size.height.saturating_sub(2));
        }

        if self.terminal_size.height > 2 {
            self.view.render(0);
        }
        let _ = Terminal::move_caret_to(self.view.caret_pos());
        let _ = Terminal::show_caret();
        let _ = Terminal::flush();
    }

    fn reset_quit_times(&mut self) {
        if self.quit_confirm > 0 {
            self.quit_confirm = 0;
            self.message_bar.update_message("");
        }
    }

    fn handle_quit(&mut self) {
        #![allow(clippy::arithmetic_side_effects)]
        if !self.view.has_unsaved_changed() || self.quit_confirm + 1 == QUIT_CONFIRM_TIMES {
            self.should_quit = true;
        } else if self.view.has_unsaved_changed() {
            self.message_bar.update_message(&format!(
                "WARNING! File has unsaved changes. Press Ctrl-Q {} more times to quit",
                QUIT_CONFIRM_TIMES - self.quit_confirm - 1
            ));

            self.quit_confirm += 1;
        }
    }

    fn handle_save(&mut self) {
        let e = self.view.save();
        if e.is_ok() {
            self.message_bar.update_message("file saved succsefully!");
        } else {
            self.message_bar.update_error(&format!(
                "Error writing to file {}! {}",
                &self.view.get_file_name(),
                e.expect_err("unreachable")
            ));
        }
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
