use std::io::Read;
#[allow(unused)]
use {
    super::error::AzuriteStorageError,
    azure_storage::{CloudLocation, StorageCredentials},
    azure_storage_blobs::{
        container::operations::list_blobs::BlobItem, prelude::*,
        service::operations::ListContainersResponse,
    },
    bytes::Bytes,
    clap::Parser,
    error_stack::{Context, Report, Result, ResultExt},
    futures::stream::{self, StreamExt},
    jlogger_tracing::{JloggerBuilder, LevelFilter, jdebug, jinfo},
};

const ACCOUNT_NAME: &str = "devstoreaccount1";
const ACCOUNT_KEY: &str =
    "Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==";

pub struct AzuriteStorage {
    runtime: tokio::runtime::Runtime,
    blob_service_client: BlobServiceClient,
}

#[allow(unused)]
impl AzuriteStorage {
    pub fn new(azurite_url: &str) -> Result<Self, AzuriteStorageError> {
        let runtime = tokio::runtime::Runtime::new().map_err(|e| {
            Report::new(AzuriteStorageError::RuntimeCreationFailed)
                .attach_printable("Failed to create Tokio runtime")
                .attach(e)
        })?;

        let credential = StorageCredentials::access_key(ACCOUNT_NAME, ACCOUNT_KEY);

        let (address, port) = azurite_url
            .trim_end_matches('/')
            .trim_start_matches("https://")
            .split_once(':')
            .map(|(address, port)| {
                let port: u16 = port
                    .parse()
                    .unwrap_or_else(|_| panic!("invalid port: {}", port));
                (address.to_owned(), port)
            })
            .ok_or_else(|| {
                Report::new(AzuriteStorageError::InvalidParameter(
                    azurite_url.to_owned(),
                ))
            })?;
        let client_builder =
            ClientBuilder::with_location(CloudLocation::Emulator { address, port }, credential);

        Ok(AzuriteStorage {
            runtime,
            blob_service_client: client_builder.blob_service_client(),
        })
    }

    pub fn list_containers(&self) -> Vec<String> {
        let mut result = Vec::new();
        self.runtime.block_on(async {
            let mut stream = self.blob_service_client.list_containers().into_stream();

            while let Some(Ok(response)) = stream.next().await {
                let ListContainersResponse {
                    containers,
                    next_marker: _,
                } = response;

                for container in containers {
                    result.push(container.name.clone());
                }
            }

            result
        })
    }

    pub fn create_container(&self, container_name: &str) -> Result<(), AzuriteStorageError> {
        self.runtime.block_on(async {
            self.blob_service_client
                .container_client(container_name)
                .create()
                .await
                .map_err(|e| {
                    Report::new(AzuriteStorageError::InternalError(format!(
                        "Failed to create container '{}': {}",
                        container_name, e
                    )))
                })
        })
    }

    pub fn delete_container(&self, container_name: &str) -> Result<(), AzuriteStorageError> {
        self.runtime.block_on(async {
            self.blob_service_client
                .container_client(container_name)
                .delete()
                .await
                .map_err(|e| {
                    Report::new(AzuriteStorageError::InternalError(format!(
                        "Failed to delete container '{}': {}",
                        container_name, e
                    )))
                })
        })
    }

    pub fn container_url(&self, container_name: &str) -> Result<String, AzuriteStorageError> {
        self.runtime.block_on(async {
            Ok(self
                .blob_service_client
                .container_client(container_name)
                .url()
                .map_err(|e| {
                    Report::new(AzuriteStorageError::InternalError(format!(
                        "Failed to get URL for container '{}': {}",
                        container_name, e
                    )))
                })?
                .path()
                .to_owned())
        })
    }

    pub fn push_blob(
        &self,
        container_name: &str,
        file_path: &str,
    ) -> Result<(), AzuriteStorageError> {
        let file = std::fs::File::open(file_path).map_err(|e| {
            AzuriteStorageError::InternalError(format!("Failed to open file: {}", e))
        })?;
        let mut reader = std::io::BufReader::new(file);
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).map_err(|e| {
            AzuriteStorageError::InternalError(format!("Failed to read file: {}", e))
        })?;

        let blob_client = self
            .blob_service_client
            .container_client(container_name)
            .blob_client(file_path);

        self.runtime.block_on(async {
            blob_client
                .put_block_blob(Bytes::from(buf))
                .await
                .map_err(|e| {
                    Report::new(AzuriteStorageError::InternalError(format!(
                        "Failed to upload file to container '{}': {}",
                        container_name, e
                    )))
                })
        })?;

        Ok(())
    }

    pub fn list_blobs(&self, container_name: &str) -> Result<Vec<Blob>, AzuriteStorageError> {
        self.runtime.block_on(async {
            let mut result = Vec::new();
            let mut stream = self
                .blob_service_client
                .container_client(container_name)
                .list_blobs()
                .into_stream();

            while let Some(Ok(response)) = stream.next().await {
                for blob in response.blobs.items.iter() {
                    if let BlobItem::Blob(blob_item) = blob {
                        result.push(blob_item.clone());
                    } else {
                        jdebug!("Skipping non-blob item in list: {:?}", blob);
                    }
                }
            }

            Ok(result)
        })
    }
}
