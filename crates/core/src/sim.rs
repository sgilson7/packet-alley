//! The network simulation. A level, with the student's edits applied, becomes
//! a network of nodes joined by cables; time moves in ticks; every frame that
//! crosses a cable is recorded, with every delivery, drop, collision and
//! table change. The page plays the record back and decides nothing.
//!
//! A tick, in order:
//! 1. programs on the hosts make their sends (`start_apps`);
//! 2. frames that arrive this tick are handled by the node they reach (a hub
//!    that receives two at once has a collision and both are lost);
//! 3. each port of each node puts at most one frame on its cable.
//!
//! Addresses: a host finds the MAC address for an IP address on its own
//! block by asking the level (an instant ARP), except in levels whose
//! program is `arp`, where the request and the reply are real broadcast
//! frames. The lessons say so out loud.

use std::collections::VecDeque;

use crate::ip::{parse_cidr, parse_ip, show_ip, show_mac, Cidr, Ip, BROADCAST_MAC};
use crate::level::Level;
use crate::rng::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Host,
    Hub,
    Switch,
    Router,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flags {
    pub syn: bool,
    pub ack: bool,
    pub fin: bool,
    pub rst: bool,
}

impl Flags {
    pub fn parse(s: &str) -> Option<Flags> {
        let mut f = Flags::default();
        for part in s.split('-') {
            match part {
                "SYN" => f.syn = true,
                "ACK" => f.ack = true,
                "FIN" => f.fin = true,
                "RST" => f.rst = true,
                _ => return None,
            }
        }
        Some(f)
    }

