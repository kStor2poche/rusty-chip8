use core::fmt;
use std::error::Error;

#[derive(Debug)]
pub struct InvalidAccessError {
    reason: String,
}

impl InvalidAccessError {
    pub fn new<T>(reason: T) -> Self
    where
        T: ToString,
    {
        Self {
            reason: reason.to_string(),
        }
    }
}
impl fmt::Display for InvalidAccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InvalidAccessError : {}", self.reason)
    }
}
impl Error for InvalidAccessError {}

#[derive(Debug)]
pub struct InvalidInstructionError {
    reason: String,
}

impl InvalidInstructionError {
    pub fn new<T>(reason: T) -> Self
    where
        T: ToString,
    {
        Self {
            reason: reason.to_string(),
        }
    }
}
impl fmt::Display for InvalidInstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InvalidInstructionError : {}", self.reason)
    }
}
impl Error for InvalidInstructionError {}

#[derive(Debug)]
pub struct ProgramLoadingError {
    reason: String,
}

impl ProgramLoadingError {
    pub fn new<T>(reason: T) -> Self
    where
        T: ToString,
    {
        Self {
            reason: reason.to_string(),
        }
    }
}
impl fmt::Display for ProgramLoadingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProgramLoadingError : {}", self.reason)
    }
}
impl Error for ProgramLoadingError {}
