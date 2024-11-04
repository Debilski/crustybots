use debug_print::debug_println;
use pelita_rust_wrapper::*;

use pyo3::buffer::Element;
use pyo3::prelude::PyObject;

use std::collections::HashMap;
use std::{i32, u32};
use std::path::Display;
use std::time::SystemTime;

use im::HashSet;
use std::rc::Rc;
use std::sync::Arc;

use lru::LruCache;
use rand::seq::SliceRandom;
use std::cell::OnceCell;

use pathfinding::directed::astar::astar;
use pathfinding::grid::Grid;

const TEAM_NAME: &str = "Team rusty lantern";

#[derive(Clone, Debug)]
struct GameState {
    is_max_player: bool,  // True if it's the max player's turn, false if it's the min player's turn

    team_id: usize, // the playing team
    me_id: usize, // the playing bot

    bots: [Pos; 4],
    walls: HashSet<Pos>,
    food: [HashSet<Pos>; 2],
    shape: Shape,
    turn: usize,
    score: [usize; 2],
    round: usize
}

impl std::fmt::Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "GameState {{ is_max: {}, bots: {:?}, food: {:?}, turn: {}, round: {}, score: {:?} }}",
        self.is_max_player,
        self.bots,
        self.food,
        self.turn,
        self.round,
        self.score)
    }
}

impl GameState {
    fn from_bot(bot: &Bot) -> Self {
        if bot.is_blue {
            let bots = if bot.turn == 0 {
                [bot.position, bot.enemy[0].position, bot.other.position, bot.enemy[1].position]
            } else {
                [bot.other.position, bot.enemy[0].position, bot.position, bot.enemy[1].position]
            };
            let turn = if bot.turn == 0 { 0 } else { 2 };

        Self {
            is_max_player: true,
            team_id: 0,
            me_id: bot.turn * 2,
            bots,
            walls: bot.walls.clone().into(),
            food: [bot.food.clone().into(), bot.enemy[0].food.clone().into()],
            shape: bot.shape,
            turn,
            score: [bot.score, bot.enemy[0].score],
            round: bot.round
        }
    } else {
        let bots = if bot.turn == 0 {
            [bot.enemy[0].position, bot.position, bot.enemy[1].position, bot.other.position]
        } else {
            [bot.enemy[0].position, bot.other.position, bot.enemy[1].position, bot.position]
        };
        let turn = if bot.turn == 0 { 1 } else { 3 };

        Self {
            is_max_player: true,
            team_id: 1,
            me_id: bot.turn * 2 + 1,
            bots,
            walls: bot.walls.clone().into(),
            food: [bot.enemy[0].food.clone().into(), bot.food.clone().into()],
            shape: bot.shape,
            turn,
            score: [bot.enemy[0].score, bot.score],
            round: bot.round
        }
    }
    }

    fn new(
        is_max_player: bool,
        team_id: usize,
        me_id: usize,
        bots: [Pos; 4],
        walls: HashSet<Pos>,
        food: [HashSet<Pos>; 2],
        shape: Shape,
        turn: usize,
        score: [usize; 2],
        round: usize) -> Self {
        Self {
            is_max_player,
            team_id,
            me_id,
            bots,
            walls,
            food,
            shape,
            turn,
            score,
            round
        }
    }

    fn is_terminal(&self) -> bool {
        if self.round == 300 || self.food[0].is_empty() || self.food[1].is_empty() {
            println!("Terminal state: {}", self);
        }

        // Define the condition for a terminal state
        self.round == 300 || self.food[0].is_empty() || self.food[1].is_empty()
    }

    fn evaluate(&self) -> i32 {
        let others = if self.team_id == 0 { 1 } else { 0 };
        let mut score = self.score[self.team_id] as i32 - self.score[others] as i32;
        score *= 100;

        // println!("Evaluating to score {}: {}", score, self);
        if self.round == 300 || self.food[0].is_empty() || self.food[1].is_empty() {
            score *= 3000;
        }

        // if we are in our zone (attack mode): move closer towards the enemy
        let enemy_ids = if self.team_id == 0 {
            [1, 3]
        } else {
            [0, 2]
        };
        let our_ids = if self.team_id == 0 {
            [0, 2]
        } else {
            [1, 3]
        };

        for our in our_ids {
            for e in enemy_ids {
                if let Some(dist) = self.distance_c(&self.bots[our], &self.bots[e]) {
                    // println!("d {}", dist);
                    score -= dist as i32;
                }
            }
        }
        // println!("{}", score);
        // try to have the shortest path to the food avoid spots with an enemy
        for our in our_ids {
            score += (self.food[1 - self.team_id].iter().map(|f| self.distance_c(&self.bots[our], &f).unwrap_or(u32::MAX)).min().unwrap_or(u32::MAX) as i32);
        }

        // if we are in the enemy zone (eat mode): eat the

        score
    }


