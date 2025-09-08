use thiserror::Error;

/// Generic error type used by service layer functions.
#[derive(Debug, Error)]
pub enum ServiceError {
    /// The user is not authorized to perform the operation.
    #[error("unauthorized")]
    Unauthorized,

    /// Requested resource was not found.
    #[error("not found")]
    NotFound,

    /// Requested resource was not found.
    #[error("conflict")]
    Conflict,

    /// Persistence layer failures.
    #[error("repository error: {0}")]
    Repository(#[from] pushkind_common::repository::errors::RepositoryError),

    /// ZmqSenderError
    #[error("repository error: {0}")]
    ZmqSender(#[from] pushkind_common::zmq::ZmqSenderError),

    /// Form validation error.
    #[error("form error: {0}")]
    Form(String),

    /// Problems with environment or configuration.
    #[error("configuration error: {0}")]
    Config(String),

    /// An unexpected internal error occurred.
    #[error("internal error")]
    Internal,
}

/// Convenient alias for results returned from service functions.
pub type ServiceResult<T> = Result<T, ServiceError>;
