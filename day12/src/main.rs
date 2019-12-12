use cgmath::Vector3;

const SIM_STEPS: u64 = 1000;

#[derive(Clone, Copy, PartialEq)]
struct Moon {
    position: Vector3<i64>,
    velocity: Vector3<i64>,
}

impl Moon {
    fn new(x: i64, y: i64, z: i64) -> Moon {
        Moon {
            position: Vector3::new(x, y, z),
            velocity: Vector3::new(0, 0, 0),
        }
    }
}

fn gcd(a: u64, b: u64) -> u64 {
    return if b == 0 { a } else { gcd(b, a % b) };
}

fn lcm(a: u64, b: u64) -> u64 {
    return a * b / gcd(a, b);
}

fn apply_gravity(own_moon: &mut Moon, other_moon: &Moon) {
    let adjust_dimension = |own_pos, other_pos| {
        if own_pos == other_pos {
            0
        } else if own_pos > other_pos {
            -1
        } else {
            1
        }
    };

    own_moon.velocity += Vector3::new(
        adjust_dimension(own_moon.position.x, other_moon.position.x),
        adjust_dimension(own_moon.position.y, other_moon.position.y),
        adjust_dimension(own_moon.position.z, other_moon.position.z),
    );
}

fn step_sim(moons: &mut Vec<Moon>) {
    for i in 0..moons.len() {
        for j in i + 1..moons.len() {
            let mut update = |idx1, idx2: usize| {
                let m2 = moons[idx2];
                let m1 = moons.get_mut(idx1).unwrap();

                apply_gravity(m1, &m2);
            };
            update(i, j);
            update(j, i);
        }
    }

    for moon in moons {
        moon.position += moon.velocity;
    }
}

fn run_sim(moons: &mut Vec<Moon>, steps: u64) {
    for _ in 0..steps {
        step_sim(moons);
    }
}

fn calc_energy(moons: &Vec<Moon>) -> u64 {
    return moons
        .iter()
        .map(|moon| {
            (moon.position.x.abs() as u64
                + moon.position.y.abs() as u64
                + moon.position.z.abs() as u64)
                * (moon.velocity.x.abs() as u64
                    + moon.velocity.y.abs() as u64
                    + moon.velocity.z.abs() as u64)
        })
        .sum();
}

fn find_repeats(orig_moons: &Vec<Moon>) -> u64 {
    let mut moons = orig_moons.clone();

    let mut i: u64 = 0;

    let mut x_repeat = None;
    let mut y_repeat = None;
    let mut z_repeat = None;
    loop {
        i += 1;
        run_sim(&mut moons, 1);

        let (x_match, y_match, z_match) =
            moons
                .iter()
                .zip(orig_moons.iter())
                .fold((true, true, true), |acc, (m1, m2)| {
                    let x =
                        acc.0 && m1.position.x == m2.position.x && m1.velocity.x == m2.velocity.x;
                    let y =
                        acc.1 && m1.position.y == m2.position.y && m1.velocity.y == m2.velocity.y;
                    let z =
                        acc.2 && m1.position.z == m2.position.z && m1.velocity.z == m2.velocity.z;

                    (x, y, z)
                });

        if x_repeat.is_none() && x_match {
            x_repeat = Some(i);
        }
        if y_repeat.is_none() && y_match {
            y_repeat = Some(i);
        }
        if z_repeat.is_none() && z_match {
            z_repeat = Some(i);
        }

        if x_repeat.is_some() && y_repeat.is_some() && z_repeat.is_some() {
            break;
        }
    }

    return lcm(x_repeat.unwrap(), lcm(y_repeat.unwrap(), z_repeat.unwrap()));
}

fn main() {
    let moons = vec![
        Moon::new(9, 13, -8),
        Moon::new(-3, 16, -17),
        Moon::new(-4, 11, -10),
        Moon::new(0, -2, -2),
    ];

    // Part 1
    let mut sim_moons = moons.clone();
    run_sim(&mut sim_moons, SIM_STEPS);
    let energy = calc_energy(&sim_moons);
    println!("Total energy after {} steps: {}", SIM_STEPS, energy);

    // Part 2
    let period = find_repeats(&moons);
    println!("Orbits repeat after {} steps", period);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pt1() {
        let mut moons = vec![
            Moon::new(-1, 0, 2),
            Moon::new(2, -10, -7),
            Moon::new(4, -8, 8),
            Moon::new(3, 5, -1),
        ];

        run_sim(&mut moons, 1);
        assert_eq!(moons[0].position, Vector3::new(2, -1, 1));
        assert_eq!(moons[0].velocity, Vector3::new(3, -1, -1));
        assert_eq!(moons[3].position, Vector3::new(2, 2, 0));
        assert_eq!(moons[3].velocity, Vector3::new(-1, -3, 1));

        run_sim(&mut moons, 9);
        assert_eq!(moons[1].position, Vector3::new(1, -8, 0));
        assert_eq!(moons[1].velocity, Vector3::new(-1, 1, 3));
        assert_eq!(moons[2].position, Vector3::new(3, -6, 1));
        assert_eq!(moons[2].velocity, Vector3::new(3, 2, -3));

        let energy = calc_energy(&moons);
        assert_eq!(energy, 179);
    }

    #[test]
    fn pt2() {
        let moons = vec![
            Moon::new(-1, 0, 2),
            Moon::new(2, -10, -7),
            Moon::new(4, -8, 8),
            Moon::new(3, 5, -1),
        ];

        let period = find_repeats(&moons);
        assert_eq!(period, 2772);

        let moons = vec![
            Moon::new(-8, -10, 0),
            Moon::new(5, 5, 10),
            Moon::new(2, -7, 3),
            Moon::new(9, -8, -3),
        ];

        let period = find_repeats(&moons);
        assert_eq!(period, 4686774924);
    }
}
