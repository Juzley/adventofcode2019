use intcode::Program;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

type Packet = (i64, i64);
type Addr = usize;
type PacketQueue = HashMap<Addr, VecDeque<Packet>>;

const NODE_COUNT: usize = 50;
const NAT_INPUT_ADDR: usize = 0;
const NAT_OUTPUT_ADDR: usize = 255;

fn recv(node: Addr, packets: &mut PacketQueue) -> Option<Packet> {
    if let Some(queue) = packets.get_mut(&node) {
        if let Some(packet) = queue.pop_front() {
            Some(packet)
        } else {
            None
        }
    } else {
        None
    }
}

fn send(node: Addr, packet: Packet, packets: &mut PacketQueue) {
    if let Some(queue) = packets.get_mut(&node) {
        queue.push_back(packet);
    } else {
        packets.insert(node, VecDeque::from(vec![packet]));
    }
}

fn main() {
    let mut nodes = vec![Program::from_file("input"); NODE_COUNT];

    // Initialize the nodes
    for (i, p) in nodes.iter_mut().enumerate() {
        let mut init = false;
        while !init {
            let _ = p.step(
                &mut || {
                    init = true;
                    i as i64
                },
                &mut |_| {},
            );
        }
    }

    let packets = RefCell::new(HashMap::new());
    let mut nat = None;
    let mut nat_ys = HashSet::new();
    loop {
        let mut idle = true;

        for (node, p) in nodes.iter_mut().enumerate() {
            let mut send_buffer = Vec::new();
            let mut recv_buffer = None;

            loop {
                let mut input = || match recv_buffer {
                    Some(y) => {
                        recv_buffer = None;
                        y
                    }
                    None => {
                        let mut packets = packets.borrow_mut();
                        match recv(node, &mut *packets) {
                            Some((x, y)) => {
                                recv_buffer = Some(y);
                                x
                            }
                            None => -1,
                        }
                    }
                };

                let mut output = |val| {
                    idle = false;
                    send_buffer.push(val);

                    if send_buffer.len() == 3 {
                        let mut iter = send_buffer.iter();
                        let addr = *iter.next().unwrap() as Addr;
                        let x = *iter.next().unwrap();
                        let y = *iter.next().unwrap();
                        let packet = (x, y);

                        if addr == NAT_OUTPUT_ADDR {
                            nat = Some(packet);
                        } else {
                            let mut packets = packets.borrow_mut();
                            send(addr, packet, &mut *packets);
                        }

                        send_buffer.clear();
                    }
                };

                let _ = p.step(&mut input, &mut output);

                // If we're not sending or receiving a packet, go to the next machine.
                if send_buffer.is_empty() && recv_buffer.is_none() {
                    break;
                }
            }
        }

        // If nothing's sending packets and there are no packets left to be processed,
        // inject a packet from the NAT.
        idle = idle
            && packets
                .borrow()
                .iter()
                .fold(true, |acc, (k, q): (&Addr, &VecDeque<Packet>)| {
                    acc && q.is_empty()
                });
        if idle && nat.is_some() {
            // Nothing sending and all packet queues are empty.
            if nat_ys.contains(&nat.unwrap().1) {
                println!("Result: {}", nat.unwrap().1);
                break;
            } else {
                nat_ys.insert(nat.unwrap().1);
                let mut packets = packets.borrow_mut();
                send(NAT_INPUT_ADDR, nat.unwrap(), &mut *packets);
                nat = None;
            }
        }
    }
}
