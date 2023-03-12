use serde::Serialize;
use rocket::serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::ops;
use bitflags::bitflags;
use phf::phf_map;

pub static DIRECTIONS: phf::Map<&'static str, Coord> = phf_map! {
    "up" => Coord{y: 1, x:0},
    "right" => Coord{y:0, x:1},
    "left" => Coord{y:0, x:-1},
    "down" => Coord{y: -1, x:0},
};

bitflags! {
    pub struct Flags: u8 {
        const EMPTY = 0x01;
        const FOOD = 0x02;
        const ALLY = 0x04;
        const SNAKE = 0x08;
        const HAZARD = 0x10;
        const BOARD_TILE_FREE_MASK = 0x07;
    }
}

#[macro_export]
macro_rules! board_tile_is_free {
    ($tile:ident) => {
        {
            !($tile & types::Flags::BOARD_TILE_FREE_MASK).is_empty()
        }
    };
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Game {
    pub id: String,
    pub ruleset: HashMap<String, Value>,
    pub timeout: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Board {
    pub height: u8,
    pub width: u8,
    pub food: Vec<Coord>,
    pub snakes: Vec<Battlesnake>,
    pub hazards: Vec<Coord>,
}
fn add_coords_to_board(board: &mut Vec<Vec<Flags>>, points: &Vec<Coord>, value: Flags) {
    for point in points{
        let x = point.x as usize;
        let y = point.y as usize;
        board[x][y] = value;
    }
}
impl Board {
    pub fn to_game_board(&self) -> Vec<Vec<Flags>> {
        let mut board = vec![vec![Flags::EMPTY; self.width.into()]; self.height.into()];

        // populate food
        add_coords_to_board(&mut board, &self.food, Flags::FOOD);

        // populate snakes
        for snake in &self.snakes {

            //populate snake body
            add_coords_to_board(&mut board, &snake.body, Flags::SNAKE);
        }

        // populate hazards
        add_coords_to_board(&mut board, &self.hazards, Flags::HAZARD);
        return board;
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Battlesnake {
    pub id: String,
    pub name: String,
    pub health: u8,
    pub body: Vec<Coord>,
    pub head: Coord,
    pub length: u32,
    // latency: String,
    pub shout: Option<String>,
}
impl PartialEq for Battlesnake{
    fn eq(&self, other: &Self) -> bool {
        return self.id == other.id;
    }
}
impl Battlesnake {
    pub fn move_snake(&mut self, game_board:&mut Vec<Vec<Flags>>, move_to:&Coord){
        self.head = *move_to;
        self.body.insert(0, *move_to);
        if game_board[move_to.x as usize][move_to.y as usize] != Flags::FOOD{
            if self.health > 0 {
                self.health -= 1;
            }
            self.body.pop();
            game_board[move_to.x as usize][move_to.y as usize] = Flags::EMPTY
        }else{
            self.health = 100;
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Copy, Clone)]
pub struct Coord {
    pub x: i16,
    pub y: i16,
}
impl ops::Add<Coord> for Coord {
    type Output = Coord;
    fn add(self, c: Coord) -> Self::Output {
        return Coord {
            x: c.x + self.x,
            y: c.y + self.y,
        };
    }
}
impl ops::Sub<Coord> for Coord{
    type Output = Coord;
    fn sub(self, c: Coord) -> Self::Output {
        return Coord {
            x: self.x - c.x,
            y: self.y - c.y,
        };
    }
}
impl Coord{
    pub fn distance(&self, c: &Coord) -> f32 {
        let vec = *self - *c;
        return ((vec.x.pow(2) + vec.y.pow(2)) as f32).sqrt();
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GameState {
    pub game: Game,
    pub turn: u32,
    pub board: Board,
    pub you: Battlesnake,
}

