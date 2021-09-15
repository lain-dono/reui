#[derive(Debug)]
pub enum Error {
    Parse(ttf_parser::FaceParsingError),
    Io(std::io::Error),
    InfoExtracion,
    NotFound,
    Unknown,
    FontSizeTooLargeForAtlas,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}
