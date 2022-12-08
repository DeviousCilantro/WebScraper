use regex::Regex;
use reqwest::StatusCode;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

mod utils;
mod models;

#[tokio::main]
async fn main() {
    let mut data_list: HashMap<String, models::Data> = HashMap::new();
    for k in 1..=2 {
		for i in 1..=32000 {
			let client = utils::get_client();
            let mut url;
			let name_selector = Selector::parse("h1.name").unwrap();
			let date_selector = Selector::parse("p.date").unwrap();
			let bio_selector = Selector::parse("div.indnotes").unwrap();
			let regex_1 = Regex::new(r"<[^>]*>").unwrap();
			let regex_2 = Regex::new(r"<[^>]*>|\n|\t").unwrap();
			let table_selector = Selector::parse("table.full > tbody > tr > td > div.columns").unwrap();
			let mut name = String::new();
			let mut date = String::new();
			let mut bio = String::new();
			let mut src = String::new();
			let mut link = String::new();
			let mut rows: HashMap<String, String> = HashMap::new();
			let mut count: u32 = 0;
			let mut key = String::new();
			let mut value;
            if k == 1 {
                url = String::from("https://www.ucl.ac.uk/lbs/person/view/");
            } else {
                url = String::from("https://www.ucl.ac.uk/lbs/claim/view/");
            }
			url.push_str(&i.to_string());
            println!("{}",&url);
			let result = client.get(&url).send().await.unwrap();
			let raw_html = match result.status() {
				StatusCode::OK => result.text().await.unwrap(),
				_ => panic!("Something went wrong"),
			};

			let document = Html::parse_document(&raw_html);
			for element in document.select(&name_selector) {
				let inner = element.inner_html().to_string();
				name = regex_1.replace_all(&inner, "").to_string();
			}

			for element in document.select(&date_selector) {
				let inner = element.inner_html().to_string();
				date = regex_1.replace_all(&inner, "").to_string();
			}

			for element in document.select(&bio_selector) {
				let inner = element.inner_html().to_string();
				let replaced = regex_1.replace_all(&inner, "");
				match replaced.split_once("Sources") {
					Some((biography, sources)) => {
						bio = biography.to_string();
						src = sources.to_string();
					}
					None => {}
				}
			}
			for element in document.select(&table_selector) {
				let inner = element.inner_html().to_string();
				if count % 2 == 0 {
					key = regex_2.replace_all(&inner, "").to_string();
					if inner.contains("href") {
						let start_bytes = inner.find("/lbs").unwrap_or(0);
						let end_bytes = inner[start_bytes..].find("\"").unwrap_or(inner.len());
						link.push_str("https://www.ucl.ac.uk");
						link.push_str(&inner[start_bytes..][..end_bytes]);
						let cloned_link = link.clone();
						key.push_str("::");
						key.push_str(&cloned_link);
						link = String::new();
					}

					count += 1;
					continue;
				} else {
					value = regex_2.replace_all(&inner, "").to_string();
					count += 1;
				}
				let cloned_key = key.clone();
				let cloned_value = value.clone();
				if inner.contains("href") {
					let start_bytes = inner.find("/lbs").unwrap_or(0);
					let end_bytes = inner[start_bytes..].find("\"").unwrap_or(inner.len());
					link.push_str("https://www.ucl.ac.uk");
					link.push_str(&inner[start_bytes..][..end_bytes]);
					let cloned_link = link.clone();
					rows.insert(String::from(cloned_key), cloned_link);
					link = String::new();
					continue;
				}
				rows.insert(String::from(cloned_key), cloned_value);
			}
			data_list.insert(i.to_string(), models::Data {
				id: url[(url.rfind('/').unwrap_or(0)+1)..].to_string(),
				name: name,
				date: date,
				biography: bio,
				sources: src,
				rows: rows,
			});
		}
        if k == 1 {
            save_data_list(&data_list, "person");
            data_list = HashMap::new();
        } else { 
            save_data_list(&data_list, "claim");
            data_list = HashMap::new();
        }
    }
}

fn save_data_list(data_list: &HashMap<String, models::Data>, which_type: &str) {
    let filename = format!("{}.json", which_type);
    let mut writer = File::create(&filename).unwrap();
    write!(
        &mut writer,
        "{}",
        &serde_json::to_string_pretty(&data_list).unwrap()
    )
    .unwrap();
}
