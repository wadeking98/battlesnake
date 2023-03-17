use crate::{get_board_tile, logic, types};
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;
use std::collections::{HashMap, VecDeque};

/// # breadth_first_search_logic
/// Finds a path to a food tile using BFS
/// ## Arguments
/// * board - the game board object
/// * game_board - the hash table representation of the game board (used for faster lookup)
/// * you - our battlesnake
/// * connection_threshold - the connectedness threshold we want tiles in the path to adhere to
/// ## Returns:
/// a path from our starting point to the goal
pub fn bfs(
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    connection_threshold: f32,
) -> Vec<types::Coord> {
    let mut frontier: VecDeque<types::Coord> = VecDeque::new();
    frontier.push_back(you.head);
    let mut visited: HashMap<types::Coord, types::Coord> = HashMap::new();
    let goal_opt = breadth_first_search_logic(
        board,
        game_board,
        you,
        &mut frontier,
        &mut visited,
        connection_threshold,
    );
    return match goal_opt {
        Some(goal) => backtrack(goal, &visited),
        None => vec![],
    };
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

pub fn a_star(
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    connection_threshold: f32,
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
    );

    return match path_found {
        Some(goal) => backtrack(goal, &visited),
        None => vec![],
    };
}

fn a_star_logic(
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    frontier: &mut PriorityQueue<types::Coord, OrderedFloat<f32>>,
    visited: &mut HashMap<types::Coord, types::Coord>,
    cost_so_far: &mut HashMap<types::Coord, u16>,
    connection_threshold: f32,
) -> Option<types::Coord> {
    if frontier.is_empty() {
        return None;
    }

    let (current_tile, _) = frontier.pop().unwrap();

    if !(get_board_tile!(game_board, current_tile.x, current_tile.y) & types::Flags::FOOD).is_empty() {
        return Some(current_tile);
    }

    // get adj tiles if they haven't been visited before and they're not in the current path
    let adj_tiles: Vec<types::Coord> = logic::get_adj_tiles_connected(
        &current_tile,
        board,
        &game_board,
        you,
        connection_threshold,
        Some(true),
        None,
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
            let priority = new_cost as f32 + closest_food(tile, board)?;

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
    );
}

/// # breadth_first_search_logic
/// Finds a path to a food tile using BFS
/// ## Arguments
/// * board - the game board object
/// * game_board - the hash table representation of the game board (used for faster lookup)
/// * you - our battlesnake
/// * frontier - keeps track of the tiles we haven't visited yet in our search
/// * visited - keeps track of the tiles we've already visited during our search and their parent nodes (values are the parent coords)
/// * connection_threshold - the connectedness threshold we want tiles in the path to adhere to
/// ## Returns:
/// an option of a tile containing a food if a path is successfully found
fn breadth_first_search_logic(
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    frontier: &mut VecDeque<types::Coord>,
    visited: &mut HashMap<types::Coord, types::Coord>,
    connection_threshold: f32,
) -> Option<types::Coord> {
    if frontier.len() <= 0 {
        return None;
    }

    let current_tile = frontier.pop_front().unwrap();

    if !(get_board_tile!(game_board, current_tile.x, current_tile.y) & types::Flags::FOOD).is_empty() {
        return Some(current_tile);
    }

    // get adj tiles if they haven't been visited before and they're not in the current path
    let adj_tiles: Vec<types::Coord> = logic::get_adj_tiles_connected(
        &current_tile,
        board,
        &game_board,
        you,
        connection_threshold,
        Some(true),
        None,
    )
    .into_iter()
    .filter(|tile| visited.get(tile).is_none())
    .collect();

    // mark adj tiles as visited and link the parent node
    for tile in &adj_tiles {
        visited.insert(*tile, current_tile);
    }

    // iterate over the frontier
    let mut adj_tiles_deque = VecDeque::from(adj_tiles);
    frontier.append(&mut adj_tiles_deque);

    // recursion step
    return breadth_first_search_logic(
        board,
        game_board,
        you,
        frontier,
        visited,
        connection_threshold,
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
        let adj = logic::get_adj_tiles(&you.head, &board, &game_board, &you, None);
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
                "health": 4,
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
        let you = board.snakes[0].clone();
        let game_board = board.to_game_board();

        let path = bfs(&board, &game_board, &you, 0.5);
        assert!(path.len() > 0 && path[path.len() - 1] == types::Coord { x: 8, y: 4 });

        let a_star_path = a_star(&board, &game_board, &you, 0.5);
        assert!(a_star_path.len() > 0 && a_star_path[a_star_path.len() - 1] == types::Coord { x: 0, y: 10 });
    }
}
