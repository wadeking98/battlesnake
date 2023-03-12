use crate::{logic, types};
use std::collections::{HashSet, VecDeque};

struct Node<'a>{
    item: &'a types::Coord,
    parent: Option<&'a Self>
}

pub fn breadth_first_search(
    game_board: &mut Vec<Vec<types::Flags>>,
    current_tile: &types::Coord,
    snakes: &Vec<types::Battlesnake>,
    you: &types::Battlesnake,
    parent_node: &Node,
    frontier: &mut VecDeque<Node>,
) -> Vec<types::Coord> {
    let adj_tiles = logic::get_adj_tiles(&game_board, current_tile, snakes, you);
    let adj_nodes: Vec<Node> = adj_tiles.iter().map(|tile| Node{item:tile, parent:Some(parent_node)}).collect();
    println!("{:?}", current_tile);

    // base case
    for adj in &adj_nodes {
        adj.parent = Some(parent_node);
        if game_board[adj.item.x as usize][adj.item.y as usize] == types::Flags::FOOD {
            // println!("HERE");

            return vec![*adj.item];
        } else {
            //mark as visited
            game_board[adj.item.x as usize][adj.item.y as usize] = types::Flags::HAZARD;
        }
    }

    // iterate over the frontier
    let mut adj_tiles_deque = VecDeque::from(adj_tiles);
    frontier.append(&mut adj_tiles_deque);

    while frontier.len() > 0 {
        let tile = frontier.pop_front().unwrap();
        let mut path = breadth_first_search(game_board, &tile, snakes, you, frontier);
        if path.len() > 0 {
            path.insert(0, tile);
            return path;
        }
    }
    return vec![] as Vec<types::Coord>;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types;

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
        let path = breadth_first_search(
            &mut game_board,
            &you.head,
            &board.snakes,
            &you,
            &mut VecDeque::new(),
        );
        assert!(path[path.len()-1] == types::Coord{x:8, y:4});
        
    }
}
