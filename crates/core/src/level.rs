//! The level file (`data/levels.json`): chapters, and for each level its
//! network, its traffic, the edits a student may make, its goals and its
//! lesson. This module reads the file and applies an edit to a level; the
//! simulation (`sim.rs`) builds a network from the edited level.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LevelFile {
    pub chapters: Vec<Chapter>,
    pub levels: Vec<Level>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    pub layer: String,
    pub blurb: String,
    pub backdrop: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Level {
    pub id: String,
    pub chapter: String,
    pub title: String,
    pub story: String,
    pub concept: Vec<String>,
    pub task: String,
    pub ticks: u32,
    #[serde(default = "one")]
    pub seed: u32,
    pub nodes: Vec<NodeDef>,
    pub links: Vec<LinkDef>,
    pub apps: Vec<AppDef>,
    pub edits: Vec<EditDef>,
    pub goal: Vec<Cond>,
    pub bonus: Cond,
    pub quiz: Quiz,
    pub debrief: String,
    /// A chapter's demonstration scene (Phase 2): a watch-only lesson on
    /// TEST-NET addresses, shown before the chapter's levels. Not a scored
    /// level — kept out of the level cards, the progression and the star total.
    #[serde(default)]
    pub scene: bool,
    /// Evidence-first (Phase 2). Node and cable ids worth inspecting before a
    /// run; the student must open at least one before committing. Empty on a
    /// level that has not been migrated — such a level keeps the old flow.
    #[serde(default)]
    pub evidence: Vec<String>,
    /// The prediction the student commits before running. Graded against the
    /// real run (not a stored answer index). `None` keeps the old quiz star.
    #[serde(default)]
    pub predict: Option<PredictDef>,
    /// A few fixed practice cases offered after a wrong committed attempt. Each
    /// re-applies the base level's edits over its own value overrides, so the
    /// numbers change but the lesson does not. Never re-awards the star.
    #[serde(default)]
    pub variants: Vec<Variant>,
}

/// The prediction step: a prompt and the outcomes the student may claim. Every
/// option carries a claim the core can check against the committed run.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PredictDef {
    pub prompt: String,
    pub options: Vec<PredictOption>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PredictOption {
    pub label: String,
    pub claim: Claim,
}

/// A verifiable statement about a finished run. The grader (`game::graded`)
/// checks it against the recorded events, so no correct option is stored.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Claim {
    /// The program's traffic reaches its destination.
    Delivered { app: String },
    /// The program's traffic does not reach its destination.
    NotDelivered { app: String },
    /// A TCP connection opens and the server answers the request.
    Served { app: String },
    /// A request gets a reply back (ping, ARP).
    Replies { app: String },
    /// A whole file (go-back-N bulk transfer) arrives complete.
    Transferred { app: String },
    /// This specific device is the one that receives the delivery.
    DeliveredTo { node: String },
    /// A frame is dropped or collides at this device.
    FailsAt { node: String },
    /// A packet runs out of hops (TTL) at this device.
    ExpiresAt { node: String },
    /// This device's outgoing line fills and sheds frames.
    QueueFull { node: String },
    /// Nothing is lost: no drops and no collisions.
    NoLoss,
}

/// One fixed practice case: new numbers over the base level. `overrides` set
/// device/program/cable fields; `edits` replace an edit's option list so the
/// labels and values match the variant's addresses. The topology, the edit
/// targets and the prediction claims (which name device ids, not numbers) stay,
/// so the lesson is the same and only the numbers change.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    #[serde(default = "one")]
    pub seed: u32,
    #[serde(default)]
    pub overrides: Vec<Override>,
    #[serde(default)]
    pub edits: Vec<OptionOverride>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Override {
    pub target: Target,
    pub value: String,
}

/// Replaces the option list of one edit (by its id) in a practice variant.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OptionOverride {
    pub edit: String,
    pub options: Vec<OptionDef>,
}

