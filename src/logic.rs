use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet, VecDeque},
};

use crate::{
    board_tile_is_free, get_board_tile,
    search::graph,
    types::{self, Coord},
};
use log::info;
use rand::seq::SliceRandom;
use rand::thread_rng;
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

pub fn get_adj_tiles(
    tile: &types::Coord,
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    avoid_snake_heads_option: Option<bool>,
) -> Vec<types::Coord> {
    let mut adj: Vec<types::Coord> = vec![];
    for (.., dir) in types::DIRECTIONS.into_iter() {
        let new_point = *dir + *tile;
        if can_move_board(&new_point, board, game_board, you, avoid_snake_heads_option) {
            adj.push(new_point)
        }
    }
    return adj;
}

/// # num_free_tiles:  
/// returns the number of free tiles on a board.  
/// We need to count the occupied tiles using a hashset because some tiles can multiple board entities. (ie: overlapping snake bodies, hazard and food, etc)
/// ## Arguments:
/// * board - reference to board object
/// ## Returns:
/// The number of free tiles on the board
fn num_free_tiles(board: &types::Board) -> u8 {
    let mut occupied_tiles: HashSet<types::Coord> = HashSet::new();
    for snake in &board.snakes {
        occupied_tiles.extend(&snake.body);
    }
    for hazard in &board.hazards {
        occupied_tiles.insert(*hazard);
    }
    return board.height * board.width - occupied_tiles.len() as u8;
}

fn num_connected_tiles(
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    frontier: &mut VecDeque<types::Coord>,
    visited: &mut HashSet<types::Coord>,
) -> u8 {
    if frontier.len() <= 0 {
        return 1;
    }
    let current_tile = frontier.pop_front().unwrap();
    let adj_tiles: Vec<types::Coord> = get_adj_tiles(&current_tile, board, game_board, you, None)
        .into_iter()
        .filter(|adj| visited.get(adj).is_none())
        .collect();
    visited.extend(adj_tiles.clone());
    let mut adj_deque = VecDeque::from(adj_tiles);
    frontier.append(&mut adj_deque);
    return 1 + num_connected_tiles(board, game_board, you, frontier, visited);
}

fn percent_connected(
    tile: &types::Coord,
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
) -> f32 {
    let free_tiles = num_free_tiles(board);

    let mut frontier = VecDeque::from([*tile]);
    let mut visited: HashSet<types::Coord> = HashSet::new();
    let connected_tiles = num_connected_tiles(board, game_board, you, &mut frontier, &mut visited);

    if free_tiles == 0 {
        return 0.0;
    } else {
        return connected_tiles as f32 / free_tiles as f32;
    }
}

fn coords_diverge(
    tile: &types::Coord,
    unit_coords: (&types::Coord, &types::Coord),
    game_board: &HashMap<types::Coord, types::Flags>,
) -> bool {
    let (unit_coord1, unit_coord2) = unit_coords;
    let unit_vec = *unit_coord1 + *unit_coord2;
    let vec = unit_vec + *tile;
    let unit_vec_val = get_board_tile!(game_board, vec.x, vec.y);
    return unit_vec == (Coord { x: 0, y: 0 }) || !board_tile_is_free!(unit_vec_val);
}

fn favourable_divergent_coords<'a>(
    tile: &types::Coord,
    unit_moves: [&'a types::Coord; 2],
    game_board: &HashMap<types::Coord, types::Flags>,
    board: &types::Board,
    you: &types::Battlesnake,
    threshold: f32,
    strict: bool,
) -> Vec<(&'a types::Coord, f32)> {
    let connected_unit_moves: Vec<(&types::Coord, f32)> = unit_moves
        .into_iter()
        .map(|mv| {
            (
                mv,
                percent_connected(&(*tile + *mv), board, game_board, you),
            )
        })
        .collect();
    let mut connected_unit_moves_filtered: Vec<(&types::Coord, f32)> = connected_unit_moves
        .clone()
        .into_iter()
        .filter(|(_, conn)| *conn >= threshold)
        .collect();
    if connected_unit_moves_filtered.len() <= 0 && !strict {
        connected_unit_moves_filtered = connected_unit_moves;
    }
    connected_unit_moves_filtered
        .sort_by(|(_, a_conn), (_, b_conn)| (*a_conn).partial_cmp(b_conn).unwrap());
    return connected_unit_moves_filtered;
}

