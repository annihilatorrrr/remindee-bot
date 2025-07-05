use crate::db;
use std::fmt;

#[derive(Debug)]
pub(crate) enum Error {
    Database(db::Error),
    Parse(chrono_tz::ParseError),
    CronParse(cron_parser::ParseError),
    TeloxideRequest(teloxide::RequestError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Database(ref err) => write!(f, "Database error: {err}"),
            Self::Parse(ref err) => write!(f, "Parse error: {err}"),
            Self::CronParse(ref err) => write!(f, "Cron parse error: {err}"),
            Self::TeloxideRequest(ref err) => {
                write!(f, "Telegram request error: {err}")
            }
        }
    }
}

impl From<db::Error> for Error {
    fn from(err: db::Error) -> Self {
        Self::Database(err)
    }
}

impl From<cron_parser::ParseError> for Error {
    fn from(err: cron_parser::ParseError) -> Self {
        Self::CronParse(err)
    }
}

impl From<teloxide::RequestError> for Error {
    fn from(err: teloxide::RequestError) -> Self {
        Self::TeloxideRequest(err)
    }
}

impl std::error::Error for Error {}