    fn get_neighbors_for(&self, pos: &Pos) -> Vec<Pos> {
        let mut allowed = Vec::new();

        let moves: [(i32, i32); 5] = [(0, 0), (0, 1), (1, 0), (0, -1), (-1, 0)];
        for mv in moves {
            let new_pos = ((pos.0 as i32 + mv.0) as usize, (pos.1 as i32 + mv.1) as usize);
            if ! self.walls.contains(&new_pos) {
                allowed.push(new_pos);
            }
        }
        allowed
    }

    fn get_neighbors(&self) -> Vec<Pos> {
        let pos = self.bots[self.turn];
        self.get_neighbors_for(&pos)
    }

    fn get_initial_pos(&self, idx: usize) -> Pos {
        let shape = self.shape;
        [(1, shape.1 - 3), (shape.0 - 2, 2), (1, shape.1 - 3), (shape.0 - 2, 3)][idx]
    }

    fn move_bot(&self, pos: Pos) -> GameState {
        // moves the bot with self.turn and returns a new Gamestate for the next bot

        let FOOD_POINTS = 1;
        let BOT_POINTS = 5;

        let mut score = self.score;
        let bot_id = self.turn;
        let team_id = self.turn % 2;
        let enemy_team_id = (1 - team_id) as usize;
        let mut bots = self.bots;

        // eat food if there is enemy food on the new spot
        let food = if team_id == 0 && self.food[1].contains(&pos) {
            // TODO: is clone ok here or too slow?
            score[0] += FOOD_POINTS;
            [self.food[0].clone(), self.food[1].without(&pos)]
        } else if team_id == 1 && self.food[0].contains(&pos) {
            score[1] += FOOD_POINTS;
            [self.food[0].without(&pos), self.food[1].clone()]
        } else {
            [self.food[0].clone(), self.food[1].clone()]
        };

        bots[self.turn] = pos;

        // enemy eating
        if team_id == 0 && pos.0 < self.shape.0 / 2 {
            // team 0 can eat
            for enemy_bot in [1, 3] {
                if bots[enemy_bot] == pos {
                    bots[enemy_bot] = self.get_initial_pos(enemy_bot);
                    score[0] += BOT_POINTS;
                }
            }
        } else if team_id == 1 && pos.0 >= self.shape.0 / 2 {
            // team 1 can eat
            for enemy_bot in [0, 2] {
                if bots[enemy_bot] == pos {
                    bots[enemy_bot] = self.get_initial_pos(enemy_bot);
                    score[1] += BOT_POINTS;
                }
            }
        }

        let mut next_turn = self.turn + 1;
        let mut next_round = self.round;
        if next_turn == 4 {
            next_turn = 0;
            next_round += 1;
        }

        GameState {
            team_id: self.team_id,
            me_id: self.me_id,
            bots,
            food,
            score,
            is_max_player: !self.is_max_player,
            walls: self.walls.clone(),
            shape: self.shape,
            turn: next_turn,
            round: next_round
        }
    }

    fn get_successors(&self) -> Vec<(Pos, GameState)> {
        // Generate successor states
        let mut successors = Vec::new();
        for &pos in &self.get_neighbors() {
            let succ = self.move_bot(pos);
            successors.push((pos, succ));
        }
        successors
    }

    fn get_neighbors_cost(&self, pos: &Pos) -> Vec<(Pos, u32)> {
        self.get_neighbors_for(pos).into_iter().map(|p| (p, 1)).collect()
    }

    fn distance(&self, start: &Pos, end: &Pos) -> Option<u32> {
        fn abs(a: &Pos, b: &Pos) -> u32 {
            ((a.0).abs_diff(b.0) + (a.1).abs_diff(b.1)) as u32
        }

        let path = astar(start, |&p| self.get_neighbors_cost(&p), |p| abs(p, end) , |p| p == end);
        path.map(|p| p.1)
    }

    fn distance_c(&self, start: &Pos, end: &Pos) -> Option<u32> {
        let n: (Pos, Pos) = (*start, *end);

        use std::num::NonZeroUsize;
        use std::sync::Mutex;

        lazy_static! {
            static ref LRU: Mutex<LruCache<(Pos, Pos), Option<u32>>> =
                Mutex::new(LruCache::new(NonZeroUsize::new(10000).unwrap()));
        }

        let mut cache = LRU.lock().unwrap();
        //debug_println!("Cache size: {}", &cache.len());
        let v = cache.get_or_insert(n, || self.distance(&n.0, &n.1));
        *v
    }

}
fn alpha_beta(state: &GameState, depth: i32, alpha: i32, beta: i32, count: &mut u32) -> i32 {
    if depth == 0 || state.is_terminal() {
        *count += 1;
        return state.evaluate();
    }

    let mut alpha = alpha;
    let mut beta = beta;

    if state.is_max_player {
        let mut value = i32::MIN;
        for (_pos, successor) in state.get_successors() {
            value = value.max(alpha_beta(&successor, depth - 1, alpha, beta, count));
            alpha = alpha.max(value);
            if value >= beta {
                break;
            }
        }
        value
    } else {
        let mut value = i32::MAX;
        for (_pos, successor) in state.get_successors() {
            value = value.min(alpha_beta(&successor, depth - 1, alpha, beta, count));
            beta = beta.min(value);
            if value <= alpha {
                break;
            }
        }
        value
    }
}

