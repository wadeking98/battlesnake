use log::info;
use rand::seq::SliceRandom;
use serde_json::{json, Value};
use crate::{Battlesnake, Board, Game, Coord};
use phf::phf_map;

static DIRECTIONS: phf::Map<&'static str, Coord> = phf_map! {
    "up" => Coord{y: 1, x:0},
    "right" => Coord{y:0, x:1},
    "left" => Coord{y:0, x:-1},
    "down" => Coord{y: -1, x:0},
};

// info is called when you create your Battlesnake on play.battlesnake.com
// and controls your Battlesnake's appearance
// TIP: If you open your Battlesnake URL in a browser you should see this data
pub fn info() -> Value {
    info!("INFO");

    return json!({
        "apiversion": "1",
        "author": "", // TODO: Your Battlesnake Username
        "color": "#888888", // TODO: Choose color
        "head": "default", // TODO: Choose head
        "tail": "default", // TODO: Choose tail
    });
}

// start is called when your Battlesnake begins a game
pub fn start(_game: &Game, _turn: &u32, _board: &Board, _you: &Battlesnake) {
    info!("GAME START");
}

// end is called when your Battlesnake finishes a game
pub fn end(_game: &Game, _turn: &u32, _board: &Board, _you: &Battlesnake) {
    info!("GAME OVER");
}

pub fn can_move(c: Coord, board: &Board) -> bool{
    if board.hazards.contains(&c) {
        return false;
    }
    if c.x >= board.width as i16 || c.x < 0 || c.y >= board.height as i16 || c.y <0 {
        return  false;
    } 
    for snake in &board.snakes{
        if snake.body.contains(&c){
            return false
        }
    }
    return true;
}

// move is called on every turn and returns your next move
// Valid moves are "up", "down", "left", or "right"
// See https://docs.battlesnake.com/api/example-move for available data
pub fn get_move(_game: &Game, turn: &u32, _board: &Board, _you: &Battlesnake) -> Value {
    let moves = vec!["up", "down", "left", "right"];
    let mut safe_moves: Vec<&str> = vec![];
    // We've included code to prevent your Battlesnake from moving backwards
    
    for dir in moves{
        if can_move(DIRECTIONS[dir].add(&_you.head), _board){
            safe_moves.insert(0, dir)
        }
    }
    if safe_moves.len() <= 0 {
        safe_moves.insert(0, "up")
    }

    
    
    // Choose a random move from the safe ones
    let chosen = safe_moves.choose(&mut rand::thread_rng()).unwrap();

    // TODO: Step 4 - Move towards food instead of random, to regain health and survive longer
    // let food = &board.food;

    info!("MOVE {}: {} len:{:?}", turn, chosen, safe_moves);
    return json!({ "move": chosen });
}