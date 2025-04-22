use thiserror::Error;

#[derive(Debug, Error)]
pub enum IcgrError {
    #[error("UTF-8 decoding failed: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Overlap mismatch: expected {expected:?}, got {actual:?}")]
    OverlapMismatch { expected: String, actual: String },

    #[error("Chunk too short to contain required overlap")]
    ChunkTooShort,

    #[error("Failed to parse i128 value")]
    ParseError(#[from] std::num::ParseIntError),

    #[error("BICGR read/write failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("FASTA parsing failed: {0}")]
    Fasta(#[from] noodles::fasta::record::definition::ParseError),

    #[error("Unknown nucleotide encountered: {0}")]
    UnknownNucleotide(char),
}
