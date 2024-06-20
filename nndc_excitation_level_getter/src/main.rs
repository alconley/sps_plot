use nndc_excitation_level_getter::nuclear_data_amdc_2016::ISOTOPES;
use nndc_excitation_level_getter::excitation_fetcher::ExcitationFetcher;

fn main() {
    let fetcher = ExcitationFetcher::new();
    match fetcher.process_isotopes(&ISOTOPES) {
        Ok(_) => println!("Excitation levels saved to CSV successfully."),
        Err(e) => eprintln!("Error processing isotopes: {}", e),
    }
}
