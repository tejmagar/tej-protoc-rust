pub enum StatusCode {
    FirstBit = 1,
    Ping = 2,
}

#[derive(Debug)]
pub struct File {
    pub name: Vec<u8>,
    pub data: Vec<u8>,
}

impl File {
    pub fn new(name: Vec<u8>, data: Vec<u8>) -> Self {
        File { name, data }
    }
}

pub mod encoder {
    use crate::protoc::File;

    pub fn build_raw_bytes(app_status: u8, protocol_version: u8, files: &Vec<&File>,
                           message: &Vec<u8>) -> Vec<u8> {
        let mut bytes_buffer: Vec<u8> = Vec::new();

        // Construct bytes
        let status_byte = app_status | 0b10000000;
        bytes_buffer.push(status_byte);
        bytes_buffer.push(protocol_version);

        let number_of_files: u64 = files.len() as u64;
        bytes_buffer.extend_from_slice(&number_of_files.to_be_bytes());

        for file in files {
            // Push filename length bytes to the buffer
            let filename_length: u16 = file.name.len() as u16;
            bytes_buffer.extend_from_slice(&filename_length.to_be_bytes());

            // Push filename to buffer
            let filename = &file.name;
            bytes_buffer.extend_from_slice(&filename);

            // Push file bytes length to buffer
            let file_length: u64 = file.data.len() as u64;
            bytes_buffer.extend_from_slice(&file_length.to_be_bytes());

            // Push actual file bytes to buffer
            bytes_buffer.extend_from_slice(&file.data);
        }

        // Push message length to the buffer
        let message_length: u64 = message.len() as u64;
        bytes_buffer.extend_from_slice(&message_length.to_be_bytes());

        // Push message
        bytes_buffer.extend_from_slice(&message);
        return bytes_buffer;
    }

    pub fn build_bytes(files: Option<&Vec<&File>>, message: Option<&Vec<u8>>) -> Vec<u8> {
        let mut tmp_files: &Vec<&File> = &vec![];
        let mut tmp_message: &Vec<u8> = &vec![];

        if files.is_some() {
            tmp_files = files.unwrap();
        }

        if message.is_some() {
            tmp_message = message.unwrap();
        }

        return build_raw_bytes(0, 1, &tmp_files, &tmp_message);
    }

    pub fn build_bytes_for_message(message: &Vec<u8>) -> Vec<u8> {
        return build_bytes(None, Some(message));
    }

    pub fn build_bytes_for_files(files: &Vec<&File>) -> Vec<u8> {
        return build_bytes(Some(files), None);
    }
}

pub mod decoder {
    use std::io::ErrorKind;
    use std::sync::Arc;


    use crate::protoc::File;
    use crate::protoc::StatusCode::FirstBit;
    use crate::stream::Stream;

    #[derive(Debug)]
    pub struct DecodedResponse {
        pub status: u8,
        pub app_status: u8,
        pub protocol_version: u8,
        pub number_of_files: u64,
        pub files: Vec<File>,
        pub message: Vec<u8>,
    } 

    pub async fn read_first_byte(stream: Arc<Stream>) -> std::io::Result<(u8, u8)> {
        let bytes = stream.read_exact(1).await?;

        // Extract status codes
        let mixed_status = bytes[0];
        let status_code = mixed_status >> 7;
        let app_status = mixed_status & 0b01111111;
        Ok((status_code, app_status))
    }

    pub async fn read_protocol_version(stream: Arc<Stream>) -> std::io::Result<u8> {
        let bytes = stream.read_exact(1).await?;
        Ok(bytes[0])
    }

    pub async fn read_files_count(stream: Arc<Stream>) -> std::io::Result<u64> {
        let bytes = stream.read_exact(8).await?;
        Ok(u64::from_be_bytes(bytes.try_into().unwrap()))
    }

    pub async fn read_filename_length(stream: Arc<Stream>) -> std::io::Result<u16> {
        let bytes = stream.read_exact(2).await?;
        Ok(u16::from_be_bytes(bytes.try_into().unwrap()))
    }

    pub async fn read_filename(stream: Arc<Stream>, filename_length: u16) -> std::io::Result<Vec<u8>> {
        let bytes = stream.read_exact(filename_length as usize).await?;
        Ok(bytes)
    }

    pub async fn read_file_size(stream: Arc<Stream>) -> std::io::Result<u64> {
        let bytes = stream.read_exact(8).await?;
        Ok(u64::from_be_bytes(bytes.try_into().unwrap()))
    }

    pub async fn read_file_data(stream: Arc<Stream>, file_size: u64) -> std::io::Result<Vec<u8>> {
        let bytes = stream.read_exact(file_size as usize).await?;
        Ok(bytes)
    }

    pub async fn read_files(stream: Arc<Stream>, num_files: u64) -> std::io::Result<Vec<File>> {
        let mut files: Vec<File> = Vec::new();

        for _ in 0..num_files {
            // Extract filename
            let filename_length = read_filename_length(stream.clone()).await?;
            let filename = read_filename(stream.clone(), filename_length).await?;

            // Extreact file data
            let file_size = read_file_size(stream.clone()).await?;
            let file_data = read_file_data(stream.clone(), file_size).await?.to_vec();

            let file = File::new(filename, file_data);
            files.push(file);
        }

        Ok(files)
    }

    pub async fn read_message_length(stream: Arc<Stream>) -> std::io::Result<u64> {
        let bytes = stream.read_exact(8).await?;
        return match bytes.try_into() {
            Ok(bytes) => {
                let bytes: [u8; 8] = bytes;
                Ok(u64::from_be_bytes(bytes))
            }
            Err(_) => {
                Err(std::io::Error::new(ErrorKind::Other, "Failed to read message length."))
            }
        };
    }

    pub async fn read_message(stream: Arc<Stream>, message_length: u64) -> std::io::Result<Vec<u8>> {
        let bytes = stream.read_exact(message_length as usize).await?;
        Ok(bytes)
    }

    pub async fn decode_tcp_stream(stream: Arc<Stream>) -> std::io::Result<DecodedResponse> {
        let (status, app_status) = read_first_byte(stream.clone()).await?;
        if status != FirstBit as u8 {
            let error = format!("Invalid starting byte received. Expected 1 but received {}", status);
            return Err(std::io::Error::new(ErrorKind::Other, error));
        }

        let protocol_version = read_protocol_version(stream.clone()).await?;
        let number_of_files = read_files_count(stream.clone()).await?;
        let files = read_files(stream.clone(), number_of_files).await?;

        let message_length = read_message_length(stream.clone()).await?;
        let message = read_message(stream.clone(), message_length).await?;

        return Ok(DecodedResponse {
            status,
            app_status,
            protocol_version,
            number_of_files,
            files,
            message,
        });
    }
}

#[cfg(test)]
pub mod tests {
    use std::fs;
    use std::io::Read;
    use crate::protoc::File;
    use crate::protoc::encoder::build_raw_bytes;

    #[test]
    pub fn test_build_raw_bytes() {
        let mut files: Vec<&File> = vec![];
        let mut tmp_file = fs::File::open(".gitignore").unwrap();

        let mut file_buffer = Vec::new();
        tmp_file.read_to_end(&mut file_buffer).unwrap();

        let file = File::new("hello".as_bytes().to_vec(), file_buffer);
        files.push(&file);

        let raw_bytes = build_raw_bytes(0, 1, &files, &"".as_bytes().to_vec());
        print!("{:?}", raw_bytes);
    }
}
