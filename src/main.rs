extern crate tcod;

use tcod::console::*;
use tcod::colors;
use tcod::colors::Color;

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
            if !map[next_x as usize][next_y as usize].blocked {
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

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const COLOR_DARK_WALL: Color = Color { r:0, g:0, b:100 };
const COLOR_DARK_GROUND: Color = Color { r:50, g:50, b:150 };

#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {blocked: false, block_sight: false}
    }

    pub fn wall() -> Self {
        Tile {blocked: true, block_sight: true}
    }

    pub fn new(blocked: bool, block_sight: bool) -> Self {
        Tile {
            blocked: blocked,
            block_sight: block_sight,
        }
    }
}

type Map = Vec<Vec<Tile>>;

fn make_map() -> Map {
    let map = vec![vec![Tile::empty(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    map
}

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20;

fn handle_keys(root: &mut Root, player: &mut Object, map: &Map) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    let key = root.wait_for_keypress(true);
    match key {
        Key { code: Enter, alt: true, .. } => {
            let fullscreen = root.is_fullscreen();
            root.set_fullscreen(!fullscreen);
        }
        Key { code: Escape, .. } => return true,

        // Movement Keys
        Key { code: Up, .. } => player.move_by(map, 0, -1),
        Key { code: Down, .. } => player.move_by(map, 0, 1),
        Key { code: Left, .. } => player.move_by(map, -1, 0),
        Key { code: Right, .. } => player.move_by(map, 1, 0),

        _ => {},
    }
    false
}

fn render_all(root: &mut Root, con: &mut Offscreen, objects: &[Object], map: &Map) {
    for object in objects {
        object.draw(con);
    }

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            if map[x as usize][y as usize].block_sight {
                con.set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
            } else {
                con.set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
            }
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
    let mut map = make_map();
    map[30][22] = Tile::wall();
    map[50][22] = Tile::wall();

    let player = Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, '@', colors::WHITE);
    let npc = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', colors::WHITE);
    let mut objects = [player, npc];

    tcod::system::set_fps(LIMIT_FPS);


    while !root.window_closed() {
        render_all(&mut root, &mut con, &objects, &map);
        root.flush();

        for object in &mut objects {
            object.clear(&mut con);
        }

        if handle_keys(&mut root, &mut objects[0], &map) {
            break;
        }
    }
}
