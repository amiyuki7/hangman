use colored::Colorize;
use std::{
    collections::HashSet,
    io::{self, prelude::*},
};

mod consts;
use consts::*;

enum HangmanState {
    Menu,
    Playing,
}

struct GameState {
    hp: u8,
    secret: String,
    guessed: HashSet<char>,
}

impl GameState {
    fn init(secret: String) -> Self {
        Self {
            hp: 6,
            secret,
            guessed: HashSet::new(),
        }
    }
}

struct WinLoss {
    win: usize,
    loss: usize,
}

impl WinLoss {
    fn new() -> Self {
        Self { win: 0, loss: 0 }
    }

    /// Returns a stringified session win rate
    fn winrate(&self) -> String {
        let total = self.win + self.loss;

        if total == 0 {
            return "> No games played yet".into();
        }

        format!(
            "> Won {} / {} ({:.2}%)",
            self.win,
            total,
            (self.win as f32 / total as f32) * 100f32
        )
    }
}

pub struct App {
    state: HangmanState,
    playstate: Option<GameState>,
    winloss: WinLoss,
}

impl App {
    pub fn init() -> Self {
        Self {
            state: HangmanState::Menu,
            playstate: None,
            winloss: WinLoss::new(),
        }
    }

    /// Pretty prints the game state and handles when the game ends
    fn display(&mut self) {
        let GameState { hp, secret, guessed } = &self.playstate.as_ref().unwrap();

        let frame = HANGMAN_FRAMES[hp.abs_diff(6) as usize];

        let word: String = secret
            .chars()
            .map(|c| if guessed.contains(&c) { c } else { '_' })
            .collect();

        println!("{frame}");
        println!("{word} ({} letters)", secret.len());

        // Dead
        if *hp == 0 {
            println!(
                "{}{}",
                "> Game over! The word was ".blue().italic(),
                secret.yellow().bold().italic()
            );
            self.winloss.loss += 1;
            self.state = HangmanState::Menu;
            self.playstate = None;
            return;
        }

        // All letters in the word have been guessed correctly
        if word == *secret {
            println!(
                "{}{}",
                "> Well done! The word was ".blue().italic(),
                secret.yellow().bold().italic()
            );
            self.winloss.win += 1;
            self.state = HangmanState::Menu;
            self.playstate = None;
        }
    }

    /// Inserts `c` into a list of guessed characters; returns whether the guess was valid, ie:
    /// * `false` if the guess is not new
    /// * `true` if the guess is new
    fn handle_guess(&mut self, c: char) -> bool {
        if !self.playstate.as_mut().unwrap().guessed.insert(c) {
            println!(
                "{}{}{}",
                "> ".blue().italic(),
                format!("'{c}'").yellow().bold().italic(),
                " has already been guessed".blue().italic()
            );
            return false;
        }

        if let Some(ref mut playstate) = self.playstate {
            if !playstate.secret.contains(c) {
                playstate.hp -= 1;
            }
        }

        true
    }

    /// Tries to initialise a game
    fn play(&mut self) {
        println!("{}", "> Fetching a word...".blue().italic());
        let word = Self::get_word();

        if let Ok(s) = word {
            println!("{}", "> Found a word!".blue().italic());
            self.state = HangmanState::Playing;
            self.playstate = Some(GameState::init(s));
            self.display();
        } else {
            println!(
                "{}",
                "> Couldn't find a word... try checking your internet connection?"
                    .blue()
                    .italic()
            );
        }
    }

    /// Blocking call which returns a random english word
    fn get_word() -> reqwest::Result<String> {
        let mut response =
            reqwest::blocking::get("https://random-word-api.herokuapp.com/word")?.json::<Vec<String>>()?;

        Ok(response.remove(0).to_lowercase())
    }

    /// Returns the recent line of CLI user input
    fn input(prompt: &str) -> io::Result<String> {
        print!("{prompt}");
        io::stdout().flush()?;
        io::stdin()
            .lock()
            .lines()
            .next()
            .unwrap()
            .map(|x| x.trim().to_owned())
    }

    /// Game loop
    pub fn run(&mut self) -> io::Result<()> {
        println!(
            "\n{}{}{}",
            "> Welcome to Hangman! View a list of commands with "
                .blue()
                .italic(),
            "help".magenta().bold().italic(),
            "\n> NB: Internet connection is required and disabling font ligatures is recommended"
                .blue()
                .italic()
        );

        loop {
            let command = Self::input("\n> ")?;

            match self.state {
                HangmanState::Menu => {
                    let valid = MENU_COMMANDS.contains(&&command[..]);

                    if !valid {
                        continue;
                    }

                    // Handle Menu commands
                    match &command[..] {
                        "play" => {
                            self.play();
                        }
                        "quit" => break Ok(()),
                        "help" => {
                            println!(
                                "{}{}{}\n{}{}{}\n{}{}{}\n{}{}{}",
                                "> ".blue().italic(),
                                "help".magenta().bold().italic(),
                                " => Show this list".blue().italic(),
                                "> ".blue().italic(),
                                "play".magenta().bold().italic(),
                                " => Start a game".blue().italic(),
                                "> ".blue().italic(),
                                "quit".magenta().bold().italic(),
                                " => Quit the process".blue().italic(),
                                "> ".blue().italic(),
                                "winr".magenta().bold().italic(),
                                " => Session win rate".blue().italic(),
                            );
                        }
                        "winr" => {
                            println!("{}", self.winloss.winrate().blue().italic())
                        }
                        _ => {}
                    }

                    continue;
                }
                HangmanState::Playing => {
                    let valid = command.len() == 1 && command.chars().next().unwrap().is_ascii_alphabetic();

                    if !valid {
                        println!("{}", "> Invalid guess".blue().italic());
                        continue;
                    }

                    // Turn character to lowercase; unwrap is explicitly used because to_lowercase() won't return
                    // multiple characters
                    let guess: char = command
                        .chars()
                        .next()
                        .unwrap()
                        .to_lowercase()
                        .next()
                        .unwrap();

                    // Only redraw the hangman and word if the guess is new
                    if self.handle_guess(guess) {
                        self.display();
                    }
                }
            };
        }
    }
}
