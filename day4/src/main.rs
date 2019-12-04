const MIN: u32 = 134564;
const MAX: u32 = 585159;

fn to_digits(num: u32) -> Vec<u32> {
    let mut cur = num;

    let mut output = Vec::new();
    while cur > 0 {
        output.push(cur % 10);
        cur /= 10;
    }
    output.reverse();
    return output;
}

fn check_num(num: u32) -> bool {
    let digits = to_digits(num);

    let mut has_double = false;
    for i in 1..digits.len() {
        // Check digits don't decrease.
        if digits[i] < digits[i - 1] {
            return false;
        }

        // Check we have a double, but not a longer sequence.
        if digits[i] == digits[i - 1] {
            if !((i > 1 && digits[i - 1] == digits[i - 2]) ||
                (i + 1 < digits.len() && digits[i] == digits[i + 1])) {
                has_double = true;
            }
        }
    }
    
    return has_double;
}


fn main() {
    let numbers: Vec<u32> = (MIN..=MAX).into_iter().filter(|n| check_num(*n)).collect();
    println!("Result: {}", numbers.len());
}
