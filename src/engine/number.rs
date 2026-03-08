use num_bigint::BigInt;
use num_rational::BigRational;

pub type Number = BigRational;

pub fn format_number(number: &Number) -> String {
    if number.is_integer() {
        number.to_integer().to_string()
    } else {
        format!("{}/{}", number.numer(), number.denom())
    }
}

pub fn pow10(exponent: usize) -> BigInt {
    BigInt::from(10u8).pow(exponent as u32)
}
