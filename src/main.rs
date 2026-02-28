use trekr::app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();
    println!("{}", app.bootstrap_summary());
    app.run()
}