fn find_best_move(state: &GameState, depth: i32) -> (Pos, GameState) {
    let mut best_move = None;
    let mut best_value = if state.is_max_player { i32::MIN } else { i32::MAX };
    let mut count: u32 = 0;

    for (pos, successor) in state.get_successors() {
        let value = alpha_beta(&successor, depth - 1, i32::MIN, i32::MAX, &mut count);
        if (state.is_max_player && value > best_value) || (!state.is_max_player && value < best_value) {
            best_value = value;
            best_move = Some((pos, successor));
        }
    }

    println!("Found a best move with value {} after {} evaluations", best_value, count);

    best_move.expect("There should be at least one valid move")
}

fn main() {
    // let initial_state = GameState::new(5, true);  // Initial game state
    // let depth = 4;  // Depth of search
    // let best_move = find_best_move(&initial_state, depth);
    // println!("Best move: {:?}", best_move);
}


// fn movebot(bot: &Bot, state: &mut Option<PelitaGameState>) -> Pos {
//     // TODO: Our state should probably be a &mut Option<T> that starts out None

//     // TODO Algo: Check the areas where enemy bots cross the border
//     // estimate which border they will cross
//     // wait behind the area with two bots and attack

//     let start = SystemTime::now();

//     let state = state.get_or_insert(PelitaGameState::init(bot));
//     state.update(bot);

//     // println!("x{:?}", state);

//     // generate_states(state);

//     // let best_move = best_option(state, state);


//         let initial_state = GameState::new(FullGameState::Running(state.to_owned()), true, 0);  // Initial game state
//         let depth = 4;  // Depth of search
//         let best_move = find_best_move(&initial_state, depth);
//         println!("Best move: {:?}", best_move);


//     //println!("Best option {:?}", best_move);

//     let end = SystemTime::now();
//     let duration = end.duration_since(start).unwrap();
//     println!("it took {} microseconds", duration.as_micros());

//     (0, 0)
// }

fn mymove(bot: &Bot, state: &mut Option<i32>) -> Pos {
    let gs = GameState::from_bot(bot);
    println!("{}", gs);
    let depth = 9;  // Depth of search
    let best_move = find_best_move(&gs, depth);
    println!("Best move: {:?} {}", best_move.0, best_move.1);
    best_move.0
}

// Register player (package name, team name, move fn)
// NB: The package name needs to be the same as in pyproject.toml
pelita_player!(crustybots, TEAM_NAME, mymove, i32);


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_succ() {

        let layout_str = "
        ################
        #. #.     .. .y#
        # .  #.## ####x#
        # .#      #..  #
        #  ..#     a#. #
        # #### ##.#  . #
        #b. ..     .# .#
        ################
        ";
        let layout = pelita_rust_wrapper::parse_layout(layout_str).expect("TEST");
        // println!("{:?}", layout);

        let shape = layout.shape;
        let mut split_food = [HashSet::new(), HashSet::new()];
        for food in layout.food {
            if food.0 < shape.0 / 2 {
                split_food[0] = split_food[0].update(food)
            } else {
                split_food[1] = split_food[1].update(food)
            }
        }

        let gs = GameState::new(true, 0, 0, layout.bots, layout.walls.into(), split_food, shape, 0, [0, 0], 0);
        println!("{:?}", gs.get_neighbors());
        println!("{:?}", gs);

        assert_eq!(gs.distance(&gs.bots[1], &gs.bots[3]), Some(1));
        assert_eq!(gs.distance(&gs.bots[0], &gs.bots[2]), Some(12));

        let depth = 4;  // Depth of search
        let best_move = find_best_move(&gs, depth);
        println!("Best move: {:?} {}", best_move.0, best_move.1);

        let new_gs = gs.move_bot((11, 3));
        println!("{}", new_gs);
        let new_gs = new_gs.move_bot((14, 3));
        println!("{}", new_gs);
    }


    #[test]
    fn test_kill() {

        let layout_str = "
            ################
            #. #. .   . . y#
            #..# #     .## #
            # b    ##   #..#
            #..#x# ##      #
            # ##      # #..#
            #  .a.   . .# .#
            ################
        ";
        let layout = pelita_rust_wrapper::parse_layout(layout_str).expect("TEST");
        // println!("{:?}", layout);

        let shape = layout.shape;
        let mut split_food = [HashSet::new(), HashSet::new()];
        for food in layout.food {
            if food.0 < shape.0 / 2 {
                split_food[0] = split_food[0].update(food)
            } else {
                split_food[1] = split_food[1].update(food)
            }
        }

        let gs = GameState::new(true, 0, 0, layout.bots, layout.walls.into(), split_food, shape, 0, [0, 0], 0);
        println!("{:?}", gs.get_neighbors());
        println!("{:?}", gs);

        let depth = 17;  // Depth of search
        let best_move = find_best_move(&gs, depth);
        println!("Best kill move: {:?}", best_move);
    }


    #[test]
    fn it_works() {
        // let res= pelita_rust_wrapper::run_game("crustybots");
        // println!("{:?}", res.unwrap());
    }
}
