use std::fmt;

/// All recoverable failures produced by parsing or calculation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EecalcError {
    UnknownCommand(String),
    MissingArgument(String),
    UnexpectedArgument(String),
    InvalidNumber { value: String, message: String },
    WrongArgumentCount { command: &'static str },
    NonPositiveValue { what: &'static str },
    DivisionByZero,
}

impl EecalcError {
    /// Process exit code: 1 for calculation errors, 2 for usage/parse errors.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::NonPositiveValue { .. } | Self::DivisionByZero => 1,
            _ => 2,
        }
    }
}

impl fmt::Display for EecalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownCommand(name) => write!(f, "unknown command '{name}'"),
            Self::MissingArgument(message) => write!(f, "{message}"),
            Self::UnexpectedArgument(token) => write!(f, "unexpected argument '{token}'"),
            Self::InvalidNumber { value, message } => {
                write!(f, "invalid value '{value}': {message}")
            }
            Self::WrongArgumentCount { command } => match *command {
                "ohm" => write!(
                    f,
                    "ohm requires exactly two of --voltage, --current, --resistance"
                ),
                "power" => write!(
                    f,
                    "power requires exactly two of --voltage, --current, --resistance"
                ),
                other => write!(f, "{other} requires at least two values"),
            },
            Self::NonPositiveValue { what } => write!(f, "{what} must be greater than zero"),
            Self::DivisionByZero => write!(f, "current must not be zero"),
        }
    }
}

impl std::error::Error for EecalcError {}
