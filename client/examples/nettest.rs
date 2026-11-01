// Проверка сети без окна: два клиента, видимость, общий мир, рынок, рейтинг.
#[path = "../src/net.rs"]
mod net;
use net::{Net, NetEvent};
use shared::{ClientMsg, ServerMsg};
use std::time::{Duration, Instant};

#[derive(Default)]
struct Seen {
    id: u32,
    both: bool,
    dead: bool,
    market: usize,
    posted: bool,
    sold: u32,
    bought: bool,
    board: bool,
    boat_coat_held: bool,
    err: bool,
}

fn pump(net: &Net, s: &mut Seen) {
    while let Ok(ev) = net.in_rx.try_recv() {
        if let NetEvent::Msg(m) = ev {
            match m {
                ServerMsg::Welcome { id } => s.id = id,
                ServerMsg::Snapshot { players } => {
                    if players.len() == 2 {
                        s.both = true;
                    }
                    if players.iter().any(|p| p.boat && p.coat && p.held == 2 && p.name == "Боб<script>") {
                        s.boat_coat_held = true;
                    }
                }
                ServerMsg::Dead { idx: 77, left } if left > 0 => s.dead = true,
                ServerMsg::World { dead } if dead.iter().any(|d| d.0 == 77) => s.dead = true,
                ServerMsg::Market { offers } => s.market = offers.len(),
                ServerMsg::Posted { .. } => s.posted = true,
                ServerMsg::Sold { gold, .. } => s.sold = gold,
                ServerMsg::Bought { .. } => s.bought = true,
                ServerMsg::Board { .. } => s.board = true,
                ServerMsg::Err { .. } => s.err = true,
                _ => {}
            }
        }
    }
}

fn main() {
    let url = std::env::args().nth(1).unwrap_or("ws://127.0.0.1:8080/ws".into());
    let a = Net::start(url.clone(), "Алиса".into());
    let b = Net::start(url, "Боб<script>".into());
    let (mut sa, mut sb) = (Seen::default(), Seen::default());
    let t0 = Instant::now();
    let mut step = 0;
    while t0.elapsed() < Duration::from_secs(9) {
        pump(&a, &mut sa);
        pump(&b, &mut sb);
        let e = t0.elapsed().as_millis();
        if step == 0 && e > 800 {
            a.send(ClientMsg::Move { x: 25.0, y: 22.5, face: 3, moving: true, boat: false, coat: false, held: 0 });
            b.send(ClientMsg::Move { x: 21.0, y: 21.0, face: 1, moving: false, boat: true, coat: true, held: 2 });
            a.send(ClientMsg::Dead { idx: 77 });
            a.send(ClientMsg::Stats { score: 120, big: 3.5, fish: 4 });
            a.send(ClientMsg::Post { item: 12, qty: 3, price: 10 });
            step = 1;
        }
        if step == 1 && e > 1800 && sb.market == 1 {
            b.send(ClientMsg::Buy { id: 1 });
            b.send(ClientMsg::Buy { id: 1 }); // повторная покупка того же — должна дать «Уже продано»
            step = 2;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    println!("видят друг друга: А={} Б={}", sa.both, sb.both);
    println!("добытый ресурс виден другому игроку: {}", sb.dead);
    println!("предложение выставлено: {}, у Боба видно предложений: {}", sa.posted, sb.market);
    println!("покупка: Боб купил={}, Алиса получила золота={}", sb.bought, sa.sold);
    println!("повторная покупка отклонена: {}", sb.err);
    println!("таблица лидеров пришла: {}", sa.board && sb.board);
    println!("лодка/одежда/предмет в руке передаются: {}", sa.boat_coat_held);
}
