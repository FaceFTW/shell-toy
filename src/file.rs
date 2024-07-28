///Functions for processing files and related things. Also checks environment variables for certain things
use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

use rand::{seq::SliceRandom, thread_rng};

fn get_list_of_cows(path: &PathBuf) -> Result<Vec<String>, io::Error> {
    let mut total_list = vec![];
    let dir_list = fs::read_dir(path)?;
    for entry in dir_list {
        match entry {
            Ok(item) => match item.metadata()?.is_dir() {
                true => total_list.append(get_list_of_cows(&item.path()).unwrap().as_mut()),
                false => {
                    if item.path().extension().unwrap() == "cow" {
                        total_list.push(item.path().to_str().unwrap().to_string());
                    }
                }
            },
            Err(e) => return Err(e),
        }
    }

    Ok(total_list)
}

pub fn choose_random_cow(cow_path: &PathBuf) -> String {
    let mut rng = thread_rng();
    let cow_list = get_list_of_cows(cow_path).expect("Could not open the cow path");

    let chosen_path = cow_list
        .as_slice()
        .choose(&mut rng)
        .expect("List had no elements to choose from!");
    match fs::File::open(chosen_path) {
        Ok(mut file) => {
            let mut cow_str = String::new();
            file.read_to_string(&mut cow_str)
                .expect("Error reading cow string");
            cow_str
        }
        Err(e) => panic!("{e}"),
    }
}

pub fn identify_cow_path() -> PathBuf {
    //Check if we have an environment variable defined:
    let os = std::env::consts::OS;
    if let Ok(val) = std::env::var("COWPATH") {
        PathBuf::from(val.as_str())
    } else if let Ok(val) = std::env::var("COW_PATH") {
        PathBuf::from(val.as_str())
    } else {
        match os{
			"linux" => PathBuf::from("/usr/share/cowsay/cows"),
			_ => panic!("I don't know what the default path for cowfiles is for this OS!.\nPlease provide a COWPATH or COW_PATH environment variable")
		}
    }
}
