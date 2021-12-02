use crate::item::{Flow, Item, ItemName};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::Index;

pub type RecipeName = &'static str;

#[derive(Debug, Clone)]
pub struct RecipeBook {
    items: HashMap<String, ItemName>,
    recipes: HashMap<String, RecipeOrPlan>,
    recipes_by_input: HashMap<ItemName, HashSet<RecipeName>>,
    recipes_by_output: HashMap<ItemName, HashSet<RecipeName>>,
}

#[derive(Debug, Clone)]
pub enum RecipeOrPlan {
    Recipe(Recipe),
    Plan(Plan),
}

#[derive(Debug, Clone)]
pub struct Recipe {
    name: RecipeName,
    flow: Flow,
}

#[derive(Debug, Clone)]
pub struct Plan {
    name: RecipeName,
    flow: Flow,
    recipes: HashMap<RecipeName, f64>,
}

impl fmt::Display for Recipe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "  {}:\n{}", self.name, self.flow)
    }
}

impl fmt::Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "  {}:\n{}", self.name, self.flow)
    }
}

impl RecipeOrPlan {
    pub fn name(&self) -> RecipeName {
        match self {
            RecipeOrPlan::Recipe(Recipe { name, .. }) => name,
            RecipeOrPlan::Plan(Plan { name, .. }) => name,
        }
    }

    pub fn as_recipe(&self) -> Option<&Recipe> {
        match self {
            RecipeOrPlan::Recipe(recipe) => Some(recipe),
            RecipeOrPlan::Plan(_) => None,
        }
    }

    pub fn as_plan(&self) -> Option<&Plan> {
        match self {
            RecipeOrPlan::Recipe(_) => None,
            RecipeOrPlan::Plan(plan) => Some(plan),
        }
    }
}

impl Index<RecipeName> for RecipeBook {
    type Output = RecipeOrPlan;
    fn index(&self, name: RecipeName) -> &RecipeOrPlan {
        &self.recipes[name]
    }
}

impl RecipeBook {
    pub fn new() -> RecipeBook {
        RecipeBook {
            items: HashMap::new(),
            recipes: HashMap::new(),
            recipes_by_input: HashMap::new(),
            recipes_by_output: HashMap::new(),
        }
    }

    pub fn add_recipe(
        &mut self,
        recipe_name: &str,
        time: f64,
        inputs: Vec<(f64, &str)>,
        outputs: Vec<(f64, &str)>,
    ) -> RecipeName {
        let mut flow = Flow::new();
        for (amount, name) in inputs {
            let name = self.insert_item(name);
            let rate = amount / time;
            flow -= Item { name, rate };
        }
        for (amount, name) in outputs {
            let name = self.insert_item(name);
            let rate = amount / time;
            flow += Item { name, rate };
        }
        assert!(
            self.recipes.get(recipe_name).is_none(),
            "Duplicate recipe/plan name"
        );
        let name = intern_string(recipe_name);
        let recipe = Recipe { name, flow };
        self.recipes
            .insert(name.to_owned(), RecipeOrPlan::Recipe(recipe));
        name
    }

    pub fn add_plan(&mut self, name: &str, recipes: HashMap<RecipeName, f64>) -> RecipeName {
        assert!(
            self.recipes.get(name).is_none(),
            "Duplicate recipe/plan name"
        );
        let name = intern_string(name);
        let mut total_flow = Flow::new();
        for (recipe, qty) in &recipes {
            let mut flow = self.flow(&self[recipe]);
            flow *= *qty;
            total_flow += flow;
        }
        let plan = Plan {
            name,
            flow: total_flow,
            recipes,
        };
        self.recipes
            .insert(name.to_owned(), RecipeOrPlan::Plan(plan));
        name
    }

    pub fn flow(&self, recipe_or_plan: &RecipeOrPlan) -> Flow {
        match &recipe_or_plan {
            RecipeOrPlan::Recipe(recipe) => recipe.flow.clone(),
            RecipeOrPlan::Plan(plan) => {
                let mut total_flow = Flow::new();
                for (recipe_name, qty) in &plan.recipes {
                    let mut flow = self.flow(&self[recipe_name]);
                    flow *= *qty;
                    total_flow += flow;
                }
                total_flow
            }
        }
    }

    pub fn item(&self, name: &str) -> Option<ItemName> {
        self.items.get(name).copied()
    }

    pub fn recipe_name(&self, name: &str) -> Option<RecipeName> {
        self.recipes.get(name).map(|r| r.name())
    }

    pub fn all_items(&self) -> impl Iterator<Item = ItemName> + '_ {
        self.items.values().copied()
    }

    pub fn all_recipes_and_plans(&self) -> impl Iterator<Item = &RecipeOrPlan> + '_ {
        self.recipes.values()
    }

    pub fn find_recipes_with_input(
        &mut self,
        input: ItemName,
    ) -> impl Iterator<Item = RecipeName> + '_ {
        let set = self
            .recipes_by_input
            .entry(input)
            .or_insert_with(|| HashSet::new());
        set.iter().map(|s| *s)
    }

    pub fn find_recipes_with_output(
        &mut self,
        output: ItemName,
    ) -> impl Iterator<Item = RecipeName> + '_ {
        let set = self
            .recipes_by_output
            .entry(output)
            .or_insert_with(|| HashSet::new());
        set.iter().map(|s| *s)
    }

    fn insert_item(&mut self, name: &str) -> ItemName {
        if let Some(interned_name) = self.items.get(name) {
            interned_name
        } else {
            let interned_name = intern_string(name);
            self.items.insert(name.to_owned(), interned_name);
            interned_name
        }
    }

    fn insert_recipe_or_plan(&mut self, recipe_or_plan: RecipeOrPlan) -> RecipeName {
        let name = recipe_or_plan.name();
        let flow = self.flow(&recipe_or_plan);
        for (item, rate) in &flow.items {
            let set = if *rate < 0.0 {
                self.recipes_by_input
                    .entry(item)
                    .or_insert_with(|| HashSet::new())
            } else {
                self.recipes_by_output
                    .entry(item)
                    .or_insert_with(|| HashSet::new())
            };
            set.insert(name);
        }
        name
    }
}

fn intern_string(s: &str) -> &'static str {
    Box::leak(Box::new(s.to_owned()))
}

impl PartialEq for Recipe {
    fn eq(&self, other: &Recipe) -> bool {
        self.name == other.name
    }
}
impl Eq for Recipe {}
impl PartialOrd for Recipe {
    fn partial_cmp(&self, other: &Recipe) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Recipe {
    fn cmp(&self, other: &Recipe) -> Ordering {
        self.name.cmp(other.name)
    }
}

impl PartialEq for Plan {
    fn eq(&self, other: &Plan) -> bool {
        self.name == other.name
    }
}
impl Eq for Plan {}
impl PartialOrd for Plan {
    fn partial_cmp(&self, other: &Plan) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Plan {
    fn cmp(&self, other: &Plan) -> Ordering {
        self.name.cmp(other.name)
    }
}

impl PartialEq for RecipeOrPlan {
    fn eq(&self, other: &RecipeOrPlan) -> bool {
        self.name() == other.name()
    }
}
impl Eq for RecipeOrPlan {}
impl PartialOrd for RecipeOrPlan {
    fn partial_cmp(&self, other: &RecipeOrPlan) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for RecipeOrPlan {
    fn cmp(&self, other: &RecipeOrPlan) -> Ordering {
        self.name().cmp(other.name())
    }
}
