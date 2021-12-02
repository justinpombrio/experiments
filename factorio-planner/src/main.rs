#![feature(str_strip)]

mod item;
mod planner;
mod recipe;

use once_cell::sync::Lazy;
use planner::Planner;
use recipe::RecipeBook;
use regex::Regex;
use std::io::{stdin, stdout, Write};

static DESIRE_OUTPUT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("desire output ([0-9.]+) ([a-zA-Z0-9_-]+)").unwrap());
static USE_RECIPE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("use ([a-zA-Z0-9_-]+) for ([a-zA-Z0-9_-]+)").unwrap());

struct Wizard {
    planner: Planner,
}

enum Command {
    Quit,
    Help,
    Show,
    ViewItems,
    ViewRecipes,
    ViewPlans,
    DesireOutput(f64, String),
    UseRecipe(String, String),
    Clear,
    Save,
    Invalid,
}

impl Command {
    fn parse(input: &str) -> Command {
        use Command::*;
        let input = input.trim();
        match input {
            "" | "show" => Show,
            "quit" | "exit" | "die" => Quit,
            "help" | "Help" | "halp" | "?" | "F1" => Help,
            "view items" => ViewItems,
            "view recipes" => ViewRecipes,
            "view plans" => ViewPlans,
            "clear" => Clear,
            "save" => Save,
            _ => {
                if let Some(captures) = DESIRE_OUTPUT_REGEX.captures(input) {
                    DesireOutput(
                        captures[1].to_owned().parse::<f64>().unwrap(),
                        captures[2].to_owned(),
                    )
                } else if let Some(captures) = USE_RECIPE_REGEX.captures(input) {
                    UseRecipe(captures[1].to_owned(), captures[2].to_owned())
                } else {
                    Invalid
                }
            }
        }
    }
}

impl Wizard {
    fn new() -> Wizard {
        // Hard-coding some recipes for now, for testing
        let mut book = RecipeBook::new();
        book.add_recipe("Cog", 0.5, vec![(2.0, "iron")], vec![(1.0, "cog")]);
        book.add_recipe(
            "Belt",
            0.5,
            vec![(1.0, "cog"), (1.0, "iron")],
            vec![(2.0, "belt")],
        );
        let planner = Planner::new(book);
        Wizard { planner }
    }

    fn enliven(&mut self) {
        loop {
            print!("\n> ");
            stdout().flush().unwrap();
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            self.obey(Command::parse(&input));
        }
    }

    fn obey(&mut self, command: Command) {
        use Command::*;
        match command {
            Quit => ::std::process::exit(0),
            Help => {
                println!("I'm a Wizard! Good luck.");
            }
            Show => {
                println!("Recipes in use:");
                let mut recipes = self.planner.recipes().collect::<Vec<_>>();
                recipes.sort_unstable_by_key(|(r, _)| *r);
                for (recipe, qty) in recipes {
                    println!("  {} {}", qty, recipe);
                }
                println!("Desires:\n{}", self.planner.desires());
                println!("Current flow:\n{}", self.planner.flow());
            }
            ViewItems => {
                let mut items = self.planner.all_items().collect::<Vec<_>>();
                items.sort_unstable();
                println!("All items:");
                for item in items {
                    println!("  {}", item);
                }
            }
            ViewRecipes => {
                let mut recipes = self.planner.all_recipes().collect::<Vec<_>>();
                recipes.sort_unstable();
                println!("All recipes:");
                for recipe in recipes {
                    println!("{}", recipe);
                }
            }
            ViewPlans => {
                let mut plans = self.planner.all_plans().collect::<Vec<_>>();
                plans.sort_unstable();
                println!("All plans:");
                for plan in plans {
                    println!("{}", plan);
                }
            }
            DesireOutput(rate, item) => {
                self.planner.add_desire(rate, &item);
                println!("Desires:\n{}", self.planner.desires());
            }
            UseRecipe(recipe, item) => {
                self.planner.choose_recipe(&recipe, &item);
                println!("New flow:\n{}", self.planner.flow());
            }
            Clear | Save => unimplemented!(),
            Invalid => println!("I don't understand"),
        }
    }
}

fn main() {
    let mut wizard = Wizard::new();
    wizard.enliven();
}
