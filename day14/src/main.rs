use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

const COLLECTED_ORE: u64 = 1000000000000;

#[derive(Debug, Eq, PartialEq)]
struct Reaction {
    output: (String, u64),
    ingredients: Vec<(String, u64)>,
}

type ReactionMap = HashMap<String, Reaction>;

fn calc_ore(reactions: &ReactionMap) -> u64 {
    calc_ore_for_fuel(1, reactions)
}

fn calc_ore_for_fuel(fuel: u64, reactions: &ReactionMap) -> u64 {
    let mut ore = 0;
    let mut spare_chemicals = HashMap::new();
    let mut requirements = Vec::new();

    requirements.push((String::from("FUEL"), fuel));
    let ore_name = String::from("ORE");

    while !requirements.is_empty() {
        let cur_requirements = requirements.clone();
        requirements.clear();

        for (req_chem, req_amount) in cur_requirements {
            // Check whether we have any spare of this ingredient from
            // other reactions.
            let mut adj_req_amount = req_amount;
            if let Some(spare) = spare_chemicals.get_mut(&req_chem) {
                if *spare >= req_amount {
                    // We have enough spare to completely fulfill this
                    // requirement, no need to go further.
                    *spare -= req_amount;
                    continue;
                } else {
                    // Reduce the required amount by the amount we have
                    // spare;
                    adj_req_amount = req_amount - *spare;
                    *spare = 0;
                }
            }

            // Find the reaction that produces this ingredient.
            let reaction = reactions
                .get(&req_chem)
                .expect(format!("Couldn't find reaction for {}", req_chem).as_ref());

            // Find out how many times we need to run this reaction,
            // and how much will be spare.
            let output_amount = reaction.output.1;
            let reaction_count = (adj_req_amount - 1) / output_amount + 1;
            let spare = output_amount * reaction_count - adj_req_amount;

            // Update the spare count for this ingredient.
            if let Some(existing_spare) = spare_chemicals.get_mut(&req_chem) {
                *existing_spare += spare;
            } else {
                spare_chemicals.insert(req_chem, spare);
            }

            // Update the required ingredients list with the ingredients
            // needed to make this chemical.
            for ingredient in reaction.ingredients.clone() {
                let ingredient_name = ingredient.0;
                let ingredient_count = reaction_count * ingredient.1;

                if ingredient_name == ore_name {
                    ore += ingredient_count;
                } else {
                    requirements.push((ingredient_name, ingredient_count));
                }
            }
        }
    }

    ore
}

fn calc_fuel_for_ore(ore: u64, reactions: &ReactionMap) -> u64 {
    let mut lower = 1;
    let mut current;
    let mut upper = 1;

    // Find an upper bound to use for binary search.
    loop {
        let used_ore = calc_ore_for_fuel(upper, reactions);
        if used_ore < ore {
            upper *= 2;
        } else {
            break;
        }
    }

    // Binary search to find the highest amount of fuel we can
    // produce without using all the fuel.
    loop {
        current = (upper - lower) / 2 + lower;

        let used_ore = calc_ore_for_fuel(current, reactions);

        if used_ore < ore {
            lower = current;
        } else {
            upper = current;
        }

        if upper - 1 == lower {
            return lower;
        }
    }
}

fn parse_chemical(chemical: &str) -> (String, u64) {
    let mut iter = chemical.split_whitespace();
    let count = iter.next().unwrap().parse::<u64>().unwrap();
    let chem = iter.next().unwrap();

    (String::from(chem), count)
}

fn parse_reactions(strs: &[String]) -> ReactionMap {
    let mut reactions = HashMap::new();

    for reaction in strs {
        let mut iter = reaction.split(" => ");
        let ingredients_str = iter.next().unwrap();
        let output_str = iter.next().unwrap();

        let mut ingredients = Vec::new();
        for ingredient in ingredients_str.split(", ") {
            ingredients.push(parse_chemical(ingredient));
        }

        let output = parse_chemical(output_str);
        reactions.insert(
            output.0.clone(),
            Reaction {
                output: output,
                ingredients: ingredients,
            },
        );
    }

    reactions
}

fn parse_input(filename: &str) -> ReactionMap {
    let file = File::open(filename).expect("Failed to open file");
    let reader = BufReader::new(file);

    let reactions: Vec<String> = reader
        .lines()
        .map(|l| l.expect("Failed to read line"))
        .map(|l| String::from(l.trim()))
        .collect();
    parse_reactions(reactions.as_slice())
}

