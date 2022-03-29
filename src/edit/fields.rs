use patternfly_yew::*;
use std::{
    convert::Infallible,
    fmt::{Display, Formatter},
    num::ParseFloatError,
};

pub trait FieldType {
    type Type: ToString + 'static;
    type ParseError;

    fn required() -> bool;

    fn base_validator() -> Option<Validator<String, ValidationResult>>;
    fn parse(value: &str) -> Result<Self::Type, Self::ParseError>;
}

pub struct Optional<T>(pub Option<T>);

impl<T> From<Option<T>> for Optional<T> {
    fn from(value: Option<T>) -> Self {
        Optional(value)
    }
}

impl<T> Display for Optional<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(v) => v.fmt(f),
            None => Ok(()),
        }
    }
}

pub struct StringOptional;

impl FieldType for StringOptional {
    type Type = Optional<String>;
    type ParseError = Infallible;

    fn required() -> bool {
        false
    }

    fn base_validator() -> Option<Validator<String, ValidationResult>> {
        None
    }

    fn parse(value: &str) -> Result<Self::Type, Self::ParseError> {
        if value.is_empty() {
            Ok(None.into())
        } else {
            Ok(Some(value.to_string()).into())
        }
    }
}

pub struct StringRequired;

impl FieldType for StringRequired {
    type Type = String;
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

    fn parse(value: &str) -> Result<Self::Type, Self::ParseError> {
        Ok(value.to_string())
    }
}

pub struct FloatRequired;

impl FieldType for FloatRequired {
    type Type = f64;
    type ParseError = ParseFloatError;

    fn required() -> bool {
        true
    }

    fn base_validator() -> Option<Validator<String, ValidationResult>> {
        Some(Validator::from(|ctx: ValidationContext<String>| {
            if let Err(err) = Self::parse(&ctx.value) {
                ValidationResult::error(format!("Must be a floating-point number: {}", err))
            } else {
                ValidationResult::ok()
            }
        }))
    }

    fn parse(value: &str) -> Result<Self::Type, Self::ParseError> {
        value.parse::<Self::Type>()
    }
}

pub struct DurationRequired;

impl FieldType for DurationRequired {
    type Type = humantime::Duration;
    type ParseError = humantime::DurationError;

    fn required() -> bool {
        true
    }

    fn base_validator() -> Option<Validator<String, ValidationResult>> {
        Some(Validator::from(|ctx: ValidationContext<String>| {
            if let Err(err) = Self::parse(&ctx.value) {
                ValidationResult::error(format!("Must be a duration: {}", err))
            } else {
                ValidationResult::ok()
            }
        }))
    }

    fn parse(value: &str) -> Result<Self::Type, Self::ParseError> {
        value.parse::<Self::Type>()
    }
}
