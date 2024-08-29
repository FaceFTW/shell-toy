use core::panic;
use std::{
    fs::File,
    io::{BufReader, Read},
};

use fs_extra::copy_items;
use serde::{de, Deserialize};

//This will likely always trigger because it just affects "pre-"compile time and not runtime
fn main() -> Result<(), std::io::Error> {
    let config: BuildConfig = get_config()?;
    //Download Resources
    get_source_archive(&config.cowsay.url, "cowsay")?;
    get_source_archive(&config.fortune_mod.url, "fortune")?;

    extract_resources(
        "target/downloads/cowsay.zip",
        &config.cowsay.internal_path,
        "target/resources/cowsay",
    )?;

    #[cfg(feature = "inline-fortune")]
    create_fortune_db()?;

    #[cfg(feature = "inline-cowsay")]
    generate_cowsay_source()?;

    Ok(())
}

#[cfg(feature = "inline-fortune")]
fn create_fortune_db() -> Result<(), std::io::Error> {
    use std::{
        ffi::OsStr,
        fs::{self, File},
        io::{self, Read, Write},
        path::PathBuf,
    };

    /***************************************
     * Function Definitions (Because this is easier to fold)
     ***************************************/
    fn gen_concat_fortune_files(val: String) -> Result<(), io::Error> {
        println!("cargo::rerun-if-changed={val}");
        let (fortune_list, offensive_list) = fortune_list_iterate(&PathBuf::from(val), false);
        let concat_fortunes = concat_fortune_files(fortune_list.as_slice())?.replace("\r\n", "\n");
        let off_concat_fortunes =
            concat_fortune_files(offensive_list.as_slice())?.replace("\r\n", "\n");
        println!("{:#?}", offensive_list.as_slice());
        match File::create("target/resources/fortunes") {
            Ok(mut file) => {
                let _ = file.write_all(concat_fortunes.trim().as_bytes())?;
            }
            Err(err) => panic!("Could not concatenate fortunes into single file: {err}"),
        }
        match File::create("target/resources/off_fortunes") {
            Ok(mut file) => {
                let _ = file.write_all(off_concat_fortunes.trim().as_bytes())?;
            }
            Err(err) => panic!("Could not concatenate fortunes into single file: {err}"),
        }
        Ok(())
    }

    fn fortune_list_iterate(path: &PathBuf, is_offensive: bool) -> (Vec<PathBuf>, Vec<PathBuf>) {
        let illegal_file_suffixes: [&OsStr; 16] = [
            OsStr::new("dat"),
            OsStr::new("pos"),
            OsStr::new("c"),
            OsStr::new("h"),
            OsStr::new("p"),
            OsStr::new("i"),
            OsStr::new("f"),
            OsStr::new("pas"),
            OsStr::new("ftn"),
            OsStr::new("ins.c"),
            OsStr::new("ins.pas"),
            OsStr::new("ins.ftn"),
            OsStr::new("sml"),
            OsStr::new("sh"),
            OsStr::new("pl"),
            OsStr::new("csv"),
        ];

        let mut fortune_list = vec![];
        let mut offensive_list = vec![];
        let dir_list = fs::read_dir(path).expect("Could not open directory");
        for entry in dir_list.filter(|item| {
            !illegal_file_suffixes.contains(
            &item
                .as_ref()
                .unwrap()
                .path()
                .extension()
                .unwrap_or_default(),
        ) &&
		//Additional condition to ignore the CMakeLists.txt file specifically in fortune-mod
		!item.as_ref().unwrap().path().ends_with("CMakeLists.txt")
        }) {
            match entry {
                Ok(item) => match item.metadata().unwrap().is_dir() {
                    true => {
                        if item.file_name() == "off" {
                            offensive_list.append(&mut fortune_list_iterate(&item.path(), true).1)
                        } else {
                            let vecs = &mut fortune_list_iterate(&item.path(), is_offensive);
                            fortune_list.append(&mut vecs.0);
                            offensive_list.append(&mut vecs.1);
                        }
                    }
                    false => match is_offensive {
                        true => offensive_list.push(item.path()),
                        false => fortune_list.push(item.path()),
                    },
                },
                Err(e) => panic!("Could not identify a file in the fortune directory {e}"),
            }
        }
        (fortune_list, offensive_list)
    }

    fn concat_fortune_files(list: &[PathBuf]) -> Result<String, io::Error> {
        let mut buffer = String::new();
        for path in list {
            match File::open(path) {
                Ok(mut file) => {
                    let _ = file.read_to_string(&mut buffer)?;
                }
                Err(err) => panic!("Could not open file: {err}"),
            }
        }
        Ok(buffer)
    }

    /***************************************
     * The actual build steps
     ***************************************/
    if let Err(_) = fs::read_dir("target/resources") {
        fs::create_dir("target/resources")?
    }

    cfg_if::cfg_if! {
        if #[cfg(feature="fortune-git")]{
            println!("cargo::rerun-if-changed={val}");
            gen_concat_fortune_files("./fortunes/fortune-mod/datfiles".to_string())
        } else {
            if let Ok(val) = std::env::var("FORTUNE_FILE") {
                println!("cargo::rerun-if-changed={val}");
                let _ = fs::copy(val, "target/resources/fortunes")?;
                Ok(())
            } else if let Ok(val) = std::env::var("FORTUNE_PATH") {
                println!("cargo::rerun-if-changed={val}");
                gen_concat_fortune_files(val)
            } else if let Ok(val) = std::env::var("FORTUNEPATH") {
                println!("cargo::rerun-if-changed={val}");
                gen_concat_fortune_files(val)
            } else {
                match std::env::consts::OS{
                    //It's a longshot but you never know
                    "linux" => {
                        println!("cargo::rerun-if-changed=/usr/share/games/fortunes");
                        gen_concat_fortune_files(String::from("/usr/share/games/fortunes"))
                    },
                    _ => panic!("I don't know what the default path for fortunes are for this OS!.\nPlease provide a FORTUNEPATH or FORTUNE_PATH environment variable, or a single file with FORTUNE_FILE")
                }
            }
        }
    }
}

