//! Сетевой поток: подключение к серверу (ws:// и защищённый wss://), автопереподключение.
use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::Duration,
};

use shared::{ClientMsg, DeviceInfo, ServerMsg};
use tungstenite::{connect, stream::MaybeTlsStream, Message};

pub enum NetEvent {
    Connected,
    Disconnected,
    Msg(ServerMsg),
}

pub struct Net {
    out_tx: Sender<ClientMsg>,
    pub in_rx: Receiver<NetEvent>,
}

fn set_timeout(s: &mut MaybeTlsStream<std::net::TcpStream>) {
    let t = Some(Duration::from_millis(30));
    match s {
        MaybeTlsStream::Plain(s) => {
            let _ = s.set_read_timeout(t);
            let _ = s.set_nodelay(true);
        }
        MaybeTlsStream::Rustls(s) => {
            let _ = s.sock.set_read_timeout(t);
            let _ = s.sock.set_nodelay(true);
        }
        _ => {}
    }
}

impl Net {
    pub fn start(url: String, name: String) -> Net {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let (out_tx, out_rx) = channel::<ClientMsg>();
        let (in_tx, in_rx) = channel::<NetEvent>();

        thread::spawn(move || {
            let mut name = name;
            loop {
                match connect(url.as_str()) {
                    Ok((mut ws, _)) => {
                        set_timeout(ws.get_mut());
                        // Старые движения не нужны, но смену имени запоминаем.
                        while let Ok(m) = out_rx.try_recv() {
                            if let ClientMsg::Join { name: n, .. } = m {
                                name = n;
                            }
                        }
                        let _ = in_tx.send(NetEvent::Connected);
                        let join = serde_json::to_string(&ClientMsg::Join {
                            name: name.clone(),
                            device_info: Some(DeviceInfo::detect()),
                        }).unwrap();
                        let _ = ws.send(Message::Text(join));

                        'conn: loop {
                            while let Ok(m) = out_rx.try_recv() {
                                if let ClientMsg::Join { name: n, .. } = &m {
                                    name = n.clone();
                                }
                                let txt = serde_json::to_string(&m).unwrap();
                                if ws.send(Message::Text(txt)).is_err() {
                                    break 'conn;
                                }
                            }
                            match ws.read() {
                                Ok(Message::Text(t)) => {
                                    if let Ok(m) = serde_json::from_str::<ServerMsg>(&t) {
                                        let _ = in_tx.send(NetEvent::Msg(m));
                                    }
                                }
                                Ok(Message::Close(_)) => break 'conn,
                                Ok(_) => {}
                                Err(tungstenite::Error::Io(e))
                                    if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {}
                                Err(_) => break 'conn,
                            }
                        }
                        let _ = in_tx.send(NetEvent::Disconnected);
                    }
                    Err(_) => {
                        let _ = in_tx.send(NetEvent::Disconnected);
                    }
                }
                // Если ждали пробуждения сервера, не спамим подключениями.
                while let Ok(m) = out_rx.try_recv() {
                    if let ClientMsg::Join { name: n, .. } = m {
                        name = n;
                    }
                }
                thread::sleep(Duration::from_secs(3));
            }
        });

        Net { out_tx, in_rx }
    }

    pub fn send(&self, m: ClientMsg) {
        let _ = self.out_tx.send(m);
    }
}
