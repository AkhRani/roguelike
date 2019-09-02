extern crate tcod;

use tcod::colors;
use tcod::colors::Color;
use tcod::console::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

extern crate rand;
use rand::Rng;

use std::cmp::max;
use std::cmp::min;

const PLAYER: usize = 0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Fighter {
    max_hp: i32,
    hp: i32,
    defense: i32,
    attack: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Ai;

fn normalize(delta: i32) -> i32 {
    if delta < 0 {
        -1
    } else if delta > 0 {
        1
    } else {
        0
    }
}

fn move_towards(id: usize, target_x: i32, target_y: i32, map: &MapSlice, objects: &mut [Object]) {
    let dx = normalize(target_x - objects[id].x);
    let dy = normalize(target_y - objects[id].y);
    if move_by(id, dx, dy, map, objects) == PlayerAction::DidntTakeTurn
        && move_by(id, dx, 0, map, objects) == PlayerAction::DidntTakeTurn
    {
        move_by(id, 0, dy, map, objects);
    }
}

fn ai_take_turn(id: usize, map: &MapSlice, objects: &mut [Object], fov_map: &FovMap) {
    let (x, y) = objects[id].pos();
    if fov_map.is_in_fov(x, y) {
        /*
        let (player_slice, ai_slice) = objects.split_at_mut(id);
        let player = &player_slice[0];
        let ai = &ai_slice[0];
        if ai.grid_distance_to(player) > 1 {
            let (px, py) = player.pos();
            move_towards(0, px, py, map, ai_slice);
        } else {

        }
        */
        if objects[id].grid_distance_to(&objects[PLAYER]) > 1 {
            let (player_x, player_y) = objects[PLAYER].pos();
            move_towards(id, player_x, player_y, map, objects);
        } else if objects[PLAYER].fighter.map_or(false, |f| f.hp > 0) {
            let (player_slice, ai_slice) = objects.split_at_mut(id);
            ai_slice[0].attack(&mut player_slice[0]);
        }
    }
}

#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
    name: String,
    fighter: Option<Fighter>,
    ai: Option<Ai>,
    is_walkable: bool,
    is_alive: bool,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, name: &str, color: Color) -> Self {
        Object {
            x: x,
            y: y,
            char: char,
            color: color,
            name: name.to_string(),
            fighter: None,
            ai: None,
            is_walkable: false,
            is_alive: true,
        }
    }

    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn take_damage(&mut self, damage: i32) {
        if let Some(fighter) = self.fighter.as_mut() {
            if damage > 0 {
                if damage >= fighter.hp {
                    fighter.hp = 0;
                    println!("The {} dies!", self.name);
                } else {
                    fighter.hp -= damage;
                }
            }
        }
    }

    pub fn attack(&self, other: &mut Object) {
        let damage = self.fighter.map_or(0, |f| f.attack) - other.fighter.map_or(0, |f| f.defense);
        if damage > 0 {
            println!("{} attacks {} for {} damage", self.name, other.name, damage);
            other.take_damage(damage);
        } else {
            println!("{} attacks {} but it has no effect!", self.name, other.name);
        }
    }

    pub fn draw(&self, con: &mut Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }

    fn grid_distance_to(&self, other: &Object) -> i32 {
        let dx = (other.x - self.x).abs();
        let dy = (other.y - self.y).abs();
        max(dx, dy)
    }

    /*
    pub fn clear(&self, con: &mut Console) {
        con.put_char(self.x, self.y, ' ', BackgroundFlag::None);
    }
    */
}

//
// map-related stuff
//
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const MAX_ROOMS: i32 = 30;
const MAX_ROOM_WIDTH: i32 = 15;
const MIN_ROOM_WIDTH: i32 = 6;
const MAX_ROOM_HEIGHT: i32 = 10;
const MIN_ROOM_HEIGHT: i32 = 5;
const MAX_ROOM_MONSTERS: i32 = 3;

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_LIGHT_WALL: Color = Color {
    r: 130,
    g: 110,
    b: 50,
};
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};
const COLOR_LIGHT_GROUND: Color = Color {
    r: 200,
    g: 180,
    b: 50,
};

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;
const TORCH_RADIUS: i32 = 10;

fn light_blend(
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    close: Color,
    far: Color,
    max_radius: f32,
) -> Color {
    let dx = (max(x1, x2) - min(x1, x2)) as f32;
    let dy = (max(y1, y2) - min(y1, y2)) as f32;

    let f = (dx * dx + dy * dy) / (max_radius * max_radius);
    // adjacent squares (f ~= 0) should be the close color
    // squares at maximum visible distance (f ~= 1) should be the far color
    close * (1. - f) + far * f
}

#[derive(Clone, Copy, Debug)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }

    pub fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }

    pub fn intersects_with(&self, other: &Rect) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }
}