#[cfg(feature = "inline-cowsay")]
fn generate_cowsay_source() -> Result<(), std::io::Error> {
    use std::{
        collections::HashMap,
        fs::{self, File},
        io::{self, Read, Write},
        path::PathBuf,
    };
    /***************************************
     * Function Definitions (Because this is easier to fold)
     ***************************************/
    fn identify_cow_path() -> PathBuf {
        cfg_if::cfg_if! {
            if #[cfg(feature="cowsay-git")]{
                println!("cargo::rerun-if-changed=./cowsay/share/cowsay/cows");
                PathBuf::from("./cowsay/share/cowsay/cows")
            } else {
                //Check if we have an environment variable defined:
                let os = std::env::consts::OS;
                if let Ok(val) = std::env::var("COWPATH") {
                    println!("cargo::rerun-if-changed={val}");
                    PathBuf::from(val.as_str())
                } else if let Ok(val) = std::env::var("COW_PATH") {
                    println!("cargo::rerun-if-changed={val}");
                    PathBuf::from(val.as_str())
                } else {
                    match os{
                        "linux" => {
                            println!("cargo::rerun-if-changed=/usr/share/cowsay/cows");
                            PathBuf::from("/usr/share/cowsay/cows")
                        },
                        _ => panic!("I don't know what the default path for cowfiles is for this OS!.\nPlease provide a COWPATH or COW_PATH environment variable")
                    }
                }
            }
        }
    }

    fn get_cow_data(path: &PathBuf) -> Result<HashMap<String, String>, io::Error> {
        let mut total_list: HashMap<String, String> = HashMap::new();
        let dir_list = fs::read_dir(path)?;
        for entry in dir_list {
            match entry {
                Ok(item) => match item.metadata()?.is_dir() {
                    true => {
                        let _ = get_cow_data(&item.path())?.iter_mut().map(|(k, v)| {
                            total_list.insert(k.clone(), v.clone());
                        });
                    }
                    false => {
                        if item.path().extension().unwrap() == "cow" {
                            let key = item.file_name().to_str().unwrap().to_string();
                            match File::open(item.path()) {
                                Ok(mut file) => {
                                    let mut value = String::new();
                                    let _ = file.read_to_string(&mut value)?;
                                    total_list.insert(key, value);
                                }
                                Err(e) => {
                                    panic!(
                                        "Could not open a cow for inlining: {} {e}",
                                        item.path().display()
                                    )
                                }
                            }
                        }
                    }
                },
                Err(e) => return Err(e),
            }
        }

        Ok(total_list)
    }

    fn make_source(
        cow_data: HashMap<String, String>,
    ) -> Result<proc_macro2::TokenStream, io::Error> {
        let keys = cow_data.keys().into_iter().map(|key| key.clone());
        let vals = cow_data
            .keys()
            .into_iter()
            .map(|key| cow_data.get(key).unwrap().clone());
        let len = keys.len();

        let test = quote::quote! {
            const COW_DATA: [(&'static str, &'static str); #len] = [
                #( (#keys, #vals) ),*
            ];
        };

        Ok(test)
    }

    /***************************************
     * Actual Start of Build Steps
     ***************************************/
    if let Err(_) = fs::read_dir("target/generated_sources") {
        fs::create_dir("target/generated_sources")?
    }

    let cowpath = identify_cow_path();
    let cow_data = get_cow_data(&cowpath)?;
    let tokenstream = make_source(cow_data)?;

    match File::create("target/generated_sources/cow_literals.rs") {
        Ok(mut file) => {
            let _ = file.write_all(tokenstream.to_string().as_bytes())?;
        }
        Err(err) => panic!("Could not concatenate fortunes into single file: {err}"),
    }

    Ok(())
}

