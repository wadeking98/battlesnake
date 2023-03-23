use crate::logic::{get_adj_tiles, get_all_adj_tiles};
use crate::{get_board_tile, logic, types};
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;
use std::cmp;
use std::collections::{HashMap, HashSet, VecDeque};

/// # dfs_long
/// finds a long path to a specified coordinate. uses hueristic distance to approximate longest path
/// ## Arguments
/// * goal - the goal to search for
/// * board - the game board object
/// * game_board - the hash table representation of the game board (used for faster lookup)
/// * you - our battlesnake
/// * connection_threshold - the connectedness threshold we want tiles in the path to adhere to
/// * degree_threshold - the minimum number of adjacent tiles that a given tile must have to be considered valid
/// ## Returns:
/// a path from our starting point to the goal
pub fn dfs_long(
    goal: &types::Coord,
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    connection_threshold: f32,
    degree_threshold: u8
) -> Vec<types::Coord> {
    let mut visited: HashMap<types::Coord, types::Coord> = HashMap::new();
    let success = depth_first_search_logic(
        goal,
        &you.head,
        board,
        game_board,
        you,
        &mut visited,
        connection_threshold,
        degree_threshold
    );
    return match success {
        Some(tile) => backtrack(tile, &visited),
        None => vec![],
    };
}

/// # depth_first_search_logic
/// Approximates the longest path to a specified coord using a priority queue
/// ## Arguments
/// * goal - the goal tile to search for
/// * board - the game board object
/// * game_board - the hash table representation of the game board (used for faster lookup)
/// * you - our battlesnake
/// * frontier - keeps track of the tiles we haven't visited yet in our search
/// * visited - keeps track of the tiles we've already visited during our search and their parent nodes (values are the parent coords)
/// * connection_threshold - the connectedness threshold we want tiles in the path to adhere to
/// * degree_threshold - the minimum number of adjacent tiles that a given tile must have to be considered valid
/// ## Returns:
/// an option of a tile containing a food if a path is successfully found
fn depth_first_search_logic(
    goal: &types::Coord,
    from: &types::Coord,
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    visited: &mut HashMap<types::Coord, types::Coord>,
    connection_threshold: f32,
    degree_threshold: u8,
) -> Option<types::Coord> {
    if from.distance(goal) <= 1.0 {
        visited.insert(*goal, *from);
        return Some(*goal);
    }

    // get current path so we make sure we don't intersect our own path
    let current_path = backtrack(*from, visited);
    let path_index =
        usize::try_from(cmp::max(0, current_path.len() as i32 - you.length as i32)).unwrap_or(0);
    let future_snake_positions: Vec<types::Coord> = current_path[path_index..].to_vec();

    // get adj tiles if they haven't been visited before and they're not in the current path
    let mut adj_tiles: Vec<types::Coord> = logic::get_adj_tiles_connected(
        from,
        board,
        &game_board,
        you,
        0.0,
        0,
        None,
        Some(future_snake_positions),
    )
    .into_iter()
    .filter(|item| visited.get(item).is_none())
    .collect();

    adj_tiles.sort_by(|a, b| goal.distance(b).partial_cmp(&goal.distance(a)).unwrap());

    // mark adj tiles as visited and link the parent node
    for tile in &adj_tiles {
        visited.insert(*tile, *from);
        let success = depth_first_search_logic(
            goal,
            tile,
            board,
            game_board,
            you,
            visited,
            connection_threshold,
            degree_threshold
        );
        if success.is_some() {
            return success;
        }
    }

    // search failed so backtrack
    return None;
}

pub fn inside_box(
    you: &types::Battlesnake,
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    box_threshold: f32,
) -> bool {
    let mut frontier: VecDeque<types::Coord> = VecDeque::from([you.head]);
    let mut visited: HashSet<types::Coord> = HashSet::new();
    let num_free_tiles = logic::num_free_tiles(board);
    return inside_box_logic(
        you,
        board,
        game_board,
        &mut frontier,
        &mut visited,
        num_free_tiles,
        box_threshold,
    );
}

