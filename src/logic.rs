use std::collections::HashMap;

use crate::{board_tile_is_free, types, get_board_tile, search::graph};
use log::info;
use rand::seq::SliceRandom;
use serde_json::{json, Value};

// info is called when you create your Battlesnake on play.battlesnake.com
// and controls your Battlesnake's appearance
// TIP: If you open your Battlesnake URL in a browser you should see this data
pub fn info() -> Value {
    info!("INFO");

    return json!({
        "apiversion": "1",
        "author": "", // TODO: Your types::Battlesnake Username
        "color": "#888888", // TODO: Choose color
        "head": "default", // TODO: Choose head
        "tail": "default", // TODO: Choose tail
    });
}

// start is called when your types::Battlesnake begins a game
pub fn start(_game: &types::Game, _turn: &u32, _board: &types::Board, _you: &types::Battlesnake) {
    info!("GAME START");
}

// end is called when your types::Battlesnake finishes a game
pub fn end(_game: &types::Game, _turn: &u32, _board: &types::Board, _you: &types::Battlesnake) {
    info!("GAME OVER");
}

// pub fn can_move(c: types::Coord, board: &types::Board) -> bool {
//     if board.hazards.contains(&c) {
//         return false;
//     }
//     if c.x >= board.width as i16 || c.x < 0 || c.y >= board.height as i16 || c.y < 0 {
//         return false;
//     }
//     for snake in &board.snakes {
//         if snake.body.contains(&c) {
//             return false;
//         }
//     }
//     return true;
// }

fn adj_to_bigger_snake(
    c: &types::Coord,
    board: &types::Board,
    you: &types::Battlesnake
) -> bool {
    // calculate distance to other snake heads to see if we are adjacent to snakes with higher health
    for snake in &board.snakes {
        if snake != you {
            let distance = c.distance(&snake.head);
            if distance <= 1.0 && snake.length >= you.length {
                return true;
            }
        }
    }
    return false;
}

macro_rules! can_move_on_tail {
    ($snakes:ident, $coord:ident) => {
        $snakes.into_iter().find(|snake| snake.health < 100 && snake.body[snake.body.len()-1] == *$coord).is_some()
    };
}

pub fn can_move_board(
    c: &types::Coord,
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    avoid_snake_heads_option: Option<bool>,
) -> bool {
    let avoid_snake_heads = avoid_snake_heads_option.unwrap_or(true);
    if c.x as u8 >= board.width || c.y as u8 >= board.height || c.x < 0 || c.y < 0 {
        return false;
    }
    // special case: we can move onto a tile that has the tip of a snake's tail as long as we know that snake hasn't just eaten 
    // if tile is free: Food | Ally | Empty
    let board_tile = get_board_tile!(game_board, c.x, c.y);
    let snakes = &board.snakes;
    if board_tile_is_free!(board_tile) || (board_tile == types::Flags::SNAKE && can_move_on_tail!(snakes, c)) {
        // if tile is adjacent to head, only return true if we can't move anywhere else
        if adj_to_bigger_snake(c, board, you) && avoid_snake_heads {
            return false;
        }
        return true;
    }
    return false;
}

fn get_rand_moves(from_point: &types::Coord, board: &types::Board, game_board: &HashMap<types::Coord, types::Flags>, you: &types::Battlesnake) -> Vec<&'static str>{
    let moves = vec!["up", "down", "left", "right"];
    let mut safe_moves: Vec<&str> = Vec::new();
    for dir in &moves {
        let new_coord = types::DIRECTIONS[dir] + *from_point;
        if can_move_board(&new_coord, board, game_board, you, None) {
            safe_moves.insert(0, dir)
        }
    }
    if safe_moves.len() <= 0 {
        // insert any unsafe moves (ie: adjacent to the heads of bigger snakes)
        for dir in &moves {
            let new_coord = types::DIRECTIONS[dir] + *from_point;
            if can_move_board(
                &new_coord,
                board,
                game_board,
                you,
                Some(false),
            ) {
                safe_moves.insert(0, dir)
            }
        }

        //if we still don't have any moves just go up
        if safe_moves.len() <= 0 {
            safe_moves.insert(0, "up")
        }
    }
    return safe_moves;
}

