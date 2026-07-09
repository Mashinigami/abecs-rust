// Biblioteca para comunicação com Pinpad via Protocolo ABECS 2.12

pub mod command;
pub mod commands;
pub mod connection;
pub mod emv;
pub mod error;
pub mod protocol;
pub mod response;
pub mod serialize;
pub mod qrcode;
pub mod libpinpadso;
pub mod image;

mod pinpad_display;
mod pinpad_jni;
mod pinpad_multimedia;

// Re-exporta os tipos principais para facilitar o uso
pub use command::AbecsCommand as RawAbecsCommand;
pub use commands::AbecsCommand; // Novo namespace de comandos tipados
pub use connection::PinpadConnection;
pub use emv::{EmvData, EmvTag};
pub use error::AbecsError;
pub use response::AbecsResponse;
pub use serialize::{AbecsDeserialize, AbecsSerialize, AbecsTypedCommand};

// Re-exporta as respostas e estruturas
pub use commands::{
    CardType, EmptyResponse, FinishChipResponse, GetCardResponse, GetDataResponse, GetInfoResponse,
    GetKeyResponse, GetPinResponse, GetTracksResponse, GoOnChipResponse, ListMultimediaFilesResponse,
    MenuResponse, MultimediaFileType, PaymentMethod, Track1Data,
};

/// Tipo Result padrão da biblioteca
pub type Result<T> = std::result::Result<T, AbecsError>;
