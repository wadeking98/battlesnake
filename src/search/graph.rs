use crate::{get_board_tile, logic, types};
use std::collections::{HashMap, VecDeque};

fn get_adj_tiles(
    tile: &types::Coord,
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
) -> Vec<types::Coord> {
    let mut adj: Vec<types::Coord> = vec![];
    for (.., dir) in types::DIRECTIONS.into_iter() {
        let new_point = *dir + *tile;
        if logic::can_move_board(&new_point, board, game_board, you, None) {
            adj.push(new_point)
        }
    }
    return adj;
}

pub fn bfs(
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    food_connected_option: Option<u8>,
) -> Vec<types::Coord> {
    let food_connected = food_connected_option.unwrap_or(1);
    let mut frontier: VecDeque<types::Coord> = VecDeque::new();
    frontier.push_back(you.head);
    let mut visited: HashMap<types::Coord, types::Coord> = HashMap::new();
    let mut current_path:Vec<types::Coord> = Vec::new();
    let path_res = breadth_first_search_logic(
        board,
        game_board,
        you,
        &mut current_path,
        &mut frontier,
        &mut visited,
        food_connected,
    );
    return path_res;
}

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
    if path.len() <= 0{
        return path;
    }

    // remove the root node, usually the head of the snake
    let split_slice = path.split_last().unwrap().1;
    let mut cleaned_path = Vec::from(split_slice);
    cleaned_path.reverse();

    return cleaned_path;
}

fn breadth_first_search_logic(
    board: &types::Board,
    game_board: &HashMap<types::Coord, types::Flags>,
    you: &types::Battlesnake,
    current_path: &mut Vec<types::Coord>,
    frontier: &mut VecDeque<types::Coord>,
    visited: &mut HashMap<types::Coord, types::Coord>,
    mut food_connected: u8,
) -> Vec<types::Coord> {
    if frontier.len() <= 0 {
        return vec![];
    }

    let current_tile = frontier.pop_front().unwrap();

    if get_board_tile!(game_board, current_tile.x, current_tile.y) == types::Flags::FOOD && !current_path.contains(&current_tile) {
        if food_connected <= 1 {
            let mut best_path = backtrack(current_tile, &visited);
            current_path.append(&mut best_path);
            return current_path.clone();
        } else {
            let mut path = backtrack(current_tile, &visited);
            current_path.append(&mut path);
            visited.clear();
            frontier.clear();
            frontier.push_back(current_path[current_path.len()-1]);
            food_connected -= 1;
        }
    }

    // get adj tiles if they haven't been visited before and they're not in the current path
    let adj_tiles: Vec<types::Coord> = get_adj_tiles(&current_tile, board, &game_board, you)
        .into_iter()
        .filter(|tile| visited.get(tile).is_none() && !current_path.contains(tile))
        .collect();

    // mark adj tiles as visited and link the parent node
    for tile in &adj_tiles {
        visited.insert(*tile, current_tile);
    }

    // iterate over the frontier
    let mut adj_tiles_deque = VecDeque::from(adj_tiles);
    frontier.append(&mut adj_tiles_deque);

    // recursion step
    let best_path =
        breadth_first_search_logic(board, game_board, you, current_path, frontier, visited, food_connected);

    // if we can't find a food that is connected to n other foods, return a food that is connected to n-1 foods
    if best_path.len() > 0 {
        return best_path;
    } else {
        return current_path.clone();
    }
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
        let adj = get_adj_tiles(&you.head, &board, &game_board, &you);
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
                "x": 4,
                "y": 10
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
            "hazards": []
          }
        "#;
        let board: types::Board = serde_json::from_str(FOOD_DATA).unwrap();
        let you = board.snakes[0].clone();
        let mut game_board = board.to_game_board();

        let path = bfs(&board, &mut game_board, &you, None);
        assert!(path.len() > 0 && path[path.len() - 1] == types::Coord { x: 8, y: 4 });

        let path_connected = bfs(&board, &game_board, &you, Some(2));
        assert!(path_connected.len() > 0 && path_connected[path_connected.len() - 1] == types::Coord { x: 4, y: 10 });
    }
}