// move is called on every turn and returns your next move
// Valid moves are "up", "down", "left", or "right"
// See https://docs.battlesnake.com/api/example-move for available data
pub fn get_move(
    _game: &types::Game,
    turn: &u32,
    _board: &types::Board,
    _you: &types::Battlesnake,
) -> Value {
    let game_board = _board.to_game_board();
    
    let mut safe_moves: Vec<&str> = vec![];

    let mut you_copy = _you.clone();
    you_copy.health -= 1;

    // move towards closest connected food
    let path = graph::bfs(_board, &game_board, &_you, Some(1));

    if path.len() > 0 {
        let dir_vector = path[0] - _you.head;
        let dir = types::DIRECTIONS.into_iter().find_map(|(key, &val)| if val == dir_vector {Some(key)} else {None});
        if dir.is_some(){
            safe_moves.push(dir.unwrap());
        }
    } else{
        let mut rand_moves = get_rand_moves(&_you.head, _board, &game_board, _you);
        safe_moves.append(&mut rand_moves);
    }
    // This Code is messy but we'll remove it once we get BFS/MiniMax working
    

    // Choose a random move from the safe ones
    let chosen = safe_moves.choose(&mut rand::thread_rng()).unwrap();

    // TODO: Step 4 - Move towards food instead of random, to regain health and survive longer
    // let food = &board.food;

    info!("MOVE {}: {} len:{:?}", turn, chosen, safe_moves);
    return json!({ "move": chosen });
}

#[cfg(test)]
mod tests {
    use crate::types::{self, Coord};

    use super::*;

    #[test]
    fn avoid_wall() {
        static YOU_DATA: &str = r#"
    {
        "id": "GUODB",
        "name": "snake GUODB",
        "health": 100,
        "body": [
          {
            "x": 5,
            "y": 10
          },
          {
            "x": 5,
            "y": 9
          },
          {
            "x": 5,
            "y": 8
          },
          {
            "x": 5,
            "y": 7
          }
        ],
        "latency": 0,
        "head": {
          "x": 5,
          "y": 10
        },
        "length": 4,
        "shout": "",
        "squad": ""
      }
    "#;

        static WALL_DATA: &str = r#"{
        "food": [],
        "snakes": [
          {
            "id": "GUODB",
            "name": "snake GUODB",
            "health": 100,
            "body": [
              {
                "x": 5,
                "y": 10
              },
              {
                "x": 5,
                "y": 9
              },
              {
                "x": 5,
                "y": 8
              },
              {
                "x": 5,
                "y": 7
              }
            ],
            "latency": 0,
            "head": {
              "x": 5,
              "y": 10
            },
            "length": 4,
            "shout": "",
            "squad": ""
          }
        ],
        "width": 11,
        "height": 11,
        "hazards": []
      }"#;

        let board: types::Board = serde_json::from_str(WALL_DATA).unwrap();
        let mut you: types::Battlesnake = serde_json::from_str(YOU_DATA).unwrap();
        you.health -= 1;
        let game_board = board.to_game_board();
        let point = Coord { x: 5, y: 11 };