#[derive(Clone, Copy, Debug)]
struct Tile {
    is_walkable: bool,
    is_transparent: bool,
    explored: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile::new(true, true)
    }

    pub fn wall() -> Self {
        Tile::new(false, false)
    }

    pub fn new(is_walkable: bool, is_transparent: bool) -> Self {
        Tile {
            is_walkable: is_walkable,
            is_transparent: is_transparent,
            explored: false,
        }
    }
}

type Map = Vec<Vec<Tile>>;
type MapSlice = [Vec<Tile>];

fn is_blocked_by_object(x: i32, y: i32, objects: &[Object]) -> bool {
    objects
        .iter()
        .any(|object| !object.is_walkable && object.pos() == (x, y))
}

fn is_blocked(map: &MapSlice, x: i32, y: i32, objects: &[Object]) -> bool {
    if !map[x as usize][y as usize].is_walkable {
        return true;
    }
    is_blocked_by_object(x, y, objects)
}

fn move_by(id: usize, dx: i32, dy: i32, map: &MapSlice, objects: &mut [Object]) -> PlayerAction {
    let (x, y) = objects[id].pos();
    // move by the given amount
    let next_x = x + dx;
    let next_y = y + dy;
    if 0 <= next_x
        && next_x < MAP_WIDTH
        && 0 <= next_y
        && next_y < MAP_HEIGHT
        && !is_blocked(map, next_x, next_y, objects)
    {
        objects[id].set_pos(next_x, next_y);
        return PlayerAction::TookTurn;
    }
    PlayerAction::DidntTakeTurn
}

fn player_move_or_attack(dx: i32, dy: i32, map: &MapSlice, objects: &mut [Object]) -> PlayerAction {
    let (x, y) = objects[PLAYER].pos();
    let next_x = x + dx;
    let next_y = y + dy;

    let target_id = objects
        .iter()
        .position(|object| object.pos() == (next_x, next_y));

    match target_id {
        Some(target_id) => {
            let (player_slice, target_slice) = objects.split_at_mut(target_id);
            player_slice[0].attack(&mut target_slice[0]);
            // println!("The {} says 'Stop poking me!!!'", objects[target_id].name);
            PlayerAction::TookTurn
        }
        None => move_by(PLAYER, dx, dy, map, objects),
    }
}

fn make_room(room: Rect, map: &mut Map) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn make_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    for x in min(x1, x2)..=(max(x1, x2)) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn make_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    for y in min(y1, y2)..=(max(y1, y2)) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn place_objects(room: Rect, objects: &mut Vec<Object>) {
    let mut rng = rand::thread_rng();
    let num_monsters = rng.gen_range(0, MAX_ROOM_MONSTERS + 1);
    for _ in 0..num_monsters {
        let x = rng.gen_range(room.x1 + 1, room.x2);
        let y = rng.gen_range(room.y1 + 1, room.y2);
        if is_blocked_by_object(x, y, objects) {
            continue;
        }

        let mut monster = if rand::random::<f32>() < 0.8 {
            // Create an orc
            let mut orc = Object::new(x, y, 'o', "orc", colors::DESATURATED_GREEN);
            orc.fighter = Some(Fighter {
                max_hp: 10,
                hp: 10,
                defense: 0,
                attack: 3,
            });
            orc.ai = Some(Ai);
            orc
        } else {
            let mut troll = Object::new(x, y, 'T', "troll", colors::DARKER_RED);
            troll.fighter = Some(Fighter {
                max_hp: 16,
                hp: 16,
                defense: 1,
                attack: 4,
            });
            troll.ai = Some(Ai);
            troll
        };

        objects.push(monster);
    }
}

fn make_map(objects: &mut Vec<Object>) -> (Map, (i32, i32)) {
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    let mut rng = rand::thread_rng();
    let mut rooms = vec![];
    let mut starting_position = (0, 0);
    let mut prev_x = 0;
    let mut prev_y = 0;

    for _ in 0..MAX_ROOMS {
        let w = rng.gen_range(MIN_ROOM_WIDTH, MAX_ROOM_WIDTH);
        let h = rng.gen_range(MIN_ROOM_HEIGHT, MAX_ROOM_HEIGHT);
        let room_rect = Rect::new(
            rng.gen_range(0, MAP_WIDTH - w),
            rng.gen_range(0, MAP_HEIGHT - h),
            w,
            h,
        );

        let blocked = rooms
            .iter()
            .any(|other_room| room_rect.intersects_with(other_room));
        if !blocked {
            make_room(room_rect, &mut map);
            place_objects(room_rect, objects);
            let (new_x, new_y) = room_rect.center();
            if rooms.is_empty() {
                starting_position = (new_x, new_y)
            } else if rand::random() {
                make_h_tunnel(prev_x, new_x, prev_y, &mut map);
                make_v_tunnel(prev_y, new_y, new_x, &mut map);
            } else {
                make_v_tunnel(prev_y, new_y, prev_x, &mut map);
                make_h_tunnel(prev_x, new_x, new_y, &mut map);
            }
            prev_x = new_x;
            prev_y = new_y;
            rooms.push(room_rect)
        }
    }
    (map, starting_position)
}

//
// primary game stuff
//
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20;