pub fn get_adj_tiles_connected(
    tile: &types::Coord,
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    threshold: f32,
    strict_option: Option<bool>,
    avoid_snake_heads_option: Option<bool>,
) -> Vec<types::Coord> {
    let strict = strict_option.unwrap_or(false);

    let mut moves = get_adj_tiles(tile, board, game_board, you, avoid_snake_heads_option);
    // shuffle moves if they're equally connected
    moves.shuffle(&mut thread_rng());
    let unit_moves: Vec<types::Coord> = (&moves).into_iter().map(|adj| *adj - *tile).collect();
    if unit_moves.len() == 2 {
        if coords_diverge(tile, (&unit_moves[0], &unit_moves[1]), game_board) {
            return favourable_divergent_coords(
                tile,
                [&unit_moves[0], &unit_moves[1]],
                game_board,
                board,
                you,
                threshold,
                strict,
            )
            .into_iter()
            .map(|(mv, _)| *mv + *tile)
            .collect();
        } else {
            return moves;
        }
    } else if unit_moves.len() == 3 {
        let forward_vec = unit_moves[0] + unit_moves[1] + unit_moves[2];
        let side_moves: Vec<Coord> = unit_moves
            .into_iter()
            .filter(|mv| *mv != forward_vec)
            .collect();
        if side_moves.len() != 2 {
            return vec![];
        }
        //find the best connected moves on one side of the head
        let mut favouravble_moves = favourable_divergent_coords(
            tile,
            [&forward_vec, &side_moves[0]],
            game_board,
            board,
            you,
            threshold,
            strict,
        );
        //find the best connected moves on the other side of the head
        let mut favouravble_moves_2 = favourable_divergent_coords(
            tile,
            [&forward_vec, &side_moves[1]],
            game_board,
            board,
            you,
            threshold,
            strict,
        );
        favouravble_moves.append(&mut favouravble_moves_2);
        favouravble_moves.sort_by(|(_, a_conn), (_, b_conn)| a_conn.partial_cmp(b_conn).unwrap());
        favouravble_moves.dedup();

        // if strict is off, we may have parts of an array that pass the connected threshold value and parts that don't because we looked at both sides of the head
        // if any part of the array passes the connected threshold, filter the whole array to only include values that pass that threshold
        let mut favourable_moves_filtered: Vec<(&types::Coord, f32)> = favouravble_moves
            .clone()
            .into_iter()
            .filter(|(_, val)| {
                let order = (*val).partial_cmp(&threshold).unwrap();
                return order == Ordering::Greater || order == Ordering::Equal;
            })
            .collect();
        if favourable_moves_filtered.len() <= 0 {
            favourable_moves_filtered = favouravble_moves;
        }
        return favourable_moves_filtered
            .into_iter()
            .map(|(mv, _)| *mv + *tile)
            .collect();
    }
    return moves;
}

fn adj_to_bigger_snake(c: &types::Coord, board: &types::Board, you: &types::Battlesnake) -> bool {
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
        $snakes
            .into_iter()
            .find(|snake| snake.health < 100 && snake.body[snake.body.len() - 1] == *$coord)
            .is_some()
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
    if board_tile_is_free!(board_tile)
        || (board_tile == types::Flags::SNAKE && can_move_on_tail!(snakes, c))
    {
        // if tile is adjacent to head, only return true if we can't move anywhere else
        if adj_to_bigger_snake(c, board, you) && avoid_snake_heads {
            return false;
        }
        return true;
    }
    return false;
}

fn get_rand_moves(
    from_point: &types::Coord,
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    threshold: f32,
) -> Vec<&'static str> {
    let mut safe_moves = get_adj_tiles_connected(
        from_point,
        board,
        game_board,
        you,
        threshold,
        Some(false),
        None,
    );
    if safe_moves.len() <= 0 {
        safe_moves = get_adj_tiles_connected(
            from_point,
            board,
            game_board,
            you,
            threshold,
            Some(false),
            Some(false),
        );
    }
    let mut move_words: Vec<&str> = Vec::new();
    for mv in safe_moves {
        let dir_option = types::DIRECTIONS
            .into_iter()
            .find_map(|(&key, &val)| if val == (mv - *from_point) { Some(key) } else { None });
        if dir_option.is_some(){
          move_words.push(dir_option.unwrap());
        }
    }
    
    return move_words;
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

    let tile_connection_threshold = 0.5;
    // move towards closest connected food
    let path = graph::bfs(_board, &game_board, &_you, tile_connection_threshold);

    if path.len() > 0 {
        let dir_vector = path[0] - _you.head;
        let dir = types::DIRECTIONS.into_iter().find_map(|(key, &val)| {
            if val == dir_vector {
                Some(key)
            } else {
                None
            }
        });
        if dir.is_some() {
            safe_moves.push(dir.unwrap());
        }
    } else {
        let mut rand_moves = get_rand_moves(
            &_you.head,
            _board,
            &game_board,
            _you,
            tile_connection_threshold,
        );
        safe_moves.append(&mut rand_moves);
    }
    // This Code is messy but we'll remove it once we get BFS/MiniMax working

    let chosen = safe_moves.last().unwrap_or(&"up");

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

        assert!(!can_move_board(&point, &board, &game_board, &you, None));
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
    #[test]
    fn avoid_poorly_connected_tiles() {
        const BOARD_DATA: &str = r#"
        {
          "food": [],
          "snakes": [
            {
              "id": "2j__G",
              "name": "snake 2j__G",
              "health": 100,
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
                  "x": 3,
                  "y": 6
                },
                {
                  "x": 3,
                  "y": 7
                },
                {
                  "x": 3,
                  "y": 8
                },
                {
                  "x": 4,
                  "y": 8
                },
                {
                  "x": 5,
                  "y": 8
                },
                {
                  "x": 6,
                  "y": 8
                },
                {
                  "x": 7,
                  "y": 8
                },
                {
                  "x": 7,
                  "y": 7
                },
                {
                  "x": 7,
                  "y": 6
                },
                {
                  "x": 7,
                  "y": 5
                },
                {
                  "x": 7,
                  "y": 4
                },
                {
                  "x": 6,
                  "y": 4
                },
                {
                  "x": 5,
                  "y": 4
                }
              ],
              "latency": 0,
              "head": {
                "x": 4,
                "y": 5
              },
              "length": 15,
              "shout": "",
              "squad": ""
            }
          ],
          "width": 11,
          "height": 11,
          "hazards": []
        }
      "#;
        let board: types::Board = serde_json::from_str(BOARD_DATA).unwrap();
        let game_board = board.to_game_board();
        let you: &types::Battlesnake = &board.snakes[0];
        let mut connected_tiles =
            get_adj_tiles_connected(&you.head, &board, &game_board, you, 0.8, Some(true), None);
        assert!(connected_tiles[0] == Coord { x: 4, y: 4 });
        connected_tiles =
            get_adj_tiles_connected(&you.head, &board, &game_board, you, 0.01, Some(true), None);
        assert!(
            connected_tiles.len() == 3
                && connected_tiles[connected_tiles.len() - 1] == Coord { x: 4, y: 4 }
        );
    }
}