fn one() -> u32 {
    1
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeDef {
    pub id: String,
    /// host, printer, server, hub, switch, router
    pub kind: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
    /// Hosts and servers: `10.0.1.10/24`.
    #[serde(default)]
    pub ip: String,
    /// Hosts and servers: the router address they hand off-block packets to.
    #[serde(default)]
    pub gateway: String,
    /// Routers: one address per cable, by the cable's id.
    #[serde(default)]
    pub ifaces: Vec<IfaceDef>,
    #[serde(default)]
    pub routes: Vec<RouteDef>,
    /// Switches: a cable this switch keeps blocked (spanning tree).
    #[serde(default)]
    pub block: String,
    /// Hosts: the TTL its packets start with.
    #[serde(default)]
    pub ttl: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IfaceDef {
    pub link: String,
    pub ip: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteDef {
    pub prefix: String,
    pub via: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkDef {
    pub id: String,
    pub a: String,
    pub b: String,
    #[serde(default)]
    pub down: bool,
    /// Every n-th frame on this cable is lost (0: none).
    #[serde(default)]
    pub lose_every: u32,
    #[serde(default = "one")]
    pub delay: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppDef {
    pub id: String,
    /// udp, ping, arp, tcp, bulk
    pub kind: String,
    pub from: String,
    pub to: String,
    #[serde(default = "one")]
    pub at: u32,
    #[serde(default = "one")]
    pub count: u32,
    #[serde(default = "one")]
    pub every: u32,
    #[serde(default)]
    pub label: String,
    /// tcp: the flags of the client's first segment, and the ack number of its
    /// third (`y+1`, `y` or `x+1`).
    #[serde(default)]
    pub first_flags: String,
    #[serde(default)]
    pub third_ack: String,
    /// tcp: ticks to wait before trying again, and how many tries.
    #[serde(default)]
    pub retry: u32,
    #[serde(default)]
    pub tries: u32,
    /// bulk: segments in flight, ticks before resending, segments to send.
    #[serde(default)]
    pub window: u32,
    #[serde(default)]
    pub rto: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EditDef {
    pub id: String,
    pub prompt: String,
    pub target: Target,
    pub options: Vec<OptionDef>,
    pub start: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    #[serde(default)]
    pub node: String,
    #[serde(default)]
    pub app: String,
    #[serde(default)]
    pub link: String,
    /// For a route: the prefix whose next hop is set.
    #[serde(default)]
    pub route: String,
    pub field: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OptionDef {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Cond {
    /// delivered, replies, served, transfer, max_frames, no_drops,
    /// no_collisions, max_ticks, max_retransmits
    pub kind: String,
    #[serde(default)]
    pub app: String,
    #[serde(default)]
    pub n: u32,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Quiz {
    pub question: String,
    pub options: Vec<String>,
    pub answer: usize,
    pub why: String,
}

pub fn parse(json: &str) -> Result<LevelFile, String> {
    serde_json::from_str(json).map_err(|e| format!("levels.json: {e}"))
}

fn flag(v: &str) -> Result<bool, String> {
    match v {
        "on" => Ok(true),
        "off" => Ok(false),
        _ => Err(format!("expected on or off, got {v}")),
    }
}

fn number(v: &str) -> Result<u32, String> {
    v.parse().map_err(|_| format!("expected a number, got {v}"))
}

/// Writes one chosen value into the level, where the edit's target says.
pub fn apply(level: &mut Level, t: &Target, v: &str) -> Result<(), String> {
    if !t.app.is_empty() {
        let app = level
            .apps
            .iter_mut()
            .find(|a| a.id == t.app)
            .ok_or_else(|| format!("no app {}", t.app))?;
        match t.field.as_str() {
            "to" => app.to = v.to_string(),
            "first_flags" => app.first_flags = v.to_string(),
            "third_ack" => app.third_ack = v.to_string(),
            "window" => app.window = number(v)?,
            "rto" => app.rto = number(v)?,
            f => return Err(format!("apps have no field {f}")),
        }
        return Ok(());
    }
    if !t.link.is_empty() {
        let link = level
            .links
            .iter_mut()
            .find(|l| l.id == t.link)
            .ok_or_else(|| format!("no cable {}", t.link))?;
        match t.field.as_str() {
            "down" => link.down = flag(v)?,
            f => return Err(format!("cables have no field {f}")),
        }
        return Ok(());
    }
    let node = level
        .nodes
        .iter_mut()
        .find(|n| n.id == t.node)
        .ok_or_else(|| format!("no node {}", t.node))?;
    match t.field.as_str() {
        "kind" => node.kind = v.to_string(),
        "ip" => node.ip = v.to_string(),
        "gateway" => node.gateway = v.to_string(),
        "block" => node.block = v.to_string(),
        "ttl" => node.ttl = number(v)?,
        "route" => {
            node.routes.retain(|r| r.prefix != t.route);
            if !v.is_empty() {
                node.routes.push(RouteDef {
                    prefix: t.route.clone(),
                    via: v.to_string(),
                });
            }
        }
        f => return Err(format!("nodes have no field {f}")),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny() -> Level {
        let json = r#"{"chapters":[],"levels":[{"id":"l","chapter":"c","title":"T","story":"s","concept":[],
          "task":"t","ticks":10,
          "nodes":[{"id":"a","kind":"host","name":"A","x":0,"y":0,"ip":"10.0.0.1/24"},
                   {"id":"r","kind":"router","name":"R","x":0,"y":0,"routes":[{"prefix":"10.1.0.0/16","via":"10.0.0.9"}]}],
          "links":[{"id":"l1","a":"a","b":"r"}],
          "apps":[{"id":"p","kind":"udp","from":"a","to":"10.0.0.2"}],
          "edits":[],"goal":[],"bonus":{"kind":"no_drops","text":"x"},
          "quiz":{"question":"q","options":["a"],"answer":0,"why":"w"},"debrief":"d"}]}"#;
        parse(json).unwrap().levels.remove(0)
    }

    fn target(node: &str, app: &str, link: &str, route: &str, field: &str) -> Target {
        Target {
            node: node.into(),
            app: app.into(),
            link: link.into(),
            route: route.into(),
            field: field.into(),
        }
    }

    #[test]
    fn an_edit_writes_where_its_target_says() {
        let mut l = tiny();
        apply(&mut l, &target("", "p", "", "", "to"), "10.0.0.7").unwrap();
        assert_eq!(l.apps[0].to, "10.0.0.7");
        apply(&mut l, &target("", "", "l1", "", "down"), "on").unwrap();
        assert!(l.links[0].down);
        apply(&mut l, &target("a", "", "", "", "gateway"), "10.0.0.254").unwrap();
        assert_eq!(l.nodes[0].gateway, "10.0.0.254");
    }

    #[test]
    fn a_route_edit_replaces_or_removes_the_route() {
        let mut l = tiny();
        let t = target("r", "", "", "10.1.0.0/16", "route");
        apply(&mut l, &t, "10.0.0.5").unwrap();
        assert_eq!(l.nodes[1].routes.len(), 1);
        assert_eq!(l.nodes[1].routes[0].via, "10.0.0.5");
        apply(&mut l, &t, "").unwrap();
        assert!(l.nodes[1].routes.is_empty());
    }

    #[test]
    fn a_bad_edit_is_refused() {
        let mut l = tiny();
        assert!(apply(&mut l, &target("a", "", "", "", "colour"), "red").is_err());
        assert!(apply(&mut l, &target("zz", "", "", "", "kind"), "hub").is_err());
        assert!(apply(&mut l, &target("", "", "l1", "", "down"), "maybe").is_err());
        assert!(apply(&mut l, &target("a", "", "", "", "ttl"), "lots").is_err());
    }
}
