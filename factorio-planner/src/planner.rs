use crate::item::{Flow, Item, ItemName};
use crate::recipe::{Plan, Recipe, RecipeBook, RecipeName};
use std::collections::HashMap;

pub struct Planner {
    recipe_book: RecipeBook,
    flow: Flow,
    desires: Flow,
    recipes: HashMap<RecipeName, f64>,
}

impl Planner {
    pub fn new(recipe_book: RecipeBook) -> Planner {
        Planner {
            recipe_book,
            flow: Flow::new(),
            desires: Flow::new(),
            recipes: HashMap::new(),
        }
    }

    pub fn flow(&self) -> &Flow {
        &self.flow
    }

    pub fn desires(&self) -> &Flow {
        &self.desires
    }

    pub fn recipes(&self) -> impl Iterator<Item = (RecipeName, f64)> + '_ {
        self.recipes.iter().map(|(recipe, qty)| (*recipe, *qty))
    }

    pub fn all_items(&self) -> impl Iterator<Item = ItemName> + '_ {
        self.recipe_book.all_items()
    }

    pub fn all_recipes(&self) -> impl Iterator<Item = &Recipe> + '_ {
        self.recipe_book
            .all_recipes_and_plans()
            .filter_map(|r| r.as_recipe())
    }

    pub fn all_plans(&self) -> impl Iterator<Item = &Plan> + '_ {
        self.recipe_book
            .all_recipes_and_plans()
            .filter_map(|r| r.as_plan())
    }

    pub fn inputs(&self) -> impl Iterator<Item = (ItemName, f64)> + '_ {
        self.flow
            .items
            .iter()
            .map(|(item, rate)| (*item, *rate))
            .filter(|(_item, rate)| *rate < 0.0)
    }

    pub fn outputs(&self) -> impl Iterator<Item = (ItemName, f64)> + '_ {
        self.flow
            .items
            .iter()
            .map(|(item, rate)| (*item, *rate))
            .filter(|(_item, rate)| *rate > 0.0)
    }

    pub fn add_desire(&mut self, rate: f64, item: &str) {
        // TODO: probably shouldn't crash on typos
        let name = self.recipe_book.item(item).unwrap();
        self.desires += Item { name, rate };
    }

    pub fn recipes_with_input(&mut self, input: ItemName) -> impl Iterator<Item = RecipeName> + '_ {
        self.recipe_book.find_recipes_with_input(input)
    }

    pub fn recipes_with_output(
        &mut self,
        output: ItemName,
    ) -> impl Iterator<Item = RecipeName> + '_ {
        self.recipe_book.find_recipes_with_output(output)
    }

    pub fn choose_recipe(&mut self, recipe: &str, item: &str) {
        let recipe = self.recipe_book.recipe_name(recipe).unwrap();
        let recipe_or_plan = &self.recipe_book[recipe];
        let mut flow = self.recipe_book.flow(recipe_or_plan);
        let qty = if let Some(rate) = self.desires.items.get_mut(item) {
            let qty = *rate / flow.items[item];
            *rate = 0.0;
            qty
        } else {
            -self.flow.items[item] / flow.items[item]
        };
        flow *= qty;
        *self.recipes.entry(recipe).or_insert(0.0) += qty;
        self.flow += flow;
    }
}
