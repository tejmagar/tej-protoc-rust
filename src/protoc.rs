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
    use crate::protoc::{File};

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
    use std::io::Read;
    use std::net::TcpStream;
    use crate::protoc::File;
    use crate::protoc::StatusCode::FirstBit;

    #[derive(Debug)]
    pub struct DecodedResponse {
        pub status: u8,
        pub app_status: u8,
        pub protocol_version: u8,
        pub number_of_files: u64,
        pub files: Vec<File>,
        pub message: Vec<u8>,
    }

    pub fn read_first_byte(tcp_stream: &mut TcpStream) -> (u8, u8) {
        let mut buffer = [0u8; 1];
        tcp_stream.read_exact(&mut buffer).unwrap();

        // Extract status codes
        let mixed_status = buffer[0];
        let status_code = mixed_status >> 7;
        let app_status = mixed_status & 0b01111111;
        return (status_code, app_status);
    }

    pub fn read_protocol_version(tcp_stream: &mut TcpStream) -> u8 {
        let mut buffer = [0u8; 1];
        tcp_stream.read_exact(&mut buffer).unwrap();
        return buffer[0];
    }

    pub fn read_files_count(tcp_stream: &mut TcpStream) -> u64 {
        let mut buffer = [0u8; 8];
        tcp_stream.read_exact(&mut buffer).unwrap();
        return u64::from_be_bytes(buffer);
    }

    pub fn read_filename_length(tcp_stream: &mut TcpStream) -> u16 {
        let mut buffer = [0u8; 2];
        tcp_stream.read_exact(&mut buffer).unwrap();
        return u16::from_be_bytes(buffer);
    }

    pub fn read_filename(tcp_stream: &mut TcpStream, filename_length: u16) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![0u8; filename_length as usize];
        tcp_stream.read_exact(&mut buffer).unwrap();
        return buffer;
    }

    pub fn read_file_size(tcp_stream: &mut TcpStream) -> u64 {
        let mut buffer = [0u8; 8];
        tcp_stream.read_exact(&mut buffer).unwrap();
        return u64::from_be_bytes(buffer);
    }

    pub fn read_file_data(tcp_stream: &mut TcpStream, file_size: u64) -> Vec<u8> {
        let mut buffer = vec![0u8; file_size as usize];
        tcp_stream.read_exact(&mut buffer).unwrap();
        return buffer;
    }

    pub fn read_files(tcp_stream: &mut TcpStream, num_files: u64) -> Vec<File> {
        let mut files: Vec<File> = Vec::new();

        for _ in 0..num_files {
            // Extract filename
            let filename_length = read_filename_length(tcp_stream);
            let filename = read_filename(tcp_stream, filename_length);

            // Extreact file data
            let file_size = read_file_size(tcp_stream);
            let file_data = read_file_data(tcp_stream, file_size).to_vec();

            let file = File::new(filename, file_data);
            files.push(file);
        }

        return files;
    }

    pub fn read_message_length(tcp_stream: &mut TcpStream) -> u64 {
        let mut buffer = [0u8; 8];
        tcp_stream.read_exact(&mut buffer).unwrap();
        return u64::from_be_bytes(buffer);
    }

    pub fn read_message(tcp_stream: &mut TcpStream, message_length: u64) -> Vec<u8> {
        let mut buffer = vec![0u8; message_length as usize];
        tcp_stream.read_exact(&mut buffer).unwrap();
        return buffer;
    }

    pub fn decode_tcp_stream(tcp_stream: &mut TcpStream) -> Result<DecodedResponse, String> {
        let (status, app_status) = read_first_byte(tcp_stream);
        if status != FirstBit as u8 {
            let error = format!("Invalid starting byte received. Expected 1 but received {}", status);
            eprintln!("{}", error);
            return Err(error);
        }

        let protocol_version = read_protocol_version(tcp_stream);
        let number_of_files = read_files_count(tcp_stream);
        let files = read_files(tcp_stream, number_of_files);
        let message_length = read_message_length(tcp_stream);
        let message = read_message(tcp_stream, message_length);

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
    use crate::protoc::{File};
    use crate::protoc::encoder::build_raw_bytes;
    use crate::protoc::StatusCode::FirstBit;

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
