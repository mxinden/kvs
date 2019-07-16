/// Result type returned by the KvStore library.
pub type Result<T> = std::result::Result<T, KvStoreError>;

/// Error returned by the KvStore library.
#[derive(Debug, Fail)]
pub enum KvStoreError {
    /// Generic failure. TODO remove!
    #[fail(display = "generic failure")]
    GenericFailure,

    /// Failure when opening database file.
    #[fail(display = "failed to open file {}", name)]
    OpenFileFailure {
        /// Underlying io Error.
        #[cause]
        c: std::io::Error,
        /// Name of the file.
        name: String,
    },

    /// Failure when opening temporary file.
    #[fail(display = "failed to open temporary file")]
    OpenTmpDirFailure {
        /// Underlying io Error.
        #[cause]
        c: std::io::Error,
    },

    /// Failure when move file.
    #[fail(display = "failed to move file")]
    FileMoveFailure {
        /// Underlying io Error.
        #[cause]
        c: std::io::Error,
    },

    /// Failure when flush file.
    #[fail(display = "failed to flush file")]
    FileFlushFailure {
        /// Underlying io Error.
        #[cause]
        c: std::io::Error,
    },

    /// Failure when serializing input.
    #[fail(display = "failed to serialize input")]
    SerializationFailure {
        /// Underlying io Error.
        #[cause]
        c: serde_json::error::Error,
    },

    /// Failure when deserializing input.
    #[fail(display = "failed to deserialize input")]
    DeserializationFailure {
        /// Underlying io Error.
        #[cause]
        c: serde_json::error::Error,
    },

    /// Failure writing to file.
    #[fail(display = "failed to write to file")]
    WriteToFileFailure {
        /// Underlying io Error.
        #[cause]
        c: std::io::Error,
    },

    /// Failure seeking file.
    #[fail(display = "failed to seek file")]
    SeekFileFailure {
        /// Underlying io Error.
        #[cause]
        c: std::io::Error,
    },

    /// Failure finding key
    #[fail(display = "Key not found")]
    KeyNotFound,
}
