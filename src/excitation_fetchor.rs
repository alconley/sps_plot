use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::error::Error;

// use std::sync::mpsc::{self, Receiver, Sender};
use tokio::runtime::Runtime; // Tokio for async runtime... doesnt work on wasm 

#[derive(Debug, Serialize, Deserialize)]
pub struct ExcitationResponse {
    excitation_levels: Vec<f64>,
}

pub struct ExcitationFetcher {
    pub excitation_levels: Arc<Mutex<Option<Vec<f64>>>>,
    pub error_message: Arc<Mutex<Option<String>>>,
}

impl ExcitationFetcher {
    pub fn new() -> Self {
        Self {
            excitation_levels: Arc::new(Mutex::new(None)),
            error_message: Arc::new(Mutex::new(None)),
        }
    }

    pub fn fetch_excitation_levels(&self, isotope: &str) {
        let rt = Runtime::new().unwrap();
        let excitation_levels_clone = Arc::clone(&self.excitation_levels);
        let error_message_clone = Arc::clone(&self.error_message);
        let isotope = isotope.to_string();
    
        // Use the runtime to block on the async function
        rt.block_on(async {
            // Corrected method name in the call
            let result = self.get_excitations(&isotope).await;
            match result {
                Ok(levels) => {
                    let mut excitation_levels = excitation_levels_clone.lock().unwrap();
                    *excitation_levels = Some(levels);
                },
                Err(e) => {
                    let mut error_message = error_message_clone.lock().unwrap();
                    *error_message = Some(e.to_string());
                }
            }
        });
    }

    pub async fn get_excitations(&self, isotope: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        // Asynchronously fetch the webpage content
        let url = format!("https://www.nndc.bnl.gov/nudat3/getdatasetClassic.jsp?nucleus={}&unc=nds", isotope);
        let site_content = reqwest::get(&url).await?.text().await?;
        
        // Parse the HTML document
        let document = Html::parse_document(&site_content);
        
        let table_selector = Selector::parse("table").unwrap();
        
        // Attempt to select the specific table
        let tables = document.select(&table_selector).collect::<Vec<_>>();
        if tables.len() < 3 {
            return Err("Table not found or doesn't contain enough data".into());
        }
        
        // Prepare regex for cleaning and extracting numerical values
        let re_clean = Regex::new(r"\s*(\d+(\.\d+)?(E[+\-]?\d+)?)\s*")?;
        
        // Initialize a vector to hold the energy levels
        let mut levels = Vec::new();
        
        // Iterate over table rows, skipping the first header row
        for row in tables[2].select(&Selector::parse("tr").unwrap()).skip(1) {
            let entries = row.select(&Selector::parse("td").unwrap()).collect::<Vec<_>>();
            if !entries.is_empty() {
                let entry = &entries[0];
                let text = entry.text().collect::<Vec<_>>().join("");
                if let Some(caps) = re_clean.captures(&text) {
                    if let Some(matched) = caps.get(1) {
                        let cleaned_text = matched.as_str();                        
                        match cleaned_text.parse::<f64>() {
                            Ok(num) => {
                                // Convert to MeV and format to 3 decimal places
                                let formatted_num = format!("{:.3}", num / 1000.0);
                                match formatted_num.parse::<f64>() {
                                    Ok(formatted_num) => levels.push(formatted_num),
                                    Err(_) => continue, // Skip entries that can't be formatted/parsed as f64
                                }
                            },
                            Err(_) => continue, // Skip entries that can't be parsed as f64
                        }
                    }
                }
            }
        }
        
        Ok(levels)
    }
}
