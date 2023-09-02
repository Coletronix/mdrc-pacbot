use crate::robomodules::protos;
use protobuf::Message;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

mod robomodules;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8888").await?;

    // Assuming the server sends the message length first as a 4-byte value
    let mut length_buf = [0u8; 4];
    stream.read_exact(&mut length_buf).await?;
    let length = u32::from_be_bytes(length_buf);

    let mut buffer = vec![0u8; length as usize];
    stream.read_exact(&mut buffer).await?;

    // Deserialize using protobuf generated code
    let message = protos::lightState::LightState::parse_from_bytes(&buffer)?;

    // Now you can access the fields of the `message`
    println!("{:?}", message);

    Ok(())
}