        assert!(!can_move_board(
            &point,
            &board,
            &game_board,
            &you,
            None
        ));
    }

    #[test]
    fn avoid_snake_tail() {
        static BOARD_DATA: &str = r#"
        {
            "food": [],
            "snakes": [
              {
                "id": "unnda",
                "name": "snake unnda",
                "health": 100,
                "body": [
                  {
                    "x": 3,
                    "y": 3
                  },
                  {
                    "x": 3,
                    "y": 4
                  },
                  {
                    "x": 2,
                    "y": 4
                  },
                  {
                    "x": 2,
                    "y": 5
                  },
                  {
                    "x": 2,
                    "y": 6
                  },
                  {
                    "x": 2,
                    "y": 7
                  },
                  {
                    "x": 3,
                    "y": 7
                  },
                  {
                    "x": 4,
                    "y": 7
                  },
                  {
                    "x": 5,
                    "y": 7
                  },
                  {
                    "x": 6,
                    "y": 7
                  },
                  {
                    "x": 7,
                    "y": 7
                  }
                ],
                "latency": 0,
                "head": {
                  "x": 3,
                  "y": 3
                },
                "length": 11,
                "shout": "",
                "squad": ""
              },
              {
                "id": "q1pji",
                "name": "snake q1pji",
                "health": 100,
                "body": [
                  {
                    "x": 3,
                    "y": 6
                  },
                  {
                    "x": 3,
                    "y": 5
                  },
                  {
                    "x": 4,
                    "y": 5
                  },
                  {
                    "x": 5,
                    "y": 5
                  }
                ],
                "latency": 0,
                "head": {
                  "x": 3,
                  "y": 6
                },
                "length": 4,
                "shout": "",
                "squad": ""
              }
            ],
            "width": 11,
            "height": 11,
            "hazards": []
          }
        "#;
        static YOU_DATA: &str = r#"
        {
            "id": "q1pji",
            "name": "snake q1pji",
            "health": 100,
            "body": [
              {
                "x": 3,
                "y": 6
              },
              {
                "x": 3,
                "y": 5
              },
              {
                "x": 4,
                "y": 5
              },
              {
                "x": 5,
                "y": 5
              }
            ],
            "latency": 0,
            "head": {
              "x": 3,
              "y": 6
            },
            "length": 4,
            "shout": "",
            "squad": ""
          }
        "#;
        let board: types::Board = serde_json::from_str(BOARD_DATA).unwrap();
        let mut you: types::Battlesnake = serde_json::from_str(YOU_DATA).unwrap();
        you.health -= 1;
        let game_board = board.to_game_board();
        assert!(!can_move_board(
            &Coord { x: 2, y: 6 },
            &board,
            &game_board,
            &you,
            None
        ));
        assert!(can_move_board(
            &Coord { x: 4, y: 6 },
            &board,
            &game_board,
            &you,
            None
        ));
    }

    #[test]
    fn avoid_head_to_head() {
        const BOARD_DATA: &str = r#"
        {
            "food": [
              {
                "x": 5,
                "y": 5
              }
            ],
            "snakes": [
              {
                "id": "mTOl1",
                "name": "snake mTOl1",
                "health": 80,
                "body": [
                  {
                    "x": 4,
                    "y": 5
                  },
                  {
                    "x": 3,
                    "y": 5
                  },
                  {
                    "x": 2,
                    "y": 5
                  },
                  {
                    "x": 1,
                    "y": 5
                  }
                ],
                "latency": 0,
                "head": {
                  "x": 4,
                  "y": 5
                },
                "length": 4,
                "shout": "",
                "squad": ""
              },
              {
                "id": "uZejq",
                "name": "snake uZejq",
                "health": 80,
                "body": [
                  {
                    "x": 5,
                    "y": 4
                  },
                  {
                    "x": 5,
                    "y": 3
                  },
                  {
                    "x": 5,
                    "y": 2
                  },
                  {
                    "x": 5,
                    "y": 1
                  }
                ],
                "latency": 0,
                "head": {
                  "x": 5,
                  "y": 4
                },
                "length": 4,
                "shout": "",
                "squad": ""
              }
            ],
            "width": 11,
            "height": 11,
            "hazards": []
          }
        "#;

        const YOU_DATA: &str = r#"
        {
            "id": "uZejq",
            "name": "snake uZejq",
            "health": 80,
            "body": [
              {
                "x": 5,
                "y": 4
              },
              {
                "x": 5,
                "y": 3
              },
              {
                "x": 5,
                "y": 2
              },
              {
                "x": 5,
                "y": 1
              }
            ],
            "latency": 0,
            "head": {
              "x": 5,
              "y": 4
            },
            "length": 4,
            "shout": "",
            "squad": ""
          }
        "#;
        let board: types::Board = serde_json::from_str(BOARD_DATA).unwrap();
        let mut you: types::Battlesnake = serde_json::from_str(YOU_DATA).unwrap();
        you.health -= 1;
        let game_board = board.to_game_board();
        assert!(!can_move_board(
            &Coord { x: 5, y: 5 },
            &board,
            &game_board,
            &you,
            None
        ));
        assert!(can_move_board(
            &Coord { x: 6, y: 4 },
            &board,
            &game_board,
            &you,
            None
        ));
    }
}
