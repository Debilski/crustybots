
use pelita_rust_wrapper::*;

use pyo3::prelude::PyObject;

use rand::seq::SliceRandom;

const TEAM_NAME: &str = "Team rusty lantern";

fn movebot<T>(bot: Bot, state: T) -> Pos {
    // TODO: Our state should probably be a &mut Option<T> that starts out None

    // fetch our legal positions
    let legal_pos = bot.legal_positions;

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