/************************************************/
/**************Resource Functions****************/
/************************************************/
///Function to download a file via a synchronous call to a process
/// specific to the OS.
///
/// This function will always expect a ZIP file to be downloaded, and it will be
/// to `/target/downloads`
fn get_source_archive(path: &str, resource_name: &str) -> Result<(), std::io::Error> {
    let downloads_path = String::from("target/downloads");
    println!("cargo::rerun-if-changed=target/downloads/cowsay.zip");
    if let Err(_) = std::fs::read_dir(downloads_path.as_str()) {
        std::fs::create_dir(downloads_path.as_str())?
    }

    let os = std::env::consts::FAMILY;
    let mut proc = match os {
        "windows" => {
            let mut p = std::process::Command::new("powershell.exe");
            p.args(&[
                "-NoLogo",
                "-NoProfile",
                "-Command",
                format!("Invoke-RestMethod {path} -OutFile {downloads_path}/{resource_name}.zip")
                    .as_str(),
            ]);
            p
        },
        "unix" => {
            let mut p = std::process::Command::new("curl");
            p.args(&[
                path,
                "--output",
                format!("{downloads_path}/{resource_name}.zip").as_str()
            ]);
            p
        },
        _ => panic!("Are you being special and building this on a non-standard operating system. \
        Good for you. But I can't figure out what command to use for downloading files. \
        Consider modifying the get_external_resource function in build.rs since you are similar enough to an Arch Linux user :p")
    };
    proc.spawn()?.wait()?;

    Ok(())
}

fn extract_resources(
    archive: &str,
    internal_path: &str,
    destination: &str,
) -> Result<(), std::io::Error> {
    if let Err(_) = std::fs::read_dir("target/tmp") {
        std::fs::create_dir("target/tmp")?
    }
    if let Err(_) = std::fs::read_dir("target/resources") {
        std::fs::create_dir("target/resources")?
    }
    if let Err(_) = std::fs::read_dir(destination) {
        std::fs::create_dir(destination)?
    }

    let archive_file = match File::open(archive) {
        Ok(file) => BufReader::new(file),
        Err(_) => panic!("Could not doo this"),
    };
    let mut zip_archive = zip::ZipArchive::new(archive_file)?;
    zip_archive.extract("target/tmp")?;

    let resource_path = format!("target/tmp/{internal_path}");
    let copy_opts = fs_extra::dir::CopyOptions::new().overwrite(true);
    let _ = copy_items(&[resource_path], destination, &copy_opts)
        .expect("Could not copy resources as expected!");

    Ok(())
}
/************************************************/
/**************Configuration Functions***********/
/************************************************/
#[derive(Deserialize)]
struct ResourceConfig {
    #[serde(rename = "source-zip-url")]
    pub url: String,
    #[serde(rename = "resource-location")]
    pub internal_path: String,
}

#[derive(Deserialize)]
struct BuildConfig {
    pub cowsay: ResourceConfig,
    #[serde(rename = "fortune-mod")]
    pub fortune_mod: ResourceConfig,
}

fn get_config() -> Result<BuildConfig, std::io::Error> {
    println!("cargo::rerun-if-changed=./BuildConfig.toml");
    match std::fs::File::open("./BuildConfig.toml") {
        Ok(mut file) => {
            let mut buf = String::new();
            let _ = file.read_to_string(&mut buf);
            Ok(
                toml::from_str(buf.as_str())
                    .expect("BuildConfig.toml was in an unexpected format!"),
            )
        }
        Err(_) => {
            panic!("Could not open the BuildConfig.toml in repository root. Did something happen?")
        }
    }
}
