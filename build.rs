use std::{
    // cell::RefCell,
    collections::HashMap,
    ffi::OsStr,
    fs::{self, remove_file, File},
    io::{self, BufReader, Read, Write},
    path::PathBuf,
};

///Checks if an environment variable exists.
macro_rules! check_env_flag {
    ($name:literal) => {
        match std::env::var($name) {
            Ok(_) => true,
            Err(_) => false,
        }
    };
}

//This will likely always trigger because it just affects "pre-"compile time and not runtime
fn main() -> Result<(), std::io::Error> {
    //Here we get a bunch of the env flags we need for evaluating what features are enabled
    let inline_fortune_flag = check_env_flag!("CARGO_FEATURE_INLINE_FORTUNE");
    println!("cargo::rerun-if-env-changed=CARGO_FEATURE_INLINE_FORTUNE");
    println!("cargo::rerun-if-env-changed=CARGO_FEATURE_INLINE_OFF_FORTUNE");
    let inline_cowsay_flag = check_env_flag!("CARGO_FEATURE_INLINE_COWSAY");
    println!("cargo::rerun-if-env-changed=CARGO_FEATURE_INLINE_COWSAY");
    let force_download_flag = check_env_flag!("FORCE_DOWNLOAD");
    println!("cargo::rerun-if-env-changed=FORCE_DOWNLOAD");
    let use_default_flag = check_env_flag!("USE_DEFAULT_RESOURCES");
    println!("cargo::rerun-if-env-changed=USE_DEFAULT_RESOURCES");
    let cow_path_exists = check_env_flag!("COW_PATH");
    println!("cargo::rerun-if-env-changed=COW_PATH");
    let fortune_file_exists = check_env_flag!("FORTUNE_FILE");
    println!("cargo::rerun-if-env-changed=FORTUNE_FILE");
    let fortune_path_exists = check_env_flag!("FORTUNE_PATH");
    println!("cargo::rerun-if-env-changed=FORTUNE_PATH");

    let config: BuildConfig = get_config()?;
    println!("{:#?}", &config);

    //Download Resources
    if inline_cowsay_flag {
        if use_default_flag || !cow_path_exists {
            get_source_archive(&config.cowsay.url, "cowsay", force_download_flag)?;
            extract_resources(
                "target/downloads/cowsay.zip",
                &config.cowsay.internal_path,
                "target/resources/cowsay",
                &config.cowsay.exclude,
            )?;
        }
        generate_cowsay_source()?;
    }

    if inline_fortune_flag {
        if use_default_flag || (!fortune_file_exists && !fortune_path_exists) {
            get_source_archive(&config.fortune_mod.url, "fortune", force_download_flag)?;
            extract_resources(
                "target/downloads/fortune.zip",
                &config.fortune_mod.internal_path,
                "target/resources/fortune",
                &config.fortune_mod.exclude,
            )?;
        }
        create_fortune_db(&config.settings)?;
    }

    Ok(())
}

macro_rules! illegal_file_suffixes {
    ($($ext:literal),*) => {
        [
            $(std::ffi::OsStr::new($ext)),*
        ]
    };
}

///Checks if the directory exists. If it doesn't, it creates it
macro_rules! check_dir_exists {
    ($path:expr) => {
        if let Err(_) = std::fs::read_dir($path) {
            std::fs::create_dir($path)?
        }
    };
}

