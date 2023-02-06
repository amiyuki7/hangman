use hangman::App;

fn main() -> Result<(), Box<dyn ::std::error::Error>> {
    let mut app = App::init();
    app.run()?;
    Ok(())
}
