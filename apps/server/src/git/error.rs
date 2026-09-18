use std::error::Error;
use std::fmt::{self, Display, Formatter};

use git2::{Error as Git2Error, ErrorCode};

#[derive(Debug)]
pub enum GitError {
    NotFound(String),
    Invalid(String),
    Internal(String),
    Git(Git2Error),
}

impl Display for GitError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(message) | Self::Invalid(message) | Self::Internal(message) => {
                formatter.write_str(message)
            }
            Self::Git(error) => formatter.write_str(error.message()),
        }
    }
}

impl Error for GitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Git(error) => Some(error),
            _ => None,
        }
    }
}

impl From<Git2Error> for GitError {
    fn from(error: Git2Error) -> Self {
        match error.code() {
            ErrorCode::NotFound => Self::NotFound(error.message().to_owned()),
            ErrorCode::InvalidSpec => Self::Invalid(error.message().to_owned()),
            _ => Self::Git(error),
        }
    }
}