fn create_fortune_db(settings: &BuildSettings) -> Result<(), std::io::Error> {
    /***************************************
     * Function Definitions (Because this is easier to fold)
     ***************************************/
    fn get_fortune_strings(path: &PathBuf, is_offensive: bool) -> (String, String) {
        let illegal_file_suffixes: [&OsStr; 16] = illegal_file_suffixes!(
            "dat", "pos", "c", "h", "p", "i", "f", "pas", "ftn", "ins.c", "ins.pas", "ins.ftn",
            "sml", "sh", "pl", "csv"
        );
        let mut fortune_buf = String::new();
        let mut off_fortune_buf = String::new();

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
                            off_fortune_buf = off_fortune_buf
                                + get_fortune_strings(&item.path(), true).1.as_str();
                        } else {
                            let (fortunes, off_fortunes) =
                                get_fortune_strings(&item.path(), is_offensive);
                            fortune_buf = fortune_buf + fortunes.as_str();
                            off_fortune_buf = off_fortune_buf + off_fortunes.as_str();
                        }
                    }
                    false => match File::open(item.path()) {
                        Ok(mut file) => {
                            let mut buf = String::new();
                            let _ = file.read_to_string(&mut buf);

                            match is_offensive {
                                true => off_fortune_buf = off_fortune_buf + buf.as_str(),
                                false => fortune_buf = fortune_buf + buf.as_str(),
                            };
                        }
                        Err(_) => panic!(
                            "Could not open a fortune file for copying into internal buffers"
                        ),
                    },
                },
                Err(e) => panic!("Could not identify a file in the fortune directory {e}"),
            }
        }
        (fortune_buf, off_fortune_buf)
    }

    ///Function used for the filter iterators
    fn check_fortune_constraints(
        element: &&str,
        max_width: &Option<u64>,
        max_lines: &Option<u64>,
    ) -> bool {
        (match max_width {
        Some(val) =>
            element
                .split("\n")
                .reduce(|acc, e| if e.len() > acc.len(){e} else {acc})
                .expect("Could not split the chosen string for constraint validation")
                .len() <= *val as usize,
        None => true,
    })
    //You can do this yes very cool
    &&(match max_lines {
        Some(val) => {
            element.chars().fold(0, |acc, e| match e == '\n' {
                true => acc + 1,
                false => acc,
            }) <= *val
        }
        None => true,
    })
    }

    fn gen_fortune_db(
        path: String,
        max_width: &Option<u64>,
        max_lines: &Option<u64>,
    ) -> Result<(), io::Error> {
        println!("cargo::rerun-if-changed={path}");

        let mut concat_fortunes: String;
        let off_concat_fortunes: String;
        match fs::metadata(&path)?.is_file() {
            true => {
                //Assume file contains only non-offensive fortunes
                match File::open(&path) {
                    Ok(mut file) => {
                        concat_fortunes = String::new();
                        let _ = file.read_to_string(&mut concat_fortunes)?;
                        off_concat_fortunes = String::new();
                    }
                    Err(_) => panic!("Could not read specified file defined by FORTUNE_FILE"),
                }
            }
            false => {
                let (fortune_list, offensive_list) =
                    get_fortune_strings(&PathBuf::from(path), false);
                concat_fortunes = fortune_list.replace("\r\n", "\n");
                off_concat_fortunes = offensive_list.replace("\r\n", "\n");
            }
        }

        //TODO probably need to pass settings as param no closure capture here
        let fortunes_split: Vec<&str> = concat_fortunes
            .split("\n%\n")
            .filter(|element| check_fortune_constraints(element, max_width, max_lines))
            .collect();
        let num_fortunes = fortunes_split.len();
        let off_fortunes_split: Vec<&str> = off_concat_fortunes
            .split("\n%\n")
            .filter(|element| check_fortune_constraints(element, max_width, max_lines))
            .collect();
        let num_off_fortunes = off_fortunes_split.len();

        let fortune_arr = quote::quote! {
            const FORTUNE_LIST: [&'static str; #num_fortunes] = [
                #(#fortunes_split) ,*
            ];
        };

        let off_fortune_arr = quote::quote! {
            const OFF_FORTUNE_LIST: [&'static str; #num_off_fortunes] = [
                #(#off_fortunes_split) ,*
            ];
        };

        match File::create("target/generated_sources/fortune_db.rs") {
            Ok(mut file) => {
                let _ = file.write_all(fortune_arr.to_string().as_bytes())?;
                if check_env_flag!("CARGO_FEATURE_INLINE_OFF_FORTUNE") {
                    let _ = file.write_all(off_fortune_arr.to_string().as_bytes())?;
                }
            }
            Err(err) => panic!("Could not concatenate fortunes into single file: {err}"),
        }

        Ok(())
    }

    /***************************************
     * The actual build steps
     ***************************************/
    check_dir_exists!("target/resources");
    check_dir_exists!("target/generated_sources");

    if check_env_flag!("USE_DEFAULT_RESOURCES")
        || (!check_env_flag!("FORTUNE_FILE") && !(check_env_flag!("FORTUNE_PATH")))
    {
        gen_fortune_db(
            String::from("target/resources/fortune"),
            &settings.max_width,
            &settings.max_lines,
        )
    } else {
        if let Ok(val) = std::env::var("FORTUNE_FILE") {
            println!("cargo::rerun-if-changed={val}");
            gen_fortune_db(val, &settings.max_width, &settings.max_lines)
        } else if let Ok(val) = std::env::var("FORTUNE_PATH") {
            println!("cargo::rerun-if-changed={val}");
            gen_fortune_db(val, &settings.max_width, &settings.max_lines)
        } else {
            panic!("Unexpected else branch hit toward end of create_fortune_db")
        }
    }
}

