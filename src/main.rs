use crate::robomodules::message::MsgType;
use crate::robomodules::Client;

mod robomodules;

fn main() {
    let mut client = Client::new("localhost", 11297);
    client.subscribe(MsgType::LightState).unwrap();

    let msg = client.msg_receiver.recv().unwrap();

    println!("{:?}", msg);

    client.unsubscribe(MsgType::LightState).unwrap();

    // wait 2s
    std::thread::sleep(std::time::Duration::from_secs(2));

    client.subscribe(MsgType::FullState).unwrap();

    let msg = client.msg_receiver.recv().unwrap();

    println!("{:?}", msg);

    client.close().unwrap();
}
