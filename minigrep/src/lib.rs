use std::{env, fs};
use std::error::Error;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let file_contents = fs::read_to_string(config.filename)?;

    let result = if config.case_sensitive {
        search_case_insentive(&config.querry, &file_contents)
    } else {
        search(&config.querry, &file_contents)
    };

    for line in result {
        println!("{}", line);
    }

    Ok(())
}

pub struct Config {
    pub querry: String,
    pub filename: String,
    pub case_sensitive: bool,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {

        if args.len() < 3 {
            return Err("Your arguments is not enough!")
        }

        let querry = args[1].clone();
        let filename = args[2].clone();
        let case_sensitive = env::var("CASE_SENSITIVE").is_err();
        Ok(Config {querry, filename, case_sensitive})
    }
}

pub fn search<'a>(querry: &str, contents: &'a str) -> Vec<&'a str> {

    let mut result = Vec::new();

    for line in contents.lines() {
        if line.contains(querry) {
            result.push(line);
        }
    }

    result
}

pub fn search_case_insentive<'a>(querry: &str, contents: &'a str) -> Vec<&'a str> {

    let mut result = Vec::new();

    let querry = querry.to_lowercase();

    for line in contents.lines() {
        if line.to_lowercase().contains(&querry) {
            result.push(line);
        }
    }

    result
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_sensitive() {
        let querry = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Duct tape";

        assert_eq!(vec!["safe, fast, productive."], search(querry, contents));
    }

    #[test]
    fn case_insensitive() {
        let querry = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(vec!["Rust:", "Trust me."], search_case_insentive(querry, contents));
    }

}