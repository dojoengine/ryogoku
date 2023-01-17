#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("ryogoku CRDs are not installed")]
    CrdNotInstalled,
    #[error("Kube error: {0}")]
    Kube(#[from] kube::Error),
    #[error("Finalizer error: {0}")]
    Finalizer(#[source] Box<kube::runtime::finalizer::Error<Error>>),
}

pub type Result<T> = std::result::Result<T, Error>;