fn main() {
    let reactions = parse_input("input");

    // Part 1
    let ore = calc_ore(&reactions);
    println!("Require {} ore for 1 fuel", ore);

    // Part 2
    let fuel = calc_fuel_for_ore(COLLECTED_ORE, &reactions);
    println!("Produce {} fuel from {} ore", fuel, COLLECTED_ORE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let input = vec![String::from("7 A, 1 E => 1 FUEL")];
        let reactions = parse_reactions(input.as_slice());

        let result = reactions.get(&String::from("FUEL"));
        assert!(result.is_some());

        let reaction = result.unwrap();
        assert_eq!(
            *reaction,
            Reaction {
                output: (String::from("FUEL"), 1),
                ingredients: vec![(String::from("A"), 7), (String::from("E"), 1),],
            },
        );
    }

    #[test]
    fn example1() {
        let input = vec![
            String::from("10 ORE => 10 A"),
            String::from("1 ORE => 1 B"),
            String::from("7 A, 1 B => 1 C"),
            String::from("7 A, 1 C => 1 D"),
            String::from("7 A, 1 D => 1 E"),
            String::from("7 A, 1 E => 1 FUEL"),
        ];

        let reactions = parse_reactions(input.as_slice());
        let result = calc_ore(&reactions);
        assert_eq!(result, 31);
    }

    #[test]
    fn example2() {
        let input = vec![
            String::from("9 ORE => 2 A"),
            String::from("8 ORE => 3 B"),
            String::from("7 ORE => 5 C"),
            String::from("3 A, 4 B => 1 AB"),
            String::from("5 B, 7 C => 1 BC"),
            String::from("4 C, 1 A => 1 CA"),
            String::from("2 AB, 3 BC, 4 CA => 1 FUEL"),
        ];

        let reactions = parse_reactions(input.as_slice());
        let result = calc_ore(&reactions);

        assert_eq!(result, 165);
    }

    #[test]
    fn example3() {
        let input = vec![
            String::from("157 ORE => 5 NZVS"),
            String::from("165 ORE => 6 DCFZ"),
            String::from("44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL"),
            String::from("12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ"),
            String::from("179 ORE => 7 PSHF"),
            String::from("177 ORE => 5 HKGWZ"),
            String::from("7 DCFZ, 7 PSHF => 2 XJWVT"),
            String::from("165 ORE => 2 GPVTF"),
            String::from("3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT"),
        ];

        let reactions = parse_reactions(input.as_slice());

        let result = calc_ore(&reactions);
        assert_eq!(result, 13312);

        let result = calc_fuel_for_ore(COLLECTED_ORE, &reactions);
        assert_eq!(result, 82892753);
    }

    #[test]
    fn example4() {
        let input = vec![
            String::from("2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG"),
            String::from("17 NVRVD, 3 JNWZP => 8 VPVL"),
            String::from("53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL"),
            String::from("22 VJHF, 37 MNCFX => 5 FWMGM"),
            String::from("139 ORE => 4 NVRVD"),
            String::from("144 ORE => 7 JNWZP"),
            String::from("5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC"),
            String::from("5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV"),
            String::from("145 ORE => 6 MNCFX"),
            String::from("1 NVRVD => 8 CXFTF"),
            String::from("1 VJHF, 6 MNCFX => 4 RFSQX"),
            String::from("176 ORE => 6 VJHF"),
        ];

        let reactions = parse_reactions(input.as_slice());

        let result = calc_ore(&reactions);
        assert_eq!(result, 180697);

        let result = calc_fuel_for_ore(COLLECTED_ORE, &reactions);
        assert_eq!(result, 5586022);
    }

    #[test]
    fn example5() {
        let input = vec![
            String::from("171 ORE => 8 CNZTR"),
            String::from("7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL"),
            String::from("114 ORE => 4 BHXH"),
            String::from("14 VRPVC => 6 BMBT"),
            String::from("6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL"),
            String::from("6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT"),
            String::from("15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW"),
            String::from("13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW"),
            String::from("5 BMBT => 4 WPTQ"),
            String::from("189 ORE => 9 KTJDG"),
            String::from("1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP"),
            String::from("12 VRPVC, 27 CNZTR => 2 XDBXC"),
            String::from("15 KTJDG, 12 BHXH => 5 XCVML"),
            String::from("3 BHXH, 2 VRPVC => 7 MZWV"),
            String::from("121 ORE => 7 VRPVC"),
            String::from("7 XCVML => 6 RJRHP"),
            String::from("5 BHXH, 4 VRPVC => 5 LTCX"),
        ];

        let reactions = parse_reactions(input.as_slice());

        let result = calc_ore(&reactions);
        assert_eq!(result, 2210736);

        let result = calc_fuel_for_ore(COLLECTED_ORE, &reactions);
        assert_eq!(result, 460664);
    }
}