fn handle_keys(root: &mut Root, objects: &mut [Object], map: &MapSlice) -> PlayerAction {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;
    use PlayerAction::*;

    let key = root.wait_for_keypress(true);
    let player_alive = objects[PLAYER].is_alive;
    match (key, player_alive) {
        (
            Key {
                code: Enter,
                alt: true,
                ..
            },
            _,
        ) => {
            let fullscreen = root.is_fullscreen();
            root.set_fullscreen(!fullscreen);
            DidntTakeTurn
        }
        (Key { code: Escape, .. }, _) => Exit,

        // Movement Keys
        (Key { code: Up, .. }, true) => player_move_or_attack(0, -1, map, objects),
        (Key { code: Down, .. }, true) => player_move_or_attack(0, 1, map, objects),
        (Key { code: Left, .. }, true) => player_move_or_attack(-1, 0, map, objects),
        (Key { code: Right, .. }, true) => player_move_or_attack(1, 0, map, objects),

        _ => DidntTakeTurn,
    }
}

fn render_all(
    root: &mut Root,
    con: &mut Offscreen,
    objects: &[Object],
    map: &mut Map,
    fov_map: &mut FovMap,
    recompute_fov: bool,
) {
    if recompute_fov {
        let player = &objects[PLAYER];
        fov_map.compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
        con.set_default_foreground(colors::WHITE);

        for x in 0..MAP_WIDTH {
            let ux = x as usize;
            for y in 0..MAP_HEIGHT {
                let uy = y as usize;
                let visible = fov_map.is_in_fov(x, y);
                let wall = !map[ux][uy].is_transparent;
                let color = match (visible, wall) {
                    (false, true) => COLOR_DARK_WALL,
                    (false, false) => COLOR_DARK_GROUND,
                    (true, true) => light_blend(
                        player.x,
                        player.y,
                        x,
                        y,
                        COLOR_LIGHT_WALL,
                        COLOR_DARK_WALL,
                        TORCH_RADIUS as f32,
                    ),
                    (true, false) => light_blend(
                        player.x,
                        player.y,
                        x,
                        y,
                        COLOR_LIGHT_GROUND,
                        COLOR_DARK_GROUND,
                        TORCH_RADIUS as f32,
                    ),
                };
                let explored = &mut map[ux][uy].explored;
                if visible {
                    *explored = true;
                }
                if *explored {
                    con.set_char_background(x, y, color, BackgroundFlag::Set);
                    con.put_char(x, y, if wall { '#' } else { '.' }, BackgroundFlag::None);
                }
            }
        }
    }

    for object in objects {
        if fov_map.is_in_fov(object.x, object.y) {
            object.draw(con);
        }
    }

    blit(con, (0, 0), (MAP_WIDTH, MAP_HEIGHT), root, (0, 0), 1.0, 1.0);
    // show the player's stats
    if let Some(fighter) = objects[PLAYER].fighter {
        root.print_ex(
            1,
            SCREEN_HEIGHT - 2,
            BackgroundFlag::None,
            TextAlignment::Left,
            format!("HP: {}/{} ", fighter.hp, fighter.max_hp),
        );
    }
}

fn main() {
    let mut root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Rust/libtcod tutorial")
        .init();

    let mut con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);

    let mut player = Object::new(0, 0, '@', "Player", colors::WHITE);
    player.fighter = Some(Fighter {
        max_hp: 30,
        hp: 30,
        defense: 2,
        attack: 5,
    });

    let mut objects = vec![player];
    let (mut map, (px, py)) = make_map(&mut objects);
    objects[PLAYER].set_pos(px, py);

    let mut fov_map = FovMap::new(MAP_WIDTH, MAP_HEIGHT);
    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            fov_map.set(
                x,
                y,
                map[x as usize][y as usize].is_transparent,
                map[x as usize][y as usize].is_walkable,
            );
        }
    }

    tcod::system::set_fps(LIMIT_FPS);

    let mut previous_pos = (-1, -1);
    while !root.window_closed() {
        /* Doesn't seem to be needed, and blanks out floor characters
        for non-visible objects */
        /*
        for object in &mut objects {
            object.clear(&mut con);
        }
        */

        let recompute_fov = previous_pos != (objects[PLAYER].x, objects[PLAYER].y);
        render_all(
            &mut root,
            &mut con,
            &objects,
            &mut map,
            &mut fov_map,
            recompute_fov,
        );
        root.flush();

        previous_pos = (objects[PLAYER].x, objects[PLAYER].y);
        let player_action = handle_keys(&mut root, &mut objects, &map);
        if player_action == PlayerAction::Exit {
            break;
        }

        // Let monsters take their turn
        if objects[PLAYER].is_alive && player_action != PlayerAction::DidntTakeTurn {
            for id in 0..objects.len() {
                if objects[id].is_alive {
                    print!(".");
                    if objects[id].ai.is_some() {
                        ai_take_turn(id, &map, &mut objects, &fov_map);
                    }
                }
            }
            println!("");
        }
    }
}
