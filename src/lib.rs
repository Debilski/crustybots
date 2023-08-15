
use pelita_rust_wrapper::*;

use pyo3::prelude::PyObject;

use rand::seq::SliceRandom;

use pathfinding::grid::Grid;

const TEAM_NAME: &str = "Team rusty lantern";

fn create_grid(bot: &Bot) -> Grid {
    let (w, h) = bot.shape;
    let mut g = Grid::new(w, h);
    for &elem in bot.walls.iter() {
        g.add_vertex(elem);
    }
    g.invert();
    g
}

fn movebot<T>(bot: &Bot, state: T) -> Pos {
    // TODO: Our state should probably be a &mut Option<T> that starts out None

    let grid = create_grid(&bot);
    print!("{:?}", grid);

    let food_distance: Vec<_> = bot.enemy[0].food.iter().map(|&pos| grid.distance(bot.position, pos)).collect();
    bot.say("Hello");

    print!("{:?}", food_distance);

    // fetch our legal positions
    let legal_pos = &bot.legal_positions;

    // choose a random one
    let dice = legal_pos.choose(&mut rand::thread_rng());

    // unwrap the option (we are sure that there is at least one move possible)
    let pos: Pos = *dice.unwrap();

    // return
    pos
}


// Register player (package name, team name, move fn)
// NB: The package name needs to be the same as in pyproject.toml
pelita_player!(crustybots, TEAM_NAME, movebot);