fn inside_box_logic(
    you: &types::Battlesnake,
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    frontier: &mut VecDeque<types::Coord>,
    visited: &mut HashSet<types::Coord>,
    num_free_tiles: u16,
    box_threshold: f32,
) -> bool {
    if frontier.is_empty() {
        return true;
    }

    let current_tile = frontier.pop_front().unwrap();

    let adj_tiles: Vec<types::Coord> = get_adj_tiles(&current_tile, board, game_board, you, None, None)
        .into_iter()
        .filter(|item| visited.get(item).is_none())
        .collect();

    for adj in &adj_tiles {
        visited.insert(*adj);
    }

    if (visited.len() as f32 / num_free_tiles as f32) > box_threshold {
        return false;
    }

    frontier.append(&mut VecDeque::from(adj_tiles));

    return inside_box_logic(
        you,
        board,
        game_board,
        frontier,
        visited,
        num_free_tiles,
        box_threshold,
    );
}

fn find_blocking_tiles(
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    frontier: &mut VecDeque<types::Coord>,
    visited: &mut HashSet<types::Coord>,
    blocking_tiles: &mut Vec<types::Coord>,
) {
    if frontier.is_empty() {
        return;
    }

    let current_tile = frontier.pop_front().unwrap();

    if !(get_board_tile!(game_board, current_tile.x, current_tile.y) & types::Flags::SNAKE)
        .is_empty()
    {
        blocking_tiles.push(current_tile);
    } else {
        let adj_tiles: Vec<types::Coord> = get_all_adj_tiles(&current_tile, board)
            .into_iter()
            .filter(|item| visited.get(item).is_none())
            .collect();
        for adj in &adj_tiles {
            visited.insert(*adj);
        }
        let mut adj_tiles_deque = VecDeque::from(adj_tiles);
        frontier.append(&mut adj_tiles_deque);
    }
    find_blocking_tiles(board, game_board, frontier, visited, blocking_tiles);
}

/// # find_key_hole
/// given that the snake it trapped in a small region, find the tile that is our best bet to leave the region
pub fn find_key_hole(
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
) -> Option<types::Coord> {
    let mut frontier: VecDeque<types::Coord> =
        VecDeque::from(get_adj_tiles(&you.head, board, game_board, you, None, None));
    let mut visited: HashSet<types::Coord> = HashSet::new();
    let mut blocking_tiles: Vec<types::Coord> = Vec::new();
    find_blocking_tiles(
        board,
        game_board,
        &mut frontier,
        &mut visited,
        &mut blocking_tiles,
    );

    blocking_tiles.sort_by(|a, b| {
        let index_a;
        let index_b;
        match logic::get_snake_from_tile(a, &board.snakes) {
            Some(snake) => {
                index_a =
                    snake.body.len() - snake.body.iter().position(|item| item == a).unwrap_or(0)
            }
            None => index_a = 0,
        }
        match logic::get_snake_from_tile(b, &board.snakes) {
            Some(snake) => {
                index_b =
                    snake.body.len() - snake.body.iter().position(|item| item == b).unwrap_or(0)
            }
            None => index_b = 0,
        }

        return index_a.cmp(&index_b);
    });

    if blocking_tiles.len() <= 0 {
        return None;
    }
    // find the blocking tile that is closest to the tail of it's snake
    return Some(blocking_tiles[0]);
}

/// # backtrack
/// determines the path from the starting point to our goal
/// ## Arguments:
/// * tile - the goal tile
/// * trace_tree - hashmap containing tiles as keys and thier parents as values
/// ## Returns:
/// a path from our starting point to the goal
fn backtrack(
    tile: types::Coord,
    trace_tree: &HashMap<types::Coord, types::Coord>,
) -> Vec<types::Coord> {
    let mut current_tile = &tile;
    let mut path = vec![*current_tile];
    loop {
        let parent_opt = trace_tree.get(current_tile);
        match parent_opt {
            Some(tile) => {
                path.push(*tile);
                current_tile = tile;
            }
            None => break,
        }
    }

    // return early if the path is empty
    if path.len() <= 0 {
        return path;
    }

    // remove the root node, usually the head of the snake
    let split_slice = path.split_last().unwrap().1;
    let mut cleaned_path = Vec::from(split_slice);
    cleaned_path.reverse();

    return cleaned_path;
}

