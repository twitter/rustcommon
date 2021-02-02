// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::*;

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum MetricsError {
    #[error("metric ({0}) is not registered")]
    NotRegistered(String),
    #[error("metric ({0}) is a ({1}) not a ({2})")]
    WrongType(String, Source, Source),
    #[error("metrics have not been initialized")]
    Unitialized,
    #[error("metrics have already been initialized")]
    AlreadyInitialized,
}