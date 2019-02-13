extern crate tcod;

use tcod::colors;
use tcod::colors::Color;
use tcod::console::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

extern crate rand;
use rand::Rng;

use std::cmp::max;
use std::cmp::min;

#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Object {
            x: x,
            y: y,
            char: char,
            color: color,
        }
    }

    pub fn move_by(&mut self, map: &Map, dx: i32, dy: i32) {
        // move by the given amount
        let next_x = self.x + dx;
        let next_y = self.y + dy;
        if 0 <= next_x && next_x < MAP_WIDTH && 0 <= next_y && next_y < MAP_HEIGHT {
            if map[next_x as usize][next_y as usize].is_walkable {
                self.x += dx;
                self.y += dy;
            }
        }
    }

    pub fn draw(&self, con: &mut Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }

    pub fn clear(&self, con: &mut Console) {
        con.put_char(self.x, self.y, ' ', BackgroundFlag::None);
    }
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

fn make_room(room: Rect, map: &mut Map) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn make_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    for x in min(x1, x2)..(max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn make_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    for y in min(y1, y2)..(max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn make_map() -> (Map, (i32, i32)) {
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
            let (new_x, new_y) = room_rect.center();
            if rooms.is_empty() {
                starting_position = (new_x, new_y)
            } else {
                if rand::random() {
                    make_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    make_v_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    make_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    make_h_tunnel(prev_x, new_x, new_y, &mut map);
                }
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

fn handle_keys(root: &mut Root, player: &mut Object, map: &Map) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    let key = root.wait_for_keypress(true);
    match key {
        Key {
            code: Enter,
            alt: true,
            ..
        } => {
            let fullscreen = root.is_fullscreen();
            root.set_fullscreen(!fullscreen);
        }
        Key { code: Escape, .. } => return true,

        // Movement Keys
        Key { code: Up, .. } => player.move_by(map, 0, -1),
        Key { code: Down, .. } => player.move_by(map, 0, 1),
        Key { code: Left, .. } => player.move_by(map, -1, 0),
        Key { code: Right, .. } => player.move_by(map, 1, 0),

        _ => {}
    }
    false
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
        let player = &objects[0];
        fov_map.compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);

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
}

fn main() {
    let mut root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Rust/libtcod tutorial")
        .init();

    let mut con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);
    let (mut map, (px, py)) = make_map();

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

    let player = Object::new(px, py, '@', colors::WHITE);
    let npc = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', colors::WHITE);
    let mut objects = [player, npc];

    tcod::system::set_fps(LIMIT_FPS);

    let mut previous_pos = (-1, -1);
    while !root.window_closed() {
        let recompute_fov = previous_pos != (objects[0].x, objects[0].y);
        render_all(
            &mut root,
            &mut con,
            &objects,
            &mut map,
            &mut fov_map,
            recompute_fov,
        );
        root.flush();

        for object in &mut objects {
            object.clear(&mut con);
        }

        previous_pos = (objects[0].x, objects[0].y);
        if handle_keys(&mut root, &mut objects[0], &map) {
            break;
        }
    }
}
