use bytes::BytesMut;
use smol_str::SmolStr;
use tokio::io::{AsyncRead, AsyncReadExt};

use super::{Client, ClientError};
use crate::{
    api::commands::file::{CreateFile, CreateFileBody},
    models::Snowflake,
};

impl Client {
    /// Upload a plain file from its handle
    ///
    /// This does not do any extra handling for media files,
    /// such as finding dimensions or generating previews.
    #[cfg(feature = "fs")]
    pub async fn upload_plain_file(
        &self,
        filename: impl Into<SmolStr>,
        mime: Option<mime::Mime>,
        file: &mut tokio::fs::File,
        progress: impl FnMut(u64),
    ) -> Result<Snowflake, ClientError> {
        let meta = file.metadata().await?;

        if !meta.is_file() {
            return Err(ClientError::NotAFile);
        }

        let meta = CreateFileBody {
            filename: filename.into(),
            size: match i32::try_from(meta.len()) {
                Ok(size) => size,
                Err(_) => return Err(ClientError::FileTooLarge),
            },
            width: None,
            height: None,
            mime: mime.map(SmolStr::from),
            preview: None,
        };

        self.upload_stream(meta, file, progress).await
    }

    /// Uploads a file stream in chunks
    pub async fn upload_stream(
        &self,
        meta: CreateFileBody,
        file: impl AsyncRead,
        mut progress: impl FnMut(u64),
    ) -> Result<Snowflake, ClientError> {
        let file_size = meta.size as u64;
        let file_id = self.raw_driver().execute(CreateFile { body: meta }).await?;

        // TODO: Retrieve chunk size from server?
        let mut buffer = BytesMut::with_capacity(1024 * 1024 * 8); // 8MiB
        let mut read = 0;

        tokio::pin!(file);

        while 0 != file.read_buf(&mut buffer).await? {
            read += buffer.len() as u64;

            let mut crc32 = crc32fast::Hasher::new();
            crc32.update(&buffer);

            let offset = self
                .raw_driver()
                .patch_file(file_id, crc32.finalize(), read, buffer.split().freeze().into())
                .await?;

            if offset != read {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Upload request returned unexpected offset",
                )
                .into());
            }

            progress(read);
        }

        if file_size != read {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "File stream terminated too early",
            )
            .into());
        }

        Ok(file_id)
    }
}
