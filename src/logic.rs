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
use serde_json::{json, Value};

// info is called when you create your Battlesnake on play.battlesnake.com
// and controls your Battlesnake's appearance
// TIP: If you open your Battlesnake URL in a browser you should see this data
pub fn info() -> Value {
    info!("INFO");

    return json!({
        "apiversion": "1",
        "author": "tofurky", // TODO: Your types::Battlesnake Username
        "color": "#c76d0c", // TODO: Choose color
        "head": "chicken", // TODO: Choose head
        "tail": "mlh-gene", // TODO: Choose tail
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

/// # get_adj_tiles
/// gets the tiles adjacent to a given tile that are safe to move on
/// ## Arguments:
/// * tile - the tile in question
/// * board - the battlesnake game board
/// * game_board - the hashmap representation of the game board
/// * you - your battlesnake
/// * avoid_snake_heads_option - option to avoid tiles adjacent to the heads of larger snakes
/// ## Returns:
/// vector of tiles adjacent to the given tile that the snake can move to
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

/// # num_free_tiles  
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

/// # num_connected_tiles
/// gets the number of tiles connected to the first element in the frontier
/// ## Arguments:
/// * board - the battlesnake game board
/// * game_board - the hashmap representation of the game board
/// * you - your battlesnake
/// * frontier - used to track tiles on the edge of our explored set
/// * visited - used to track the tiles that we've already visited and their parents
/// * exclude_tiles - list of tiles to exclude from flood fill, useful when we want to calculate connectivity of a tile given a snake's future position
/// ## Returns:
/// the number of tiles connected to a supplied tile in the frontier
fn num_connected_tiles(
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    frontier: &mut VecDeque<types::Coord>,
    visited: &mut HashSet<types::Coord>,
    exclude_tiles: &Vec<types::Coord>
) -> u8 {
    if frontier.len() <= 0 {
        return 1;
    }
    let current_tile = frontier.pop_front().unwrap();
    let adj_tiles: Vec<types::Coord> = get_adj_tiles(&current_tile, board, game_board, you, None)
        .into_iter()
        .filter(|adj| visited.get(adj).is_none() && !exclude_tiles.contains(adj))
        .collect();
    visited.extend(adj_tiles.clone());
    let mut adj_deque = VecDeque::from(adj_tiles);
    frontier.append(&mut adj_deque);
    return 1 + num_connected_tiles(board, game_board, you, frontier, visited, exclude_tiles);
}

/// # percent_connected
/// gets the percentage of game tiles connected to the first element in the frontier
/// ## Arguments:
/// * tile - the tile in question
/// * board - the battlesnake game board
/// * game_board - the hashmap representation of the game board
/// * you - your battlesnake
/// * exclude_tiles - list of tiles to exclude from flood fill, useful when we want to calculate connectivity of a tile given a snake's future position
/// ## Returns:
/// the total percentage of tiles connected to a given tile
fn percent_connected(
    tile: &types::Coord,
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    exclude_tiles: &Vec<types::Coord>,
) -> f32 {
    let free_tiles = num_free_tiles(board);

    let mut frontier = VecDeque::from([*tile]);
    let mut visited: HashSet<types::Coord> = HashSet::new();
    let connected_tiles = num_connected_tiles(board, game_board, you, &mut frontier, &mut visited, exclude_tiles);

    if free_tiles == 0 {
        return 0.0;
    } else {
        return connected_tiles as f32 / free_tiles as f32;
    }
}

/// # coords_diverge
/// determines if two tiles, adjacent to the head of the snake may be disconnected
/// ## Arguments:
/// * tile - the tile in question (usually the head of the snake)
/// * unit_coords - two directions represented as unit coords (ie: "right" would be {x: 1, y: 0})
/// * game_board - the hashmap representation of the game board
/// ## Returns:
/// true if it's possible that paths starting from the two directions will not be connected
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

/// # favourable_divergent_coords
/// given that two tiles may not be connected, determine the most connected tile
/// ## Arguments:
/// * tiles - the two tiles to test
/// * board - the battlesnake game board
/// * game_board - the hashmap representation of the game board
/// * you - your battlesnake
/// * exclude_tiles - list of tiles to exclude from flood fill, useful when we want to calculate connectivity of a tile given a snake's future position
/// * threshold - the percentage of total free tiles you want to be connected to
/// * strict - true if you want to exclude all provided tiles that fall below the given threshold
/// ## Returns:
/// if strict is true it returns a reference to all the provided tiles that are connected above the threshold, otherwise it returns an array of
/// tiles and their corresponding connectivity index sorted in order from least connected to most
fn favourable_divergent_coords<'a>(
    tiles: [&'a types::Coord; 2],
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    exclude_tiles: &Vec<types::Coord>,
    threshold: f32,
    strict: bool,
) -> Vec<(&'a types::Coord, f32)> {
    let connected_unit_moves: Vec<(&types::Coord, f32)> = tiles
        .into_iter()
        .map(|tile| (tile, percent_connected(tile, board, game_board, you, exclude_tiles)))
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

/// # distance_to_center
/// determines the distance from a given tile to the center of the board
/// ## Arguments:
/// * tile - the tile in question
/// * board - the battlesnake game board
/// ## Returns:
/// the float distance from the given tile to the center
fn distance_to_center(tile: &types::Coord, board: &types::Board) -> f32 {
    let center = Coord {
        x: board.width as i16 / 2,
        y: board.height as i16 / 2,
    };
    return tile.distance(&center);
}

/// # get_adj_tiles_connected
/// gets the tiles adjacent to a given tile that are safe to move on and are sufficiently connected
/// ## Arguments:
/// * tile - the tile in question
/// * board - the battlesnake game board
/// * game_board - the hashmap representation of the game board
/// * you - your battlesnake
/// * theshold - the desired connectedness of any adjacent tiles
/// * strict - if true then only tiles that fall above the connectedness threshold will be returned
/// * avoid_snake_heads_option - option to avoid tiles adjacent to the heads of larger snakes
/// * current_planned_moves_option - option to avoid the provided tiles
/// ## Returns:
/// if strict is true then ot returns all adjacent tiles that pass the connectedness threshold,
/// else it returns all adjacent tiles in order of least to most connected
pub fn get_adj_tiles_connected(
    tile: &types::Coord,
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    threshold: f32,
    strict_option: Option<bool>,
    avoid_snake_heads_option: Option<bool>,
    current_planned_moves_option: Option<Vec<types::Coord>>,
) -> Vec<types::Coord> {
    let strict = strict_option.unwrap_or(false);
    let current_planned_moves: Vec<types::Coord> = current_planned_moves_option.unwrap_or(vec![]);

    // get adjacent moves if they don't loop back on the same path
    let mut moves: Vec<types::Coord> = get_adj_tiles(tile, board, game_board, you, avoid_snake_heads_option)
        .into_iter()
        .filter(|item| !current_planned_moves.contains(item)).collect();
    //sort moves by distance to center if they're equally connected
    moves.sort_by(|a, b| {
        distance_to_center(b, board)
            .partial_cmp(&distance_to_center(a, board))
            .unwrap()
    });
    let unit_moves: Vec<types::Coord> = (&moves).into_iter().map(|adj| *adj - *tile).collect();
    if unit_moves.len() == 2 {
        if coords_diverge(tile, (&unit_moves[0], &unit_moves[1]), game_board) {
            return favourable_divergent_coords(
                [&moves[0], &moves[1]],
                board,
                game_board,
                you,
                &current_planned_moves,
                threshold,
                strict,
            )
            .into_iter()
            .map(|(mv, _)| *mv)
            .collect();
        } else {
            return moves;
        }
    } else if unit_moves.len() == 3 {
        let forward_unit_vec = unit_moves[0] + unit_moves[1] + unit_moves[2];
        let side_unit_moves: Vec<Coord> = unit_moves
            .into_iter()
            .filter(|mv| *mv != forward_unit_vec)
            .collect();
        if side_unit_moves.len() != 2 {
            return vec![];
        }

        // if none of the coords take a divergent path then they are all equally connected, skip calculations
        if !(coords_diverge(tile, (&forward_unit_vec, &side_unit_moves[0]), game_board)
            || coords_diverge(tile, (&forward_unit_vec, &side_unit_moves[1]), game_board))
        {
            return moves;
        }

        let side_moves: Vec<types::Coord> = side_unit_moves
            .into_iter()
            .map(|item| item + *tile)
            .collect();
        let forward_vec = forward_unit_vec + *tile;
        //find the best connected moves on one side of the head
        let mut favouravble_moves_1 = favourable_divergent_coords(
            [&forward_vec, &side_moves[0]],
            board,
            game_board,
            you,
            &current_planned_moves,
            threshold,
            strict,
        );
        //find the best connected moves on the other side of the head
        let mut favouravble_moves_2 = favourable_divergent_coords(
            [&forward_vec, &side_moves[1]],
            board,
            game_board,
            you,
            &current_planned_moves,
            threshold,
            strict,
        )
        .into_iter()
        .filter(|&item| !favouravble_moves_1.contains(&item))
        .collect();
        let mut favourable_moves = Vec::new();
        favourable_moves.append(&mut favouravble_moves_1);
        favourable_moves.append(&mut favouravble_moves_2);

        // sort by most connected
        favourable_moves.sort_by(|&(_, a_conn), &(_, b_conn)| a_conn.partial_cmp(&b_conn).unwrap());

        // if strict is off, we may have parts of an array that pass the connected threshold value and parts that don't because we looked at both sides of the head
        // if any part of the array passes the connected threshold, filter the whole array to only include values that pass that threshold
        let mut favourable_moves_filtered: Vec<(&types::Coord, f32)> = favourable_moves
            .clone()
            .into_iter()
            .filter(|(_, val)| {
                let order = (*val).partial_cmp(&threshold).unwrap();
                return order == Ordering::Greater || order == Ordering::Equal;
            })
            .collect();
        if favourable_moves_filtered.len() <= 0 {
            favourable_moves_filtered = favourable_moves;
        }
        return favourable_moves_filtered
            .into_iter()
            .map(|(mv, _)| *mv)
            .collect();
    }
    return moves;
}

/// # adj_to_bigger_snake
/// determines if a tile is adjacent to the head of a bigger snake
/// ## Arguments:
/// * tile - the tile in question
/// * board - the battlesnake game board
/// * you - your battlesnake
/// ## Returns:
/// true if the given tile is adjacent to the head of a bigger snake
fn adj_to_bigger_snake(
    tile: &types::Coord,
    board: &types::Board,
    you: &types::Battlesnake,
) -> bool {
    // calculate distance to other snake heads to see if we are adjacent to snakes with higher health
    for snake in &board.snakes {
        if snake != you {
            let distance = tile.distance(&snake.head);
            if distance <= 1.0 && snake.length >= you.length {
                return true;
            }
        }
    }
    return false;
}

/// # can_move_on_tail
/// determines if it is safe to move on another snake's tail
/// ## Arguments:
/// * snakes - array of battlesnakes
/// * coord - the tile in question
/// ## Returns:
/// true if we can safely move to coord
macro_rules! can_move_on_tail {
    ($snakes:ident, $coord:ident) => {
        $snakes
            .into_iter()
            .find(|snake| snake.health < 100 && snake.body[snake.body.len() - 1] == *$coord)
            .is_some()
    };
}

/// # can_move_board
/// gets the tiles adjacent to a given tile that are safe to move on
/// ## Arguments:
/// * tile - the tile in question
/// * board - the battlesnake game board
/// * game_board - the hashmap representation of the game board
/// * you - your battlesnake
/// * avoid_snake_heads_option - option to avoid tiles adjacent to the heads of larger snakes
/// ## Returns:
/// true if we can safely move onto tile
pub fn can_move_board(
    tile: &types::Coord,
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    avoid_snake_heads_option: Option<bool>,
) -> bool {
    let avoid_snake_heads = avoid_snake_heads_option.unwrap_or(true);
    if tile.x as u8 >= board.width || tile.y as u8 >= board.height || tile.x < 0 || tile.y < 0 {
        return false;
    }
    // special case: we can move onto a tile that has the tip of a snake's tail as long as we know that snake hasn't just eaten
    // if tile is free: Food | Ally | Empty
    let board_tile = get_board_tile!(game_board, tile.x, tile.y);
    let snakes = &board.snakes;
    if board_tile_is_free!(board_tile)
        || (board_tile == types::Flags::SNAKE && can_move_on_tail!(snakes, tile))
    {
        // if tile is adjacent to head, only return true if we can't move anywhere else
        if adj_to_bigger_snake(tile, board, you) && avoid_snake_heads {
            return false;
        }
        return true;
    }
    return false;
}

/// # get_rand_moves
/// gets the most favourable moves, shuffling them if they are equally favourable
/// ## Arguments:
/// * from_point - the tile we want to move from
/// * board - the battlesnake game board
/// * game_board - the hashmap representation of the game board
/// * you - your battlesnake
/// * theshold - the connectedness theshold we want of a tile to be considered favourable
/// ## Returns:
/// an array of move options
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
            None
        );
    }
    let mut move_words: Vec<&str> = Vec::new();
    for mv in safe_moves {
        let dir_option = types::DIRECTIONS.into_iter().find_map(|(&key, &val)| {
            if val == (mv - *from_point) {
                Some(key)
            } else {
                None
            }
        });
        if dir_option.is_some() {
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

    // be less hungry, try to control the center if we have high health
    let mut path: Vec<types::Coord> = Vec::new();
    if _you.health < 75{
      path = graph::a_star(_board, &game_board, &_you, tile_connection_threshold);
    }
  
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
            get_adj_tiles_connected(&you.head, &board, &game_board, you, 0.8, Some(true), None, None);
        assert!(connected_tiles[0] == Coord { x: 4, y: 4 });
        connected_tiles =
            get_adj_tiles_connected(&you.head, &board, &game_board, you, 0.01, Some(true), None, None);
        assert!(
            connected_tiles.len() == 3
                && connected_tiles[connected_tiles.len() - 1] == Coord { x: 4, y: 4 }
        );
    }
}