fn closest_food(tile: &types::Coord, board: &types::Board) -> Option<f32> {
    if board.food.len() <= 0 {
        return None;
    }
    let mut distances: Vec<f32> = board.food.iter().map(|item| tile.distance(item)).collect();
    distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
    return Some(distances[0]);
}

/// # a_star
/// determines the shortest path to a food
/// ## Arguments:
/// * board - battlesnake game board
/// * game_board - hashmap representation of the board
/// * you - your battlesnake
/// * connection_threshold - only go to goal if it passes this connection threshold
/// * degree_threshold - the minimum number of adjacent tiles that a given tile must have to be considered valid
/// ## Returns:
/// The shortest path to the goal tile
pub fn a_star(
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    connection_threshold: f32,
    degree_threshold: u8
) -> Vec<types::Coord> {
    let mut frontier: PriorityQueue<types::Coord, OrderedFloat<f32>> = PriorityQueue::new();
    frontier.push(you.head, OrderedFloat(0.0));
    let mut visited: HashMap<types::Coord, types::Coord> = HashMap::new();
    let mut cost_so_far: HashMap<types::Coord, u16> = HashMap::new();
    let path_found = a_star_logic(
        board,
        game_board,
        you,
        &mut frontier,
        &mut visited,
        &mut cost_so_far,
        connection_threshold,
        degree_threshold
    );

    return match path_found {
        Some(goal) => backtrack(goal, &visited),
        None => vec![],
    };
}

