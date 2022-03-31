use crate::utils::float::{ApproxF64, Epsilon64};
use itertools::Itertools;
use patternfly_yew::*;
use std::{convert::Infallible, num::ParseFloatError};

pub trait FieldType: Sized {
    type ParseError: std::error::Error;

    fn required() -> bool;

    fn base_validator() -> Option<Validator<String, ValidationResult>> {
        Some(Validator::from(|ctx: ValidationContext<String>| {
            if ctx.value.is_empty() {
                if Self::required() {
                    ValidationResult::error("Value must not be empty")
                } else {
                    ValidationResult::ok()
                }
            } else {
                if let Err(err) = Self::parse(&ctx.value) {
                    ValidationResult::error(err.to_string())
                } else {
                    ValidationResult::ok()
                }
            }
        }))
    }

    fn parse(value: &str) -> Result<Self, Self::ParseError>;
    fn to_string(&self) -> String;
}

impl FieldType for String {
    type ParseError = Infallible;

    fn required() -> bool {
        true
    }

    fn base_validator() -> Option<Validator<String, ValidationResult>> {
        Some(Validator::from(|ctx: ValidationContext<String>| {
            if ctx.value.is_empty() {
                ValidationResult::error("Value must not be empty")
            } else {
                ValidationResult::ok()
            }
        }))
    }

    fn parse(value: &str) -> Result<Self, Self::ParseError> {
        Ok(value.to_string())
    }

    fn to_string(&self) -> String {
        self.clone()
    }
}

impl FieldType for f64 {
    type ParseError = ParseFloatError;

    fn required() -> bool {
        true
    }

    fn parse(value: &str) -> Result<Self, Self::ParseError> {
        value.parse()
    }

    fn to_string(&self) -> String {
        ToString::to_string(self)
    }
}

impl FieldType for humantime::Duration {
    type ParseError = humantime::DurationError;

    fn required() -> bool {
        true
    }

    fn parse(value: &str) -> Result<Self, Self::ParseError> {
        value.parse()
    }

    fn to_string(&self) -> String {
        ToString::to_string(self)
    }
}

impl<T> FieldType for Option<T>
where
    T: FieldType,
{
    type ParseError = T::ParseError;

    fn required() -> bool {
        false
    }

    fn base_validator() -> Option<Validator<String, ValidationResult>> {
        let base = T::base_validator();
        Some(Validator::from(move |ctx: ValidationContext<String>| {
            if !ctx.value.is_empty() {
                if let Some(base) = &base {
                    return base.run(ctx).unwrap_or_default();
                }
            }
            ValidationResult::ok()
        }))
    }

    fn parse(value: &str) -> Result<Self, Self::ParseError> {
        Ok(if value.is_empty() {
            None
        } else {
            Some(T::parse(value)?)
        })
    }

    fn to_string(&self) -> String {
        match self {
            Some(value) => value.to_string(),
            None => String::new(),
        }
    }
}

impl<T> FieldType for Vec<T>
where
    T: FieldType,
{
    type ParseError = T::ParseError;

    fn required() -> bool {
        false
    }

    fn parse(value: &str) -> Result<Self, Self::ParseError> {
        value.split(",").map(|s| T::parse(s.trim())).collect()
    }

    fn to_string(&self) -> String {
        self.iter().map(|v| v.to_string()).join(", ")
    }
}

impl<E: Epsilon64, const U: i64> FieldType for ApproxF64<E, U> {
    type ParseError = ParseFloatError;

    fn required() -> bool {
        true
    }

    fn parse(value: &str) -> Result<Self, Self::ParseError> {
        Ok(value.parse::<f64>()?.into())
    }

    fn to_string(&self) -> String {
        ToString::to_string(&self.0)
    }
}