    pub fn show(self) -> String {
        let mut v = Vec::new();
        if self.syn {
            v.push("SYN");
        }
        if self.ack {
            v.push("ACK");
        }
        if self.fin {
            v.push("FIN");
        }
        if self.rst {
            v.push("RST");
        }
        v.join("-")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Proto {
    Udp,
    Ping {
        reply: bool,
        n: u32,
    },
    Tcp {
        flags: Flags,
        sport: u32,
        dport: u32,
        seq: u32,
        ack: u32,
        data: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub src: Ip,
    pub dst: Ip,
    pub ttl: u32,
    pub proto: Proto,
    /// The program (index into the level's apps) this packet belongs to.
    pub app: usize,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body {
    Arp { reply: bool, sender: Ip, target: Ip },
    Ip(Packet),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub src_mac: u32,
    pub dst_mac: u32,
    pub body: Body,
    /// How many collisions this frame has already been through.
    pub attempt: u32,
}

impl Frame {
    /// The class the page draws the frame with.
    pub fn class(&self) -> &'static str {
        match &self.body {
            Body::Arp { reply: false, .. } => "broadcast",
            Body::Arp { reply: true, .. } => "arp",
            Body::Ip(p) => match &p.proto {
                Proto::Udp => "data",
                Proto::Ping { reply: false, .. } => "ping",
                Proto::Ping { reply: true, .. } => "reply",
                Proto::Tcp { flags, data, .. } => {
                    if flags.rst {
                        "rst"
                    } else if flags.syn && flags.ack {
                        "synack"
                    } else if flags.syn {
                        "syn"
                    } else if *data > 0 {
                        "seg"
                    } else {
                        "ack"
                    }
                }
            },
        }
    }
}

/// One crossing of one cable.
#[derive(Debug, Clone)]
pub struct Transit {
    pub link: usize,
    pub from: usize,
    pub to: usize,
    pub t0: u32,
    pub t1: u32,
    pub frame: Frame,
    pub lost: bool,
}

/// Something that happened, for the log and the effects on the map. `key` is
/// a copy key in `data/copy.json`; `args` fill its `{a}`, `{b}`, `{c}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub t: u32,
    pub node: usize,
    pub key: &'static str,
    pub args: Vec<String>,
    /// deliver, drop, collision, ignore, learn, info
    pub fx: &'static str,
    /// A frame lost on a cable is drawn where it was lost.
    pub transit: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stats {
    pub delivered: Vec<u32>,
    pub replies: Vec<u32>,
    pub served: Vec<bool>,
    pub transferred: Vec<bool>,
    pub drops: u32,
    pub collisions: u32,
    pub retransmits: u32,
    pub frames: u32,
}

/// One titled table in a node's inspector (a switch's MAC table, a router's
/// routes). Column headers and the title are copy keys; the page translates
/// them and draws the grid. `empty` (a copy key) is shown when there are no
/// rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoTable {
    pub title: &'static str,
    pub cols: Vec<&'static str>,
    pub rows: Vec<Vec<String>>,
    pub empty: Option<&'static str>,
}

/// A fact's value: either a literal (an address, a count — data, not words) or
/// a copy key the page translates (e.g. "(none)").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FactVal {
    Text(String),
    Copy(&'static str),
}

/// What a node knows, for its inspector: a set of plain facts (each a copy key
/// label and a value) and any titled tables. The page draws facts as a
/// key/value grid and each table as a grid, all in the game's own style.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeInfo {
    pub facts: Vec<(&'static str, FactVal)>,
    pub tables: Vec<InfoTable>,
}

#[derive(Debug, Clone)]
pub struct Record {
    pub ticks: u32,
    pub transits: Vec<Transit>,
    pub events: Vec<Event>,
    /// Per tick (index = tick), per node: what its inspector shows.
    pub tables: Vec<Vec<NodeInfo>>,
    /// Per tick: the totals at the end of that tick.
    pub stats_at: Vec<Stats>,
}

impl Record {
    pub fn stats(&self) -> &Stats {
        self.stats_at.last().expect("a record has at least one tick")
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub role: Role,
    pub mac: u32,
    pub ip: Option<Cidr>,
    pub gateway: Option<Ip>,
    pub ports: Vec<usize>,
    /// Routers: the address on each port.
    pub ifaces: Vec<Option<Cidr>>,
    pub routes: Vec<(Cidr, Ip)>,
    pub blocked: Vec<bool>,
    pub ttl: u32,
    /// A switch on a build level forwards only by the table the student filled
    /// and does not learn, so the table the student wrote is what decides.
    static_fwd: bool,
    mac_table: Vec<(u32, usize)>,
    outq: Vec<VecDeque<Frame>>,
    pending: Vec<(u32, usize, Frame)>,
    /// Servers: the connections that finished a handshake, and those waiting
    /// for the client's ACK: (client address, client port, server ISN).
    established: Vec<(Ip, u32)>,
    half_open: Vec<(Ip, u32, u32)>,
}

#[derive(Debug, Clone)]
struct Link {
    a: (usize, usize),
    b: (usize, usize),
    down: bool,
    lose_every: u32,
    delay: u32,
    sent: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Kind {
    Udp,
    Ping,
    Arp,
    Tcp,
    Bulk,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tcp {
    Idle,
    SynSent,
    Asked,
    Done,
    Failed,
}

#[derive(Debug, Clone)]
struct App {
    kind: Kind,
    from: usize,
    to: Ip,
    at: u32,
    count: u32,
    every: u32,
    label: String,
    sent: u32,
    first: Flags,
    third_ack: String,
    retry: u32,
    tries: u32,
    state: Tcp,
    timer: u32,
    x: u32,
    window: u32,
    rto: u32,
    base: u32,
    next: u32,
}

/// Frames that may wait at one port before the next is dropped.
const PORT_QUEUE: usize = 6;
const HOST_QUEUE: usize = 64;
/// A collided frame is tried again at most this many times.
const MAX_ATTEMPTS: u32 = 6;
pub const SERVER_PORT: u32 = 80;
const CLIENT_PORT: u32 = 40000;
/// The server's starting sequence number, shown in the packet inspector.
const SERVER_ISN: u32 = 7000;

pub struct Sim {
    pub nodes: Vec<Node>,
    links: Vec<Link>,
    apps: Vec<App>,
    rng: Rng,
    t: u32,
    stats: Stats,
    rec: Record,
}

fn role_of(kind: &str) -> Option<Role> {
    Some(match kind {
        "host" | "printer" | "server" => Role::Host,
        "hub" => Role::Hub,
        "switch" => Role::Switch,
        "router" => Role::Router,
        _ => return None,
    })
}

fn opt_cidr(s: &str, what: &str) -> Result<Option<Cidr>, String> {
    if s.is_empty() {
        return Ok(None);
    }
    parse_cidr(s)
        .map(Some)
        .ok_or_else(|| format!("{what}: bad address {s}"))
}

fn port_of(level: &Level, n: &Node, node: &str, link: &str) -> Result<usize, String> {
    let li = level
        .links
        .iter()
        .position(|l| l.id == link)
        .ok_or_else(|| format!("{node}: no cable {link}"))?;
    n.ports
        .iter()
        .position(|&x| x == li)
        .ok_or_else(|| format!("{node}: cable {link} is not plugged in here"))
}

impl Sim {
    /// Builds the network a level describes. An error names what is wrong.
    pub fn build(level: &Level) -> Result<Sim, String> {
        let mut nodes: Vec<Node> = Vec::new();
        for (i, d) in level.nodes.iter().enumerate() {
            let role = role_of(&d.kind).ok_or_else(|| format!("{}: unknown kind {}", d.id, d.kind))?;
            if nodes.iter().any(|o| o.id == d.id) {
                return Err(format!("two nodes are called {}", d.id));
            }
            let gateway = if d.gateway.is_empty() {
                None
            } else {
                Some(parse_ip(&d.gateway).ok_or_else(|| format!("{}: bad gateway {}", d.id, d.gateway))?)
            };
            nodes.push(Node {
                id: d.id.clone(),
                name: d.name.clone(),
                role,
                mac: i as u32 + 0x10,
                ip: opt_cidr(&d.ip, &d.id)?,
                gateway,
                ports: Vec::new(),
                ifaces: Vec::new(),
                routes: Vec::new(),
                blocked: Vec::new(),
                ttl: if d.ttl == 0 { 64 } else { d.ttl },
                static_fwd: d.static_fwd,
                mac_table: Vec::new(),
                outq: Vec::new(),
                pending: Vec::new(),
                established: Vec::new(),
                half_open: Vec::new(),
            });
            if role == Role::Host && nodes[i].ip.is_none() {
                return Err(format!("{} needs an address", d.id));
            }
        }
        let mut links = Vec::new();
        for l in &level.links {
            let find = |id: &str| nodes.iter().position(|n| n.id == id);
            let a = find(&l.a).ok_or_else(|| format!("cable {}: no node {}", l.id, l.a))?;
            let b = find(&l.b).ok_or_else(|| format!("cable {}: no node {}", l.id, l.b))?;
            if a == b {
                return Err(format!("cable {} joins {} to itself", l.id, l.a));
            }
            let pa = nodes[a].ports.len();
            nodes[a].ports.push(links.len());
            let pb = nodes[b].ports.len();
            nodes[b].ports.push(links.len());
            links.push(Link {
                a: (a, pa),
                b: (b, pb),
                down: l.down,
                lose_every: l.lose_every,
                delay: l.delay.max(1),
                sent: 0,
            });
        }
        for (i, d) in level.nodes.iter().enumerate() {
            let ports = nodes[i].ports.len();
            nodes[i].outq = vec![VecDeque::new(); ports];
            nodes[i].blocked = vec![false; ports];
            nodes[i].ifaces = vec![None; ports];
            if nodes[i].role == Role::Host && ports != 1 {
                return Err(format!("{} must have exactly one cable", d.id));
            }
            if !d.block.is_empty() {
                let p = port_of(level, &nodes[i], &d.id, &d.block)?;
                nodes[i].blocked[p] = true;
            }
            for f in &d.ifaces {
                let p = port_of(level, &nodes[i], &d.id, &f.link)?;
                nodes[i].ifaces[p] = opt_cidr(&f.ip, &d.id)?;
            }
            for r in &d.routes {
                let p = parse_cidr(&r.prefix).ok_or_else(|| format!("{}: bad route {}", d.id, r.prefix))?;
                let v = parse_ip(&r.via).ok_or_else(|| format!("{}: bad next hop {}", d.id, r.via))?;
                nodes[i].routes.push((p.network(), v));
            }
            // A build-level switch is pre-loaded with the student's table: each
            // hint names a machine and the neighbour the switch should send its
            // frames toward. Resolve that neighbour to a port and the machine to
            // its MAC, which is what the forwarding loop matches on.
            for h in &d.mac_hints {
                let dst = nodes
                    .iter()
                    .position(|o| o.id == h.dst)
                    .ok_or_else(|| format!("{}: table names no machine {}", d.id, h.dst))?;
                let via = nodes
                    .iter()
                    .position(|o| o.id == h.via)
                    .ok_or_else(|| format!("{}: table names no machine {}", d.id, h.via))?;
                let port = nodes[i].ports.iter().position(|&li| {
                    let l = &links[li];
                    let other = if l.a.0 == i { l.b.0 } else { l.a.0 };
                    other == via
                });
                let port = port.ok_or_else(|| format!("{}: no cable toward {}", d.id, h.via))?;
                let mac = nodes[dst].mac;
                nodes[i].mac_table.push((mac, port));
            }
        }
        let mut apps = Vec::new();
        for a in &level.apps {
            let kind = match a.kind.as_str() {
                "udp" => Kind::Udp,
                "ping" => Kind::Ping,
                "arp" => Kind::Arp,
                "tcp" => Kind::Tcp,
                "bulk" => Kind::Bulk,
                k => return Err(format!("{}: unknown program {k}", a.id)),
            };
            let from = nodes
                .iter()
                .position(|n| n.id == a.from)
                .ok_or_else(|| format!("{}: no node {}", a.id, a.from))?;
            if nodes[from].role != Role::Host {
                return Err(format!("{}: {} cannot run a program", a.id, a.from));
            }
            let to = parse_ip(&a.to).ok_or_else(|| format!("{}: bad address {}", a.id, a.to))?;
            let first = if a.first_flags.is_empty() {
                "SYN"
            } else {
                &a.first_flags
            };
            let first = Flags::parse(first).ok_or_else(|| format!("{}: bad flags {first}", a.id))?;
            let third_ack = if a.third_ack.is_empty() {
                "y+1"
            } else {
                &a.third_ack
            };
            if !["y+1", "y", "x+1"].contains(&third_ack) {
                return Err(format!("{}: bad ack {third_ack}", a.id));
            }
            if kind == Kind::Bulk && (a.window == 0 || a.rto == 0) {
                return Err(format!("{}: a transfer needs a window and a timeout", a.id));
            }
            apps.push(App {
                kind,
                from,
                to,
                at: a.at,
                count: a.count,
                every: a.every.max(1),
                label: a.label.clone(),
                sent: 0,
                first,
                third_ack: third_ack.to_string(),
                retry: a.retry.max(1),
                tries: a.tries.max(1),
                state: Tcp::Idle,
                timer: 0,
                x: 100 * (apps.len() as u32 + 1),
                window: a.window,
                rto: a.rto,
                base: 1,
                next: 1,
            });
        }
        let n = apps.len();
        Ok(Sim {
            nodes,
            links,
            apps,
            rng: Rng::new(level.seed as u64),
            t: 0,
            stats: Stats {
                delivered: vec![0; n],
                replies: vec![0; n],
                served: vec![false; n],
                transferred: vec![false; n],
                ..Stats::default()
            },
            rec: Record {
                ticks: 0,
                transits: Vec::new(),
                events: Vec::new(),
                tables: Vec::new(),
                stats_at: Vec::new(),
            },
        })
    }

    /// Runs the network for at most `ticks` ticks, stopping early once every
    /// program has finished and no frame is left anywhere.
    pub fn run(mut self, ticks: u32) -> Record {
        self.snapshot();
        for t in 1..=ticks {
            self.t = t;
            self.start_apps();
            self.arrivals();
            self.transmit();
            self.snapshot();
            if self.quiet() {
                break;
            }
        }
        self.rec.ticks = self.t;
        self.rec
    }

    fn quiet(&self) -> bool {
        let apps_done = self.apps.iter().all(|a| match a.kind {
            Kind::Tcp => matches!(a.state, Tcp::Done | Tcp::Failed),
            Kind::Bulk => a.base > a.count,
            _ => a.sent >= a.count,
        });
        let nothing_waits = self
            .nodes
            .iter()
            .all(|n| n.pending.is_empty() && n.outq.iter().all(|q| q.is_empty()));
        let nothing_moves = self.rec.transits.iter().all(|x| x.t1 <= self.t);
        apps_done && nothing_waits && nothing_moves
    }

    fn event(&mut self, node: usize, key: &'static str, args: Vec<String>, fx: &'static str) {
        match fx {
            "drop" => self.stats.drops += 1,
            "collision" => self.stats.collisions += 1,
            _ => {}
        }
        self.rec.events.push(Event {
            t: self.t,
            node,
            key,
            args,
            fx,
            transit: None,
        });
    }

    fn name(&self, n: usize) -> String {
        self.nodes[n].name.clone()
    }

    fn enqueue(&mut self, n: usize, port: usize, f: Frame) {
        let cap = if self.nodes[n].role == Role::Host {
            HOST_QUEUE
        } else {
            PORT_QUEUE
        };
        if self.nodes[n].outq[port].len() >= cap {
            let who = self.name(n);
            self.event(n, "ev_queue_full", vec![who], "drop");
            return;
        }
        self.nodes[n].outq[port].push_back(f);
    }

    /// The MAC address of whoever owns an IP address (the instant ARP).
    fn owner_mac(&self, ip: Ip) -> Option<u32> {
        self.nodes
            .iter()
            .find(|n| {
                n.ip.map(|c| c.ip == ip).unwrap_or(false)
                    || n.ifaces.iter().any(|f| f.map(|c| c.ip == ip).unwrap_or(false))
            })
            .map(|n| n.mac)
    }

    /// A host sends an IP packet: on its own block straight to the receiver's
    /// MAC address, otherwise to its gateway's.
    fn send_ip(&mut self, n: usize, p: Packet) {
        let me = self.nodes[n].ip.expect("hosts have addresses");
        let hop = if me.contains(p.dst) {
            p.dst
        } else {
            match self.nodes[n].gateway {
                Some(g) => g,
                None => {
                    let args = vec![self.name(n), show_ip(p.dst), me.network().show()];
                    self.event(n, "ev_no_gateway", args, "drop");
                    return;
                }
            }
        };
        let mac = if me.contains(hop) {
            self.owner_mac(hop)
        } else {
            None
        };
        let Some(mac) = mac else {
            let key = if hop == p.dst {
                "ev_arp_fail"
            } else {
                "ev_arp_fail_gw"
            };
            let args = vec![self.name(n), show_ip(hop), me.network().show()];
            self.event(n, key, args, "drop");
            return;
        };
        let f = Frame {
            src_mac: self.nodes[n].mac,
            dst_mac: mac,
            body: Body::Ip(p),
            attempt: 0,
        };
        self.enqueue(n, 0, f);
    }

    fn tcp(
        &self,
        n: usize,
        dst: Ip,
        app: usize,
        flags: Flags,
        ports: (u32, u32),
        seq_ack: (u32, u32),
    ) -> Packet {
        Packet {
            src: self.nodes[n].ip.expect("address").ip,
            dst,
            ttl: self.nodes[n].ttl,
            proto: Proto::Tcp {
                flags,
                sport: ports.0,
                dport: ports.1,
                seq: seq_ack.0,
                ack: seq_ack.1,
                data: 0,
            },
            app,
            label: String::new(),
        }
    }

    fn start_apps(&mut self) {
        let t = self.t;
        for i in 0..self.apps.len() {
            let a = self.apps[i].clone();
            if t < a.at {
                continue;
            }
            let from = a.from;
            let src = self.nodes[from].ip.expect("address").ip;
            let ttl = self.nodes[from].ttl;
            match a.kind {
                Kind::Udp | Kind::Ping => {
                    if a.sent < a.count && (t - a.at).is_multiple_of(a.every) {
                        self.apps[i].sent += 1;
                        let proto = if a.kind == Kind::Udp {
                            Proto::Udp
                        } else {
                            Proto::Ping {
                                reply: false,
                                n: a.sent + 1,
                            }
                        };
                        let p = Packet {
                            src,
                            dst: a.to,
                            ttl,
                            proto,
                            app: i,
                            label: a.label.clone(),
                        };
                        self.send_ip(from, p);
                    }
                }
                Kind::Arp => {
                    if a.sent < a.count && (t - a.at).is_multiple_of(a.every) {
                        self.apps[i].sent += 1;
                        let f = Frame {
                            src_mac: self.nodes[from].mac,
                            dst_mac: BROADCAST_MAC,
                            body: Body::Arp {
                                reply: false,
                                sender: src,
                                target: a.to,
                            },
                            attempt: 0,
                        };
                        self.enqueue(from, 0, f);
                    }
                }
                Kind::Tcp => {
                    let sport = CLIENT_PORT + i as u32;
                    match a.state {
                        Tcp::Idle => {
                            self.apps[i].state = Tcp::SynSent;
                            self.apps[i].timer = t;
                            self.apps[i].sent = 1;
                            let p = self.tcp(from, a.to, i, a.first, (sport, SERVER_PORT), (a.x, 0));
                            self.send_ip(from, p);
                        }
                        Tcp::SynSent | Tcp::Asked if t - a.timer >= a.retry => {
                            if a.sent >= a.tries {
                                self.apps[i].state = Tcp::Failed;
                                let who = self.name(from);
                                self.event(from, "ev_gave_up", vec![who], "drop");
                            } else {
                                self.apps[i].sent += 1;
                                self.apps[i].state = Tcp::SynSent;
                                self.apps[i].timer = t;
                                self.stats.retransmits += 1;
                                let who = self.name(from);
                                self.event(from, "ev_retry", vec![who], "info");
                                let p = self.tcp(from, a.to, i, a.first, (sport, SERVER_PORT), (a.x, 0));
                                self.send_ip(from, p);
                            }
                        }
                        _ => {}
                    }
                }
                Kind::Bulk => self.bulk_tick(i),
            }
        }
    }

    /// Go-back-N: keep up to `window` segments in flight; when the oldest
    /// has waited `rto` ticks without an ACK, send it and everything after
    /// it again.
    fn bulk_tick(&mut self, i: usize) {
        let t = self.t;
        let a = self.apps[i].clone();
        if a.base > a.count {
            return;
        }
        if a.next > a.base && t - a.timer >= a.rto {
            self.stats.retransmits += a.next - a.base;
            self.apps[i].next = a.base;
            let args = vec![self.name(a.from), a.base.to_string()];
            self.event(a.from, "ev_timeout", args, "info");
        }
        while self.apps[i].next < self.apps[i].base + a.window && self.apps[i].next <= a.count {
            let s = self.apps[i].next;
            if s == self.apps[i].base {
                self.apps[i].timer = t;
            }
            self.apps[i].next += 1;
            let flags = Flags {
                ack: true,
                ..Flags::default()
            };
            let mut p = self.tcp(
                a.from,
                a.to,
                i,
                flags,
                (CLIENT_PORT + i as u32, SERVER_PORT),
                (s, 0),
            );
            if let Proto::Tcp { data, .. } = &mut p.proto {
                *data = s;
            }
            p.label = a.label.clone();
            self.send_ip(a.from, p);
        }
    }

    /// Frames arriving this tick, in the order they were sent. A hub that
    /// receives more than one at once loses all of them; each sender tries
    /// again after a random wait that doubles with each collision.
    fn arrivals(&mut self) {
        let t = self.t;
        let arriving: Vec<usize> = (0..self.rec.transits.len())
            .filter(|&i| self.rec.transits[i].t1 == t)
            .collect();
        let mut at_hub: Vec<(usize, Vec<usize>)> = Vec::new();
        for &i in &arriving {
            let to = self.rec.transits[i].to;
            if self.nodes[to].role == Role::Hub && !self.rec.transits[i].lost {
                match at_hub.iter_mut().find(|e| e.0 == to) {
                    Some(e) => e.1.push(i),
                    None => at_hub.push((to, vec![i])),
                }
            }
        }
        let collided: Vec<usize> = at_hub
            .iter()
            .filter(|e| e.1.len() > 1)
            .flat_map(|e| e.1.clone())
            .collect();
        for (hub, list) in &at_hub {
            if list.len() > 1 {
                let args = vec![self.name(*hub), list.len().to_string()];
                self.event(*hub, "ev_collision", args, "collision");
            }
        }
        for &i in &arriving {
            let x = self.rec.transits[i].clone();
            if x.lost {
                continue;
            }
            if collided.contains(&i) {
                self.collided(x);
                continue;
            }
            let link = &self.links[x.link];
            let port = if link.a.0 == x.to { link.a.1 } else { link.b.1 };
            self.receive(x.to, port, x.frame);
        }
    }

    fn collided(&mut self, x: Transit) {
        let mut f = x.frame;
        f.attempt += 1;
        if f.attempt > MAX_ATTEMPTS {
            let who = self.name(x.from);
            self.event(x.from, "ev_collision_gave_up", vec![who], "drop");
            return;
        }
        let link = &self.links[x.link];
        let port = if link.a.0 == x.from { link.a.1 } else { link.b.1 };
        let slots = 1u64 << f.attempt.min(4);
        let wait = 1 + (self.rng.next() % slots) as u32;
        self.stats.retransmits += 1;
        self.nodes[x.from].pending.push((self.t + wait, port, f));
    }

    /// A cable carries nothing if it is down or either end has blocked its
    /// port. A spanning-tree block names a port on one switch, but the cable it
    /// closes is dead in both directions — so the other switch must not keep
    /// pushing frames onto it (they would ride a cable drawn as disconnected,
    /// then vanish). Those frames were silently discarded at the blocked end
    /// anyway, so dropping them here changes no score.
    fn link_closed(&self, li: usize) -> bool {
        let l = &self.links[li];
        l.down || self.nodes[l.a.0].blocked[l.a.1] || self.nodes[l.b.0].blocked[l.b.1]
    }

    /// The ports a switch or hub floods a frame out of.
    fn flood_ports(&self, n: usize, inport: usize) -> Vec<usize> {
        (0..self.nodes[n].ports.len())
            .filter(|&p| p != inport && !self.link_closed(self.nodes[n].ports[p]))
            .collect()
    }

    fn receive(&mut self, n: usize, port: usize, f: Frame) {
        match self.nodes[n].role {
            Role::Hub => {
                for p in self.flood_ports(n, port) {
                    self.enqueue(n, p, f.clone());
                }
            }
            Role::Switch => self.switch(n, port, f),
            Role::Router => self.router(n, port, f),
            Role::Host => self.host(n, f),
        }
    }

    /// A switch learns which port each sender is on, then sends a frame out
    /// of the one port its receiver is on, or every other port if it does
    /// not know yet (or the frame is a broadcast).
    fn switch(&mut self, n: usize, port: usize, f: Frame) {
        if self.nodes[n].blocked[port] {
            return;
        }
        // A build-level switch does not learn: the student's table decides.
        if !self.nodes[n].static_fwd {
            let known = self.nodes[n].mac_table.iter().position(|e| e.0 == f.src_mac);
            let args = vec![self.name(n), show_mac(f.src_mac), (port + 1).to_string()];
            match known {
                Some(i) if self.nodes[n].mac_table[i].1 == port => {}
                Some(i) => {
                    self.nodes[n].mac_table[i].1 = port;
                    self.event(n, "ev_moved", args, "learn");
                }
                None => {
                    self.nodes[n].mac_table.push((f.src_mac, port));
                    self.event(n, "ev_learn", args, "learn");
                }
            }
        }
        let out = self.nodes[n]
            .mac_table
            .iter()
            .find(|e| e.0 == f.dst_mac)
            .map(|e| e.1);
        match out {
            Some(p) if p == port => {}
            Some(p) => self.enqueue(n, p, f),
            // A learning switch floods an unknown unicast so it still arrives.
            // A build-level switch forwards only by the student's table, so an
            // address the table does not cover is dropped — the table has to be
            // right and complete. Broadcasts always flood.
            None if self.nodes[n].static_fwd && f.dst_mac != BROADCAST_MAC => {
                let args = vec![self.name(n), show_mac(f.dst_mac)];
                self.event(n, "ev_no_entry", args, "drop");
            }
            None => {
                for p in self.flood_ports(n, port) {
                    self.enqueue(n, p, f.clone());
                }
            }
        }
    }

    /// The router's choice: the matching entry with the longest prefix. A
    /// block the router is plugged into is a match with the receiver itself
    /// as the next hop.
    pub fn lookup(&self, n: usize, dst: Ip) -> Option<(usize, Ip, Cidr)> {
        let node = &self.nodes[n];
        let mut best: Option<(usize, Ip, Cidr)> = None;
        let mut consider = |port: usize, hop: Ip, c: Cidr| {
            if c.contains(dst) && best.map(|b| c.len > b.2.len).unwrap_or(true) {
                best = Some((port, hop, c));
            }
        };
        for (p, f) in node.ifaces.iter().enumerate() {
            if let Some(c) = f {
                consider(p, dst, c.network());
            }
        }
        for &(prefix, via) in &node.routes {
            let port = node
                .ifaces
                .iter()
                .position(|f| f.map(|c| c.contains(via)).unwrap_or(false));
            if let Some(port) = port {
                consider(port, via, prefix);
            }
        }
        best
    }

    fn router(&mut self, n: usize, _port: usize, f: Frame) {
        if f.dst_mac != self.nodes[n].mac {
            return;
        }
        let Body::Ip(mut p) = f.body else { return };
        let own = self.nodes[n]
            .ifaces
            .iter()
            .any(|c| c.map(|c| c.ip == p.dst).unwrap_or(false));
        if own {
            return;
        }
        if p.ttl <= 1 {
            let args = vec![self.name(n), show_ip(p.src), show_ip(p.dst)];
            self.event(n, "ev_ttl", args, "drop");
            return;
        }
        p.ttl -= 1;
        let Some((port, hop, c)) = self.lookup(n, p.dst) else {
            let args = vec![self.name(n), show_ip(p.dst)];
            self.event(n, "ev_no_route", args, "drop");
            return;
        };
        let Some(mac) = self.owner_mac(hop) else {
            let args = vec![self.name(n), show_ip(hop)];
            self.event(n, "ev_arp_fail_router", args, "drop");
            return;
        };
        let args = vec![self.name(n), show_ip(p.dst), c.show(), show_ip(hop)];
        self.event(n, "ev_route", args, "info");
        let out = Frame {
            src_mac: self.nodes[n].mac,
            dst_mac: mac,
            body: Body::Ip(p),
            attempt: 0,
        };
        self.enqueue(n, port, out);
    }

    fn host(&mut self, n: usize, f: Frame) {
        let me = self.nodes[n].ip.expect("address").ip;
        if f.dst_mac != self.nodes[n].mac && f.dst_mac != BROADCAST_MAC {
            let who = self.name(n);
            self.event(n, "ev_ignore", vec![who, show_mac(f.dst_mac)], "ignore");
            return;
        }
        match f.body {
            Body::Arp {
                reply: false,
                sender,
                target,
            } => {
                if target != me {
                    return;
                }
                let args = vec![self.name(n), show_ip(sender)];
                self.event(n, "ev_arp_answer", args, "deliver");
                let r = Frame {
                    src_mac: self.nodes[n].mac,
                    dst_mac: f.src_mac,
                    body: Body::Arp {
                        reply: true,
                        sender: me,
                        target: sender,
                    },
                    attempt: 0,
                };
                self.enqueue(n, 0, r);
            }
            Body::Arp {
                reply: true, sender, ..
            } => {
                for i in 0..self.apps.len() {
                    if self.apps[i].kind == Kind::Arp && self.apps[i].from == n && self.apps[i].to == sender {
                        self.stats.replies[i] += 1;
                        let args = vec![self.name(n), show_ip(sender), show_mac(f.src_mac)];
                        self.event(n, "ev_arp_learned", args, "deliver");
                    }
                }
            }
            Body::Ip(p) => {
                if p.dst != me {
                    let args = vec![self.name(n), show_ip(p.dst)];
                    self.event(n, "ev_not_mine", args, "drop");
                    return;
                }
                self.host_ip(n, p);
            }
        }
    }

    fn host_ip(&mut self, n: usize, p: Packet) {
        let me = p.dst;
        match p.proto.clone() {
            Proto::Udp => {
                self.stats.delivered[p.app] += 1;
                let args = vec![self.name(n), p.label.clone(), show_ip(p.src)];
                self.event(n, "ev_delivered", args, "deliver");
            }
            Proto::Ping { reply: false, n: k } => {
                self.stats.delivered[p.app] += 1;
                let args = vec![self.name(n), show_ip(p.src)];
                self.event(n, "ev_ping", args, "deliver");
                let r = Packet {
                    src: me,
                    dst: p.src,
                    ttl: self.nodes[n].ttl,
                    proto: Proto::Ping { reply: true, n: k },
                    app: p.app,
                    label: p.label,
                };
                self.send_ip(n, r);
            }
            Proto::Ping { reply: true, .. } => {
                self.stats.replies[p.app] += 1;
                let args = vec![self.name(n), show_ip(p.src)];
                self.event(n, "ev_pong", args, "deliver");
            }
            Proto::Tcp {
                flags,
                sport,
                dport,
                seq,
                ack,
                data,
            } => {
                if dport == SERVER_PORT {
                    self.server_tcp(n, &p, flags, sport, seq, ack, data);
                } else {
                    self.client_tcp(n, &p, flags, seq, ack, data);
                }
            }
        }
    }

    /// The server side of TCP: a SYN gets a SYN-ACK; an ACK that acknowledges
    /// the server's number finishes the handshake; a request on a finished
    /// connection gets its answer. Anything else gets a reset. A transfer's
    /// segments are taken in order and acknowledged cumulatively.
    #[allow(clippy::too_many_arguments)]
    fn server_tcp(&mut self, n: usize, p: &Packet, flags: Flags, sport: u32, seq: u32, ack: u32, data: u32) {
        let back = |s: &Sim, fl: Flags, sa: (u32, u32)| s.tcp(n, p.src, p.app, fl, (SERVER_PORT, sport), sa);
        if self.apps[p.app].kind == Kind::Bulk {
            let count = self.apps[p.app].count;
            let expected = self.stats.delivered[p.app] + 1;
            if data == expected {
                self.stats.delivered[p.app] += 1;
                if self.stats.delivered[p.app] == count {
                    self.stats.transferred[p.app] = true;
                    let args = vec![self.name(n), count.to_string()];
                    self.event(n, "ev_transfer_done", args, "deliver");
                }
            } else {
                let args = vec![self.name(n), data.to_string(), expected.to_string()];
                self.event(n, "ev_out_of_order", args, "ignore");
            }
            let next = self.stats.delivered[p.app] + 1;
            let r = back(
                self,
                Flags {
                    ack: true,
                    ..Flags::default()
                },
                (0, next),
            );
            self.send_ip(n, r);
            return;
        }
        let conn = (p.src, sport);
        if flags.syn && !flags.ack && !flags.rst && !flags.fin {
            self.nodes[n].half_open.retain(|h| (h.0, h.1) != conn);
            self.nodes[n].half_open.push((p.src, sport, SERVER_ISN));
            let r = back(
                self,
                Flags {
                    syn: true,
                    ack: true,
                    ..Flags::default()
                },
                (SERVER_ISN, seq + 1),
            );
            let args = vec![self.name(n), show_ip(p.src)];
            self.event(n, "ev_synack", args, "info");
            self.send_ip(n, r);
            return;
        }
        if flags.ack && !flags.syn && data == 0 {
            if let Some(i) = self.nodes[n].half_open.iter().position(|h| (h.0, h.1) == conn) {
                let y = self.nodes[n].half_open[i].2;
                self.nodes[n].half_open.remove(i);
                if ack == y + 1 {
                    self.nodes[n].established.push(conn);
                    let args = vec![self.name(n), show_ip(p.src)];
                    self.event(n, "ev_established", args, "deliver");
                    return;
                }
                let args = vec![self.name(n), (y + 1).to_string(), ack.to_string()];
                self.event(n, "ev_bad_ack", args, "drop");
                let r = back(
                    self,
                    Flags {
                        rst: true,
                        ..Flags::default()
                    },
                    (0, 0),
                );
                self.send_ip(n, r);
                return;
            }
        }
        if data > 0 && self.nodes[n].established.contains(&conn) {
            let args = vec![self.name(n), show_ip(p.src)];
            self.event(n, "ev_request", args, "deliver");
            let fl = Flags {
                ack: true,
                ..Flags::default()
            };
            let mut r = back(self, fl, (SERVER_ISN + 1, seq + 1));
            if let Proto::Tcp { data, .. } = &mut r.proto {
                *data = 1;
            }
            r.label = self.apps[p.app].label.clone();
            self.send_ip(n, r);
            return;
        }
        let args = vec![self.name(n), flags.show(), show_ip(p.src)];
        self.event(n, "ev_reset", args, "drop");
        let r = back(
            self,
            Flags {
                rst: true,
                ..Flags::default()
            },
            (0, 0),
        );
        self.send_ip(n, r);
    }

    fn client_tcp(&mut self, n: usize, p: &Packet, flags: Flags, seq: u32, ack: u32, data: u32) {
        let i = p.app;
        let a = self.apps[i].clone();
        if a.kind == Kind::Bulk {
            if ack > self.apps[i].base {
                self.apps[i].base = ack;
                self.apps[i].timer = self.t;
            }
            return;
        }
        let sport = CLIENT_PORT + i as u32;
        if flags.rst {
            self.apps[i].state = Tcp::Failed;
            let who = self.name(n);
            self.event(n, "ev_was_reset", vec![who], "drop");
            return;
        }
        if flags.syn && flags.ack && a.state == Tcp::SynSent {
            if ack != a.x + 1 {
                return;
            }
            let third = match a.third_ack.as_str() {
                "y+1" => seq + 1,
                "y" => seq,
                _ => a.x + 1,
            };
            let fl = Flags {
                ack: true,
                ..Flags::default()
            };
            let p3 = self.tcp(n, a.to, i, fl, (sport, SERVER_PORT), (a.x + 1, third));
            self.send_ip(n, p3);
            let mut req = self.tcp(n, a.to, i, fl, (sport, SERVER_PORT), (a.x + 1, third));
            if let Proto::Tcp { data, .. } = &mut req.proto {
                *data = 1;
            }
            req.label = a.label.clone();
            self.send_ip(n, req);
            self.apps[i].state = Tcp::Asked;
            self.apps[i].timer = self.t;
            return;
        }
        if data > 0 && a.state == Tcp::Asked {
            self.apps[i].state = Tcp::Done;
            self.stats.served[i] = true;
            let args = vec![self.name(n), a.label.clone()];
            self.event(n, "ev_served", args, "deliver");
        }
    }

    /// Each port sends the frame at the front of its queue. A cable that
    /// loses every n-th frame loses it halfway along.
    fn transmit(&mut self) {
        let t = self.t;
        for n in 0..self.nodes.len() {
            let due: Vec<(u32, usize, Frame)> = self.nodes[n]
                .pending
                .iter()
                .filter(|e| e.0 <= t)
                .cloned()
                .collect();
            self.nodes[n].pending.retain(|e| e.0 > t);
            for (_, port, f) in due {
                self.nodes[n].outq[port].push_front(f);
            }
            for port in 0..self.nodes[n].ports.len() {
                let li = self.nodes[n].ports[port];
                if self.link_closed(li) {
                    self.nodes[n].outq[port].clear();
                    continue;
                }
                let Some(f) = self.nodes[n].outq[port].pop_front() else {
                    continue;
                };
                let link = &mut self.links[li];
                let to = if link.a.0 == n { link.b.0 } else { link.a.0 };
                link.sent += 1;
                let lost = link.lose_every > 0 && link.sent.is_multiple_of(link.lose_every);
                let delay = link.delay;
                self.stats.frames += 1;
                self.rec.transits.push(Transit {
                    link: li,
                    from: n,
                    to,
                    t0: t,
                    t1: t + delay,
                    frame: f,
                    lost,
                });
                if lost {
                    let args = vec![self.name(n), self.name(to)];
                    self.event(n, "ev_lost", args, "drop");
                    self.rec.events.last_mut().expect("just pushed").transit =
                        Some(self.rec.transits.len() - 1);
                }
            }
        }
    }

    fn snapshot(&mut self) {
        let tables = (0..self.nodes.len()).map(|n| self.table(n)).collect();
        self.rec.tables.push(tables);
        self.rec.stats_at.push(self.stats.clone());
    }

    /// What a node knows right now, as facts and tables for the inspector.
    fn table(&self, n: usize) -> NodeInfo {
        let node = &self.nodes[n];
        let mut facts: Vec<(&'static str, FactVal)> = Vec::new();
        let mut tables: Vec<InfoTable> = Vec::new();
        let txt = FactVal::Text;
        match node.role {
            Role::Host => {
                if let Some(c) = node.ip {
                    facts.push(("tbl_ip", txt(c.show())));
                }
                facts.push(("tbl_mac", txt(show_mac(node.mac))));
                facts.push((
                    "tbl_gateway",
                    match node.gateway {
                        Some(g) => txt(show_ip(g)),
                        None => FactVal::Copy("tbl_none"),
                    },
                ));
                if !node.established.is_empty() {
                    facts.push(("tbl_open_conns", txt(node.established.len().to_string())));
                }
            }
            Role::Hub => {
                facts.push(("tbl_ports", txt(node.ports.len().to_string())));
            }
            Role::Switch => {
                facts.push(("tbl_mac", txt(show_mac(node.mac))));
                let rows: Vec<Vec<String>> = node
                    .mac_table
                    .iter()
                    .map(|(mac, port)| vec![show_mac(*mac), (port + 1).to_string()])
                    .collect();
                tables.push(InfoTable {
                    title: "tbl_mac_table",
                    cols: vec!["tbl_col_mac", "tbl_col_port"],
                    rows,
                    empty: Some("tbl_mac_table_empty"),
                });
                let blocked: Vec<Vec<String>> = node
                    .blocked
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| **b)
                    .map(|(p, _)| vec![(p + 1).to_string()])
                    .collect();
                if !blocked.is_empty() {
                    tables.push(InfoTable {
                        title: "tbl_blocked",
                        cols: vec!["tbl_col_port"],
                        rows: blocked,
                        empty: None,
                    });
                }
            }
            Role::Router => {
                facts.push(("tbl_mac", txt(show_mac(node.mac))));
                let ifaces: Vec<Vec<String>> = node
                    .ifaces
                    .iter()
                    .enumerate()
                    .filter_map(|(p, c)| {
                        c.map(|c| vec![(p + 1).to_string(), show_ip(c.ip), c.network().show()])
                    })
                    .collect();
                tables.push(InfoTable {
                    title: "tbl_ifaces",
                    cols: vec!["tbl_col_port", "tbl_col_ip", "tbl_col_block"],
                    rows: ifaces,
                    empty: None,
                });
                let routes: Vec<Vec<String>> = node
                    .routes
                    .iter()
                    .map(|(prefix, via)| vec![prefix.show(), show_ip(*via)])
                    .collect();
                tables.push(InfoTable {
                    title: "tbl_routes",
                    cols: vec!["tbl_col_dest", "tbl_col_via"],
                    rows: routes,
                    empty: Some("tbl_routes_empty"),
                });
            }
        }
        let waiting: usize = node.outq.iter().map(|q| q.len()).sum();
        if waiting > 0 {
            facts.push(("tbl_waiting", txt(waiting.to_string())));
        }
        NodeInfo { facts, tables }
    }
}