/// # a_star_logic
/// determines the shortest path to a food or specified tile
/// ## Arguments:
/// * goal_tile_option - option to find path to tile instead of food
/// * board - battlesnake game board
/// * game_board - hashmap representation of the board
/// * you - your battlesnake
/// * frontier - used to investigate new tiles
/// * visited - used to mark tiles we've already visited
/// * cost_so_far - used to remember the current cost of the path
/// * exclude_tiles - mark specified tiles as blocked, for example the starting tile if it's not a snake body
/// * connection_threshold - only go to goal if it passes this connection threshold
/// * degree_threshold - the minimum number of adjacent tiles that a given tile must have to be considered valid
/// ## Returns:
/// The goal tile if a path is found
fn a_star_logic(
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    frontier: &mut PriorityQueue<types::Coord, OrderedFloat<f32>>,
    visited: &mut HashMap<types::Coord, types::Coord>,
    cost_so_far: &mut HashMap<types::Coord, u16>,
    connection_threshold: f32,
    degree_threshold: u8,
) -> Option<types::Coord> {
    if frontier.is_empty() {
        return None;
    }

    let (current_tile, _) = frontier.pop().unwrap();

    // if we've found a food that we can get to with our current health
    if !(get_board_tile!(game_board, current_tile.x, current_tile.y) & types::Flags::FOOD)
        .is_empty()
        && cost_so_far.get(&current_tile).unwrap_or(&0) < &(you.health as u16)
    {
        return Some(current_tile);
    }

    // get current path so we make sure we don't intersect our own path
    let current_path = backtrack(current_tile, visited);
    let path_index =
        usize::try_from(cmp::max(0, current_path.len() as i32 - you.length as i32)).unwrap_or(0);
    let future_snake_positions: Vec<types::Coord> = current_path[path_index..].to_vec();

    // get adj tiles if they haven't been visited before and they're not in the current path
    let adj_tiles: Vec<types::Coord> = logic::get_adj_tiles_connected(
        &current_tile,
        board,
        &game_board,
        you,
        connection_threshold,
        degree_threshold,
        None,
        Some(future_snake_positions),
    );

    let current_cost = *cost_so_far.get(&current_tile).unwrap_or(&0);
    // mark adj tiles as visited and link the parent node
    for tile in &adj_tiles {
        let mut movement_cost: u8 = 1;
        if !(get_board_tile!(game_board, tile.x, tile.y) & types::Flags::HAZARD).is_empty() {
            movement_cost = 16;
        }
        let previous_cost_opt = cost_so_far.get(&tile);
        let new_cost = current_cost + movement_cost as u16;
        if previous_cost_opt.is_none() || *previous_cost_opt.unwrap() > new_cost {
            cost_so_far.insert(*tile, new_cost);
            let heuristic_distance = closest_food(tile, board).unwrap_or(0.0);
            let priority = new_cost as f32 + heuristic_distance;
            // here we take the negative priority so closest points are at the top
            frontier.push(*tile, OrderedFloat(-priority));
            visited.insert(*tile, current_tile);
        }
    }

    return a_star_logic(
        board,
        game_board,
        you,
        frontier,
        visited,
        cost_so_far,
        connection_threshold,
        degree_threshold
    );
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types;

    #[test]
    fn test_get_head_adj() {
        static BOARD_DATA: &str = r#"{
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
        let board: types::Board = serde_json::from_str(BOARD_DATA).unwrap();
        let you: types::Battlesnake = board.snakes[0].clone();
        let game_board = board.to_game_board();
        let adj = logic::get_adj_tiles(&you.head, &board, &game_board, &you, None, None);
        assert!(
            adj.contains(&(you.head + types::DIRECTIONS["left"]))
                && adj.contains(&(you.head + types::DIRECTIONS["right"]))
                && adj.len() == 2
        );
    }

    #[test]
    fn shortest_to_food() {
        const FOOD_DATA: &str = r#"
        {
            "food": [
              {
                "x": 8,
                "y": 4
              },
              {
                "x": 0,
                "y": 10
              }
            ],
            "snakes": [
              {
                "id": "jt-0Z",
                "name": "snake jt-0Z",
                "health": 100,
                "body": [
                  {
                    "x": 4,
                    "y": 4
                  },
                  {
                    "x": 4,
                    "y": 3
                  },
                  {
                    "x": 4,
                    "y": 2
                  },
                  {
                    "x": 4,
                    "y": 1
                  }
                ],
                "latency": 0,
                "head": {
                  "x": 4,
                  "y": 4
                },
                "length": 4,
                "shout": "",
                "squad": ""
              }
            ],
            "width": 11,
            "height": 11,
            "hazards": [
              {
                "x": 8,
                "y": 4
              }
            ]
          }
        "#;
        let board: types::Board = serde_json::from_str(FOOD_DATA).unwrap();
        let mut you = board.snakes[0].clone();
        let game_board = board.to_game_board();

        let a_star_path = a_star(&board, &game_board, &you, 0.5, 0);
        assert!(
            a_star_path.len() > 0
                && a_star_path[a_star_path.len() - 1] == types::Coord { x: 0, y: 10 }
        );
        you.health = 3;
        let a_star_path_low = a_star(&board, &game_board, &you, 0.5, 0);
        assert!(a_star_path_low.len() <= 0);
    }
    #[test]
    fn avoid_future_poorly_connected_tiles() {
        const BOARD_DATA: &str = r#"
      {
        "food": [
          {
            "x": 2,
            "y": 2
          }
        ],
        "snakes": [
          {
            "id": "5h9p6",
            "name": "snake 5h9p6",
            "health": 100,
            "body": [
              {
                "x": 4,
                "y": 2
              },
              {
                "x": 4,
                "y": 3
              },
              {
                "x": 4,
                "y": 4
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
                "y": 3
              },
              {
                "x": 1,
                "y": 3
              },
              {
                "x": 1,
                "y": 2
              },
              {
                "x": 1,
                "y": 1
              },
              {
                "x": 2,
                "y": 1
              },
              {
                "x": 2,
                "y": 0
              }
            ],
            "latency": 0,
            "head": {
              "x": 4,
              "y": 2
            },
            "length": 11,
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
        let you = &board.snakes[0];
        let game_board = board.to_game_board();

        let a_star_path = a_star(&board, &game_board, you, 0.5, 0);
        // a valid path cannot exist here because approaching the tile disconnects it from the rest of the board
        assert!(a_star_path.len() <= 0);
    }

    #[test]
    fn escape_from_box() {
        const BOARD_DATA: &str = r#"
      {
        "food": [],
        "snakes": [
          {
            "id": "PJs7i",
            "name": "snake PJs7i",
            "health": 99,
            "body": [
              {
                "x": 5,
                "y": 8
              },
              {
                "x": 5,
                "y": 7
              },
              {
                "x": 5,
                "y": 6
              },
              {
                "x": 5,
                "y": 5
              },
              {
                "x": 5,
                "y": 4
              },
              {
                "x": 4,
                "y": 4
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
                "x": 2,
                "y": 8
              },
              {
                "x": 2,
                "y": 9
              },
              {
                "x": 2,
                "y": 10
              }
            ],
            "latency": 0,
            "head": {
              "x": 5,
              "y": 8
            },
            "length": 14,
            "shout": "",
            "squad": ""
          },
          {
            "id": "uR2vE",
            "name": "snake uR2vE",
            "health": 99,
            "body": [
              {
                "x": 1,
                "y": 6
              },
              {
                "x": 1,
                "y": 5
              },
              {
                "x": 1,
                "y": 4
              },
              {
                "x": 0,
                "y": 4
              },
              {
                "x": 0,
                "y": 5
              },
              {
                "x": 0,
                "y": 6
              },
              {
                "x": 0,
                "y": 7
              },
              {
                "x": 0,
                "y": 8
              },
              {
                "x": 0,
                "y": 9
              },
              {
                "x": 0,
                "y": 10
              }
            ],
            "latency": 0,
            "head": {
              "x": 1,
              "y": 6
            },
            "length": 10,
            "shout": "",
            "squad": ""
          },
          {
            "id": "ls7Zd",
            "name": "snake ls7Zd",
            "health": 99,
            "body": [
              {
                "x": 5,
                "y": 0
              },
              {
                "x": 6,
                "y": 0
              },
              {
                "x": 6,
                "y": 1
              },
              {
                "x": 6,
                "y": 2
              },
              {
                "x": 6,
                "y": 3
              },
              {
                "x": 6,
                "y": 4
              },
              {
                "x": 6,
                "y": 5
              },
              {
                "x": 6,
                "y": 6
              },
              {
                "x": 6,
                "y": 7
              },
              {
                "x": 6,
                "y": 8
              }
            ],
            "latency": 0,
            "head": {
              "x": 5,
              "y": 0
            },
            "length": 10,
            "shout": "",
            "squad": ""
          }
        ],
        "width": 11,
        "height": 11,
        "hazards": []
      }
      "#;

        const YOU_DATA: &str = r#"{
        "id": "ls7Zd",
        "name": "snake ls7Zd",
        "health": 99,
        "body": [
          {
            "x": 5,
            "y": 0
          },
          {
            "x": 6,
            "y": 0
          },
          {
            "x": 6,
            "y": 1
          },
          {
            "x": 6,
            "y": 2
          },
          {
            "x": 6,
            "y": 3
          },
          {
            "x": 6,
            "y": 4
          },
          {
            "x": 6,
            "y": 5
          },
          {
            "x": 6,
            "y": 6
          },
          {
            "x": 6,
            "y": 7
          },
          {
            "x": 6,
            "y": 8
          }
        ],
        "latency": 0,
        "head": {
          "x": 5,
          "y": 0
        },
        "length": 10,
        "shout": "",
        "squad": ""
      }"#;
        let board: types::Board = serde_json::from_str(BOARD_DATA).unwrap();
        let game_board = board.to_game_board();
        let you: types::Battlesnake = serde_json::from_str(YOU_DATA).unwrap();
        assert_eq!(
            find_key_hole(&board, &game_board, &you),
            Some(types::Coord { x: 6, y: 3 })
        );
        assert!(inside_box(&you, &board, &game_board, 0.3));
        let long_path = dfs_long(&types::Coord { x: 6, y: 3 }, &board, &game_board, &you, 0.0, 0);
        assert_eq!(*long_path.last().unwrap(), types::Coord { x: 6, y: 3 });
    }
}
