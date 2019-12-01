use std::fs::File;
use std::io::{BufRead, BufReader};

fn calc_fuel_simple(mass: i64) -> i64 {
    return (mass / 3) - 2;
}

fn calc_fuel_integrated(mass: i64) -> i64 {
    let mut fuel = calc_fuel_simple(mass);
    let mut new_fuel: i64 = calc_fuel_simple(fuel);

    while new_fuel > 0 {
        fuel += new_fuel;
        new_fuel = calc_fuel_simple(new_fuel);
    }

    return fuel;
}

fn main() {
    let file = File::open("input").unwrap();
    let reader = BufReader::new(file);
    let mut total = 0;
    for line in reader.lines() {
        let mass = line.unwrap().parse::<i64>().unwrap();
        total += calc_fuel_integrated(mass);
    }

    println!("{}", total);
}
