//! Procedural overworld generation, terrain, locations, and pathfinding.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::rng::{Rng, fbm, tile_noise};

pub const WORLD_W: usize = 160;
pub const WORLD_H: usize = 80;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(self, other: Point) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn offset(self, dx: i32, dy: i32) -> Self {
        Self::new(self.x + dx, self.y + dy)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Terrain {
    Water,
    Swamp,
    Grass,
    Forest,
    Hills,
    Mountain,
}

impl Terrain {
    pub fn name(self) -> &'static str {
        match self {
            Terrain::Water => "open water",
            Terrain::Swamp => "a sucking marsh",
            Terrain::Grass => "open grassland",
            Terrain::Forest => "deep forest",
            Terrain::Hills => "windswept hills",
            Terrain::Mountain => "sheer mountains",
        }
    }

    pub fn is_passable(self) -> bool {
        !matches!(self, Terrain::Water | Terrain::Mountain)
    }

    pub fn move_cost(self) -> u32 {
        match self {
            Terrain::Water | Terrain::Mountain => u32::MAX,
            Terrain::Grass => 2,
            Terrain::Forest => 3,
            Terrain::Hills => 5,
            Terrain::Swamp => 7,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementKind {
    Hamlet,
    Village,
    Town,
    Hold,
}

impl SettlementKind {
    pub fn label(self) -> &'static str {
        match self {
            SettlementKind::Hamlet => "Hamlet",
            SettlementKind::Village => "Village",
            SettlementKind::Town => "Town",
            SettlementKind::Hold => "Hold",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PoiKind {
    Nest,
    Ruin,
    Cave,
    Shrine,
    Camp,
    Grove,
}

impl PoiKind {
    pub fn label(self) -> &'static str {
        match self {
            PoiKind::Nest => "Monster nest",
            PoiKind::Ruin => "Ruined place",
            PoiKind::Cave => "Cave",
            PoiKind::Shrine => "Old shrine",
            PoiKind::Camp => "Hostile camp",
            PoiKind::Grove => "Haunted grove",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settlement {
    pub id: usize,
    pub pos: Point,
    pub kind: SettlementKind,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Poi {
    pub id: usize,
    pub pos: Point,
    pub kind: PoiKind,
    pub name: String,
    pub detail: String,
    pub danger: u8,
    pub monster: String,
    pub explored: bool,
    pub cleared: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    pub seed: u64,
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Terrain>,
    pub roads: Vec<bool>,
    pub settlements: Vec<Settlement>,
    pub pois: Vec<Poi>,
}

impl World {
    pub fn generate(seed: u64) -> Self {
        let mut rng = Rng::new(seed ^ 0xA53F_9C1D_77E2_0B41);
        let width = WORLD_W;
        let height = WORLD_H;
        let mut tiles = vec![Terrain::Grass; width * height];

        // ------------------------------------------------------------------
        // Elevation & moisture
        // ------------------------------------------------------------------
        let mut elevation = vec![0.0f32; width * height];
        for y in 0..height {
            for x in 0..width {
                let nx = x as f32 / (width - 1) as f32 * 2.0 - 1.0;
                let ny = y as f32 / (height - 1) as f32 * 2.0 - 1.0;
                let distance = (nx * nx + ny * ny).sqrt();
                let base = fbm(x as f32 * 0.045, y as f32 * 0.045, seed ^ 0x1234, 5);
                let detail = fbm(x as f32 * 0.13, y as f32 * 0.13, seed ^ 0x9876, 3);
                elevation[y * width + x] =
                    base * 0.85 + detail * 0.15 - distance * distance * 0.55;
            }
        }

        let mut sorted = elevation.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let sea = quantile(&sorted, 0.28);
        let lowland = quantile(&sorted, 0.34);
        let hills = quantile(&sorted, 0.79);
        let mountains = quantile(&sorted, 0.91);

        for y in 0..height {
            for x in 0..width {
                let i = y * width + x;
                let e = elevation[i];
                let moisture = fbm(x as f32 * 0.075, y as f32 * 0.075, seed ^ 0x55AA, 4);
                tiles[i] = if e < sea {
                    Terrain::Water
                } else if e > mountains {
                    Terrain::Mountain
                } else if e > hills {
                    Terrain::Hills
                } else if e < lowland && moisture > 0.44 {
                    Terrain::Swamp
                } else if moisture > 0.57 {
                    Terrain::Forest
                } else {
                    Terrain::Grass
                };
            }
        }

        let mut land = largest_passable_component(&tiles, width, height);
        if land.is_empty() {
            tiles.fill(Terrain::Grass);
            land = vec![true; width * height];
        }
        let candidates: Vec<Point> = (0..width * height)
            .filter(|&i| land[i])
            .map(|i| Point::new((i % width) as i32, (i / width) as i32))
            .collect();

        // ------------------------------------------------------------------
        // Settlements
        // ------------------------------------------------------------------
        let mut used_settlement_names = HashSet::new();
        let mut settlements: Vec<Settlement> = Vec::new();
        let mut min_distance = 22.0f32;
        let mut attempts = 0usize;

        while settlements.len() < 7 && attempts < 12_000 {
            attempts += 1;
            if attempts.is_multiple_of(3_000) {
                min_distance = (min_distance * 0.82).max(10.0);
            }
            let p = candidates[(rng.next_u64() % candidates.len() as u64) as usize];
            if settlements
                .iter()
                .any(|s| s.pos.distance_to(p) < min_distance)
            {
                continue;
            }
            let kind = match rng.range(0, 100) {
                0..=34 => SettlementKind::Hamlet,
                35..=74 => SettlementKind::Village,
                75..=94 => SettlementKind::Town,
                _ => SettlementKind::Hold,
            };
            let name = unique_settlement_name(&mut rng, &mut used_settlement_names);
            settlements.push(Settlement {
                id: settlements.len(),
                pos: p,
                kind,
                name,
            });
        }

        if settlements.is_empty() {
            let p = candidates[0];
            settlements.push(Settlement {
                id: 0,
                pos: p,
                kind: SettlementKind::Village,
                name: "Wayfarer's Rest".to_string(),
            });
        }

        // ------------------------------------------------------------------
        // Points of interest
        // ------------------------------------------------------------------
        let mut used_poi_names = HashSet::new();
        let mut pois: Vec<Poi> = Vec::new();
        let kinds = [
            PoiKind::Ruin,
            PoiKind::Ruin,
            PoiKind::Nest,
            PoiKind::Nest,
            PoiKind::Cave,
            PoiKind::Cave,
            PoiKind::Shrine,
            PoiKind::Camp,
            PoiKind::Grove,
        ];
        let mut poi_attempts = 0usize;

        while pois.len() < 16 && poi_attempts < 20_000 {
            poi_attempts += 1;
            let p = candidates[(rng.next_u64() % candidates.len() as u64) as usize];
            if settlements
                .iter()
                .any(|s| s.pos.distance_to(p) < 7.0)
                || pois.iter().any(|o| o.pos.distance_to(p) < 7.0)
            {
                continue;
            }
            let kind = *rng.choose(&kinds);
            if poi_attempts < 12_000
                && !prefers_terrain(kind, tiles[p.y as usize * width + p.x as usize])
            {
                continue;
            }

            let nearest = settlements
                .iter()
                .map(|s| s.pos.distance_to(p))
                .fold(f32::INFINITY, f32::min);
            let danger =
                (1 + (nearest / 22.0) as u8 + rng.range(0, 2) as u8).clamp(1, 5);
            let monster = monster_for(kind, &mut rng).to_string();

            pois.push(Poi {
                id: pois.len(),
                pos: p,
                kind,
                name: poi_name(kind, &mut rng, &mut used_poi_names),
                detail: rng.choose(POI_DETAILS).to_string(),
                danger,
                monster,
                explored: false,
                cleared: false,
            });
        }

        // ------------------------------------------------------------------
        // Roads: a minimum spanning tree of settlements, plus spurs to POIs.
        // ------------------------------------------------------------------
        let mut roads = vec![false; width * height];
        if !settlements.is_empty() {
            let mut in_tree = vec![false; settlements.len()];
            in_tree[0] = true;
            for _ in 1..settlements.len() {
                let mut best: Option<(usize, usize, f32)> = None;
                for i in 0..settlements.len() {
                    if !in_tree[i] {
                        continue;
                    }
                    for j in 0..settlements.len() {
                        if in_tree[j] {
                            continue;
                        }
                        let d = settlements[i].pos.distance_to(settlements[j].pos);
                        if best.is_none_or(|(_, _, bd)| d < bd) {
                            best = Some((i, j, d));
                        }
                    }
                }
                let Some((i, j, _)) = best else { break };
                carve_road(
                    &tiles,
                    &mut roads,
                    width,
                    height,
                    settlements[i].pos,
                    settlements[j].pos,
                );
                in_tree[j] = true;
            }

            for poi in &pois {
                let nearest = settlements
                    .iter()
                    .min_by(|a, b| {
                        a.pos
                            .distance_to(poi.pos)
                            .partial_cmp(&b.pos.distance_to(poi.pos))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|s| s.pos);
                if let Some(settlement) = nearest {
                    carve_road(&tiles, &mut roads, width, height, settlement, poi.pos);
                }
            }
        }

        let mut name_rng = Rng::new(seed ^ 0x7F4A_7C15_9E37_79B9);
        let name = format!(
            "The {} {}",
            name_rng.choose(REGION_ADJECTIVES),
            name_rng.choose(REGION_NOUNS)
        );

        Self {
            seed,
            name,
            width,
            height,
            tiles,
            roads,
            settlements,
            pois,
        }
    }

    pub fn start_position(&self) -> Point {
        self.settlements[0].pos
    }

    pub fn in_bounds(&self, p: Point) -> bool {
        p.x >= 0 && p.y >= 0 && (p.x as usize) < self.width && (p.y as usize) < self.height
    }

    pub fn index(&self, p: Point) -> usize {
        p.y as usize * self.width + p.x as usize
    }

    pub fn terrain(&self, p: Point) -> Terrain {
        self.tiles[self.index(p)]
    }

    pub fn is_road(&self, p: Point) -> bool {
        self.roads[self.index(p)]
    }

    pub fn is_passable(&self, p: Point) -> bool {
        self.in_bounds(p) && self.terrain(p).is_passable()
    }

    pub fn move_cost(&self, p: Point) -> u32 {
        if self.is_road(p) {
            1
        } else {
            self.terrain(p).move_cost()
        }
    }

    pub fn poi_at(&self, p: Point) -> Option<&Poi> {
        self.pois.iter().find(|poi| poi.pos == p)
    }

    pub fn poi_at_mut(&mut self, p: Point) -> Option<&mut Poi> {
        self.pois.iter_mut().find(|poi| poi.pos == p)
    }

    pub fn settlement_at(&self, p: Point) -> Option<&Settlement> {
        self.settlements.iter().find(|s| s.pos == p)
    }

    pub fn nearest_settlement(&self, p: Point) -> Option<&Settlement> {
        self.settlements.iter().min_by(|a, b| {
            a.pos
                .distance_to(p)
                .partial_cmp(&b.pos.distance_to(p))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn find_path(&self, start: Point, goal: Point) -> Option<Vec<Point>> {
        astar(&self.tiles, &self.roads, self.width, self.height, start, goal)
    }

    /// A cheap cosmetic variation for rendering terrain texture.
    pub fn texture(&self, p: Point) -> f32 {
        tile_noise(p.x, p.y, self.seed)
    }
}

fn quantile(sorted: &[f32], q: f32) -> f32 {
    let i = ((sorted.len() as f32 - 1.0) * q).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}

fn largest_passable_component(tiles: &[Terrain], width: usize, height: usize) -> Vec<bool> {
    let mut visited = vec![false; tiles.len()];
    let mut best: Vec<usize> = Vec::new();

    for start in 0..tiles.len() {
        if visited[start] || !tiles[start].is_passable() {
            continue;
        }
        let mut component = Vec::new();
        let mut queue = VecDeque::from([start]);
        visited[start] = true;
        while let Some(i) = queue.pop_front() {
            component.push(i);
            let x = (i % width) as i32;
            let y = (i / width) as i32;
            for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
                let nx = x + dx;
                let ny = y + dy;
                if nx < 0 || ny < 0 || nx as usize >= width || ny as usize >= height {
                    continue;
                }
                let ni = ny as usize * width + nx as usize;
                if !visited[ni] && tiles[ni].is_passable() {
                    visited[ni] = true;
                    queue.push_back(ni);
                }
            }
        }
        if component.len() > best.len() {
            best = component;
        }
    }

    let mut mask = vec![false; tiles.len()];
    for i in best {
        mask[i] = true;
    }
    mask
}

fn prefers_terrain(kind: PoiKind, terrain: Terrain) -> bool {
    match kind {
        PoiKind::Nest => matches!(terrain, Terrain::Swamp | Terrain::Forest | Terrain::Grass),
        PoiKind::Ruin => matches!(terrain, Terrain::Grass | Terrain::Hills | Terrain::Forest),
        PoiKind::Cave => terrain == Terrain::Hills,
        PoiKind::Shrine => matches!(terrain, Terrain::Grass | Terrain::Forest),
        PoiKind::Camp => matches!(terrain, Terrain::Grass | Terrain::Forest),
        PoiKind::Grove => terrain == Terrain::Forest,
    }
}

fn monster_for(kind: PoiKind, rng: &mut Rng) -> &'static str {
    let list: &[&str] = match kind {
        PoiKind::Nest => &[
            "a pack of drowners",
            "a nest of nekkers",
            "a wyvern",
            "a swarm of bloodflies",
            "a bog hag",
        ],
        PoiKind::Ruin => &[
            "a wraith",
            "a grave hag",
            "a cursed knight",
            "a pair of ghouls",
            "a spectral rider",
        ],
        PoiKind::Cave => &[
            "a rock troll",
            "a fiend",
            "a shaelmaar",
            "a colony of kobolds",
            "a cave troll",
        ],
        PoiKind::Shrine => &[
            "a restless specter",
            "a possessed idol",
            "a weeping wraith",
            "a bog spirit",
        ],
        PoiKind::Camp => &[
            "a bandit crew",
            "a pack of deserters",
            "a cult cell",
            "a gang of poachers",
        ],
        PoiKind::Grove => &[
            "a leshen",
            "a pack of wolves",
            "a werewolf",
            "an ancient forest spirit",
        ],
    };
    rng.choose(list)
}

fn unique_settlement_name(rng: &mut Rng, used: &mut HashSet<String>) -> String {
    for _ in 0..80 {
        let candidate = format!("{}{}", rng.choose(NAME_PREFIXES), rng.choose(NAME_SUFFIXES));
        if used.insert(candidate.clone()) {
            return candidate;
        }
    }
    let mut n = 2;
    loop {
        let candidate = format!("Far{} {n}", rng.choose(NAME_SUFFIXES));
        if used.insert(candidate.clone()) {
            return candidate;
        }
        n += 1;
    }
}

fn poi_name(kind: PoiKind, rng: &mut Rng, used: &mut HashSet<String>) -> String {
    for _ in 0..100 {
        let candidate = match kind {
            PoiKind::Nest => format!(
                "The {} {}",
                rng.choose(ADJECTIVES),
                rng.choose(&["Warrens", "Nest", "Lair", "Hollows", "Barrow"])
            ),
            PoiKind::Ruin => format!(
                "The Ruins of {}{}",
                rng.choose(RUIN_PREFIXES),
                rng.choose(NAME_SUFFIXES)
            ),
            PoiKind::Cave => format!(
                "The {} {}",
                rng.choose(ADJECTIVES),
                rng.choose(&["Deep", "Hollow", "Chasm", "Cave"])
            ),
            PoiKind::Shrine => {
                format!("Shrine of the {}", rng.choose(EPITHETS))
            }
            PoiKind::Camp => format!("{} Camp", rng.choose(FACTIONS)),
            PoiKind::Grove => format!("The {} Grove", rng.choose(ADJECTIVES)),
        };
        if used.insert(candidate.clone()) {
            return candidate;
        }
    }
    let mut n = 2;
    loop {
        let candidate = format!("The Lost {} {n}", rng.choose(NAME_SUFFIXES));
        if used.insert(candidate.clone()) {
            return candidate;
        }
        n += 1;
    }
}

fn carve_road(
    tiles: &[Terrain],
    roads: &mut [bool],
    width: usize,
    height: usize,
    start: Point,
    goal: Point,
) {
    if let Some(path) = astar(tiles, roads, width, height, start, goal) {
        for p in path {
            roads[p.y as usize * width + p.x as usize] = true;
        }
    }
}

fn astar(
    tiles: &[Terrain],
    roads: &[bool],
    width: usize,
    height: usize,
    start: Point,
    goal: Point,
) -> Option<Vec<Point>> {
    if start == goal {
        return Some(vec![start]);
    }
    if start.x < 0
        || start.y < 0
        || goal.x < 0
        || goal.y < 0
        || start.x as usize >= width
        || start.y as usize >= height
        || goal.x as usize >= width
        || goal.y as usize >= height
    {
        return None;
    }

    let n = width * height;
    let start_i = start.y as usize * width + start.x as usize;
    let goal_i = goal.y as usize * width + goal.x as usize;
    let goal_pos = goal;
    if !tiles[goal_i].is_passable() {
        return None;
    }

    let mut g_score = vec![u32::MAX; n];
    let mut came_from = vec![usize::MAX; n];
    let mut heap: BinaryHeap<Reverse<(u32, u32, usize)>> = BinaryHeap::new();

    g_score[start_i] = 0;
    heap.push(Reverse((heuristic(start, goal_pos), 0, start_i)));

    while let Some(Reverse((_, cost, current))) = heap.pop() {
        if current == goal_i {
            let mut path = vec![goal];
            let mut at = current;
            while at != start_i {
                let prev = came_from[at];
                if prev == usize::MAX {
                    return None;
                }
                at = prev;
                path.push(Point::new((at % width) as i32, (at / width) as i32));
            }
            path.reverse();
            return Some(path);
        }
        if cost > g_score[current] {
            continue;
        }

        let x = (current % width) as i32;
        let y = (current / width) as i32;
        for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
            let nx = x + dx;
            let ny = y + dy;
            if nx < 0 || ny < 0 || nx as usize >= width || ny as usize >= height {
                continue;
            }
            let ni = ny as usize * width + nx as usize;
            if !tiles[ni].is_passable() {
                continue;
            }
            let step = if roads[ni] {
                1
            } else {
                tiles[ni].move_cost()
            };
            let tentative = cost.saturating_add(step);
            if tentative < g_score[ni] {
                g_score[ni] = tentative;
                came_from[ni] = current;
                let estimate = tentative.saturating_add(heuristic(Point::new(nx, ny), goal_pos));
                heap.push(Reverse((estimate, tentative, ni)));
            }
        }
    }

    None
}

fn heuristic(a: Point, b: Point) -> u32 {
    (a.x - b.x).unsigned_abs() + (a.y - b.y).unsigned_abs()
}

const REGION_ADJECTIVES: &[&str] = &[
    "Sundered",
    "Weeping",
    "Ashen",
    "Forgotten",
    "Hollow",
    "Gilded",
    "Broken",
    "Drowned",
    "Whispering",
    "Cinder-Scarred",
];

const REGION_NOUNS: &[&str] = &[
    "Marches",
    "Reach",
    "Coast",
    "Expanse",
    "Wilds",
    "Frontier",
    "Lowlands",
    "Moors",
    "Downs",
];

const NAME_PREFIXES: &[&str] = &[
    "Ash", "Bramble", "Crow", "Dun", "Ember", "Fen", "Gloam", "Harrow", "Iron",
    "Kestrel", "Mire", "Mourn", "Nettle", "Oak", "Pale", "Raven", "Salt", "Thorn",
    "Umber", "Vale", "Wick", "Yew", "Grey", "Black", "Cold", "Elm", "Stone", "Briar",
];

const NAME_SUFFIXES: &[&str] = &[
    "ford", "wick", "mere", "hollow", "stead", "reach", "gate", "barrow", "fell",
    "moor", "crag", "rest", "haven", "cross", "marsh", "down", "brook", "field",
];

const RUIN_PREFIXES: &[&str] = &[
    "Dun", "Grim", "Har", "Mor", "Ravens", "Thorn", "Wyr", "Eld", "Bael", "Or",
];

const ADJECTIVES: &[&str] = &[
    "Drowned",
    "Hollow",
    "Bleak",
    "Whispering",
    "Gilded",
    "Sundered",
    "Moss-Eaten",
    "Starving",
    "Pale",
    "Thorn-Wrapped",
    "Howling",
    "Forgotten",
    "Sunken",
    "Cinder",
    "Glimmering",
    "Rotting",
];

const EPITHETS: &[&str] = &[
    "Pale Flame",
    "Weeping Mother",
    "Drowned King",
    "Nameless Saint",
    "Hollow Star",
    "Broken Wheel",
    "Quiet Hunter",
    "Last Lantern",
];

const FACTIONS: &[&str] = &[
    "Bandit", "Deserter", "Cultist", "Mercenary", "Poacher", "Smuggler",
];

const POI_DETAILS: &[&str] = &[
    "Cold air spills from its mouth.",
    "The birds avoid this place.",
    "Old wards are scratched into the stone.",
    "Something has been dragging carcasses here.",
    "A faint humming rises after dusk.",
    "Tracks circle the site but never cross it.",
    "The ground is soft and recently turned.",
    "Every tree leans away from the centre.",
    "A rotten rope hangs from the highest branch.",
    "The locals will not say its name aloud.",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worlds_are_deterministic() {
        let a = World::generate(1234);
        let b = World::generate(1234);
        assert_eq!(a.tiles, b.tiles);
        assert_eq!(a.roads, b.roads);
        assert_eq!(a.pois.len(), b.pois.len());
        assert_eq!(a.settlements.len(), b.settlements.len());
        assert_eq!(a.name, b.name);
    }

    #[test]
    fn settlements_are_connected_by_roads() {
        for seed in [1, 2, 3, 42, 9001] {
            let world = World::generate(seed);
            assert!(!world.settlements.is_empty());
            let start = world.settlements[0].pos;
            for settlement in &world.settlements {
                assert!(
                    world.find_path(start, settlement.pos).is_some(),
                    "seed {seed}: no path to {}",
                    settlement.name
                );
            }
            for poi in &world.pois {
                assert!(
                    world.find_path(start, poi.pos).is_some(),
                    "seed {seed}: no path to {}",
                    poi.name
                );
            }
        }
    }

    #[test]
    fn points_are_on_passable_tiles() {
        let world = World::generate(77);
        for poi in &world.pois {
            assert!(world.is_passable(poi.pos));
        }
        for settlement in &world.settlements {
            assert!(world.is_passable(settlement.pos));
        }
    }
}