fn generate_cowsay_source() -> Result<(), std::io::Error> {
    /***************************************
     * Function Definitions (Because this is easier to fold)
     ***************************************/
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
                            let key = item
                                .file_name()
                                .to_str()
                                .unwrap()
                                .to_string()
                                .replace(".cow", "");
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
    check_dir_exists!("target/generated_sources");

    let cowpath: PathBuf;
    if check_env_flag!("USE_DEFAULT_RESOURCES") || !check_env_flag!("COW_PATH") {
        cowpath = PathBuf::from("target/resources/cowsay");
    } else {
        let path = std::env::var("COW_PATH").unwrap();
        println!("cargo::rerun-if-changed={path}");
        cowpath = PathBuf::from(path.as_str());
    }

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
/// to `/target/downloads`.
fn get_source_archive(
    path: &str,
    resource_name: &str,
    force_download: bool,
) -> Result<(), std::io::Error> {
    let downloads_path = String::from("target/downloads");
    check_dir_exists!(downloads_path.as_str());

    //Short circuit if we aren't force-redownloading and the resource exists
    match fs::metadata(format!("{downloads_path}/{resource_name}.zip").as_str()) {
        Ok(_) if force_download => {
            remove_file(format!("{downloads_path}/{resource_name}.zip").as_str())?;
        }
        Ok(_) => {
            return Ok(()); //Short Circuit
        }
        Err(_) => (),
    };

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
                "-L",
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
    exclude: &Option<Vec<String>>,
) -> Result<(), std::io::Error> {
    match fs::read_dir("target/tmp") {
        Ok(_) => {
            fs::remove_dir_all("target/tmp")?;
            fs::create_dir("target/tmp")?;
        }
        Err(_) => fs::create_dir("target/tmp")?,
    };
    if let Err(_) = fs::read_dir("target/resources") {
        fs::create_dir("target/resources")?
    }
    match fs::read_dir(destination) {
        Ok(_) => {
            fs::remove_dir_all(destination)?;
            fs::create_dir(destination)?;
        }
        Err(_) => fs::create_dir(destination)?,
    };

    let archive_file = match File::open(archive) {
        Ok(file) => BufReader::new(file),
        Err(_) => panic!("Could not doo this"),
    };
    let mut zip_archive = zip::ZipArchive::new(archive_file)?;
    zip_archive.extract("target/tmp")?;

    let resource_path = format!("target/tmp/{internal_path}");
    let copy_opts = fs_extra::dir::CopyOptions::new().overwrite(true);
    let resource_list: Vec<PathBuf> = fs::read_dir(resource_path)?
        .filter(|file| {
            if let Some(exclude_list) = exclude {
                !exclude_list.contains(
                    &file
                        .as_ref()
                        .expect("Could not get metadata for some of the resources")
                        .file_name()
                        .clone()
                        .into_string()
                        .unwrap(),
                )
            } else {
                true
            }
        })
        .map(|file| {
            file.expect("Could not get metadata for some of the resources")
                .path()
        })
        .collect();
    let _ = fs_extra::copy_items(resource_list.as_slice(), destination, &copy_opts)
        .expect("Could not copy resources as expected!");

    Ok(())
}
/************************************************/
/**************Configuration Functions***********/
/************************************************/

#[derive(serde::Deserialize, Debug)]
struct ResourceConfig {
    #[serde(rename = "source-zip-url")]
    pub url: String,
    #[serde(rename = "resource-location")]
    pub internal_path: String,
    pub exclude: Option<Vec<String>>,
}

#[derive(serde::Deserialize, Debug)]
struct BuildSettings {
    #[serde(rename = "max-fortune-line-len")]
    pub max_width: Option<u64>,
    #[serde(rename = "max-fortune-lines")]
    pub max_lines: Option<u64>,
}

#[derive(serde::Deserialize, Debug)]
struct BuildConfig {
    pub cowsay: ResourceConfig,
    #[serde(rename = "fortune-mod")]
    pub fortune_mod: ResourceConfig,
    pub settings: BuildSettings,
}

fn get_config() -> Result<BuildConfig, std::io::Error> {
    use std::io::Read;
    println!("cargo::rerun-if-changed=./BuildConfig.toml");

    match File::open("./BuildConfig.toml") {
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
