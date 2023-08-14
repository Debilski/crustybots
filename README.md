# Write a pelita bot with rust (demo code)

This demo repository uses the rust crate at https://github.com/Debilski/pelita-rust-wrapper which converts parts of the Python Bot API to Rust. (Unfortunately, we cannot deal with the JSON directly for now.)
It comes with a macro `pelita_player!` to set up the Python package for us (we need a `move` function and a `TEAM_NAME`). The Python package is then build using mathurin.

## How does the code look

```rust
// Rust structs for the Bot and pelita_player macro
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
```


## How to get it running

```fish
# clone this repo
git clone https://github.com/Debilski/crustybots
cd crustybots

# create and activate virtual environment
python3 -m venv venv
. ./venv/bin/activate.fish

# install build dependencies and pelita
pip install maturin pelita

# install this package
pip install .

# Run a game of pelita
pelita crustybots crustybots
```

## Further possibilities

It is already possible to import the maze into ndarrays:

```rust
fn make_array(shape: Shape, walls: HashSet<Pos>) -> Maze {
    Array::from_shape_fn(shape, |(i, j)| {
        walls.contains(&(i, j))
    })
}

println!("{:?}", make_array(bot.shape, bot.walls));
```