use trekr::app::App;

fn main() {
    let app = App::new();
    println!("{}", app.bootstrap_summary());
}
