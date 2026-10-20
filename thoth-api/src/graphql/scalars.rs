//! GraphQL scalars owned by the GraphQL layer rather than by one domain model.
//!
//! `MET-WP4-02` introduces exactly one: [`BigInt`], the transport for Metrics
//! aggregate values. The repository's existing `Uuid`, `Date` and `Timestamp`
//! scalars are reused unchanged; no duplicate `UUID` or `DateTime` scalar is
//! declared.

use juniper::{InputValue, ScalarValue, Value};

/// An exact signed integer carried as a canonical base-10 string.
///
/// GraphQL's built-in `Int` is 32-bit and a JSON number is commonly decoded as
/// a double, so neither can carry a Metrics aggregate without an eventual
/// silent truncation or loss of precision. This scalar therefore serializes as
/// a JSON **string** in canonical form: an optional leading `-`, no leading
/// zeros, no `+`, no whitespace, no exponent and no `-0`.
///
/// The value is held as `i128`. Every stored Metrics value is a signed
/// `BIGINT`, and a dashboard aggregate is a sum of at most a bounded number of
/// such values, so `i128` holds every reachable total exactly. Arithmetic on
/// it is always checked: an aggregate that cannot be represented fails rather
/// than wrapping, saturating or being rounded.
#[derive(juniper::GraphQLScalar, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[graphql(
    name = "BigInt",
    with = big_int,
    parse_token(String),
    description = "An exact signed integer serialized as a canonical base-10 string, for example \"-42\" or \"18446744073709551614\". It is never a JSON number, never rounded and never truncated"
)]
pub struct BigInt(i128);

impl BigInt {
    pub const ZERO: BigInt = BigInt(0);

    pub fn new(value: i128) -> Self {
        BigInt(value)
    }

    pub fn value(self) -> i128 {
        self.0
    }

    /// Add two values, or `None` if the exact result is not representable.
    pub fn checked_add(self, other: BigInt) -> Option<BigInt> {
        self.0.checked_add(other.0).map(BigInt)
    }

    /// Parse the canonical decimal form, rejecting every other spelling.
    ///
    /// This is also the path used to read a PostgreSQL `NUMERIC` aggregate
    /// rendered as text, so a sum outside the `i128` range is rejected here
    /// instead of being approximated.
    pub fn parse_canonical(input: &str) -> Option<BigInt> {
        let digits = input.strip_prefix('-').unwrap_or(input);
        let canonical = !digits.is_empty()
            && digits.bytes().all(|byte| byte.is_ascii_digit())
            && (digits == "0" || !digits.starts_with('0'))
            && input != "-0";
        if !canonical {
            return None;
        }
        input.parse::<i128>().ok().map(BigInt)
    }
}

impl From<i64> for BigInt {
    fn from(value: i64) -> Self {
        BigInt(i128::from(value))
    }
}

impl std::fmt::Display for BigInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

mod big_int {
    use super::*;

    pub(super) fn to_output<S: ScalarValue>(value: &BigInt) -> Value<S> {
        Value::scalar(value.0.to_string())
    }

    pub(super) fn from_input<S: ScalarValue>(value: &InputValue<S>) -> Result<BigInt, String> {
        value
            .as_string_value()
            .ok_or_else(|| "Expected a BigInt as a canonical base-10 string".to_string())
            .and_then(|text| {
                BigInt::parse_canonical(text).ok_or_else(|| {
                    "Invalid BigInt: expected a canonical base-10 string".to_string()
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use juniper::{DefaultScalarValue, FromInputValue};

    #[test]
    fn values_beyond_every_narrower_integer_serialize_exactly_as_strings() {
        for (value, expected) in [
            (0_i128, "0"),
            (-1, "-1"),
            (i128::from(i32::MAX) + 1, "2147483648"),
            (i128::from(i64::MAX), "9223372036854775807"),
            (i128::from(i64::MIN), "-9223372036854775808"),
            (i128::from(i64::MAX) * 2, "18446744073709551614"),
            (i128::MAX, "170141183460469231731687303715884105727"),
            (i128::MIN, "-170141183460469231731687303715884105728"),
        ] {
            let rendered: Value<DefaultScalarValue> = big_int::to_output(&BigInt(value));
            assert_eq!(
                rendered,
                Value::scalar(expected.to_string()),
                "{value} must render as the exact string {expected}"
            );
        }
    }

    #[test]
    fn only_the_canonical_decimal_form_is_accepted() {
        for accepted in [
            "0",
            "7",
            "-7",
            "9223372036854775808",
            "-9223372036854775809",
        ] {
            let input = InputValue::<DefaultScalarValue>::scalar(accepted.to_string());
            let parsed = BigInt::from_input_value(&input).expect("canonical form parses");
            assert_eq!(parsed.to_string(), accepted);
        }
        for rejected in [
            "",
            "-",
            "-0",
            "00",
            "007",
            "+7",
            " 7",
            "7 ",
            "1e3",
            "1.0",
            "0x10",
            "١",
            // One beyond the i128 range in each direction.
            "170141183460469231731687303715884105728",
            "-170141183460469231731687303715884105729",
        ] {
            let input = InputValue::<DefaultScalarValue>::scalar(rejected.to_string());
            assert!(
                BigInt::from_input_value(&input).is_err(),
                "`{rejected}` must not be accepted as a BigInt"
            );
        }
        let number = InputValue::<DefaultScalarValue>::scalar(7);
        assert!(
            BigInt::from_input_value(&number).is_err(),
            "a JSON number is not a BigInt"
        );
    }

    #[test]
    fn addition_is_checked_rather_than_wrapping_or_saturating() {
        assert_eq!(
            BigInt::from(i64::MAX).checked_add(BigInt::from(i64::MAX)),
            Some(BigInt(i128::from(i64::MAX) * 2))
        );
        assert_eq!(
            BigInt::from(-5).checked_add(BigInt::from(3)),
            Some(BigInt(-2))
        );
        assert_eq!(BigInt(i128::MAX).checked_add(BigInt(1)), None);
        assert_eq!(BigInt(i128::MIN).checked_add(BigInt(-1)), None);
    }
}
