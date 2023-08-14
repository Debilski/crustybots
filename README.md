# Write a pelita bot with rust (demo code)

This demo repository uses the rust crate at https://github.com/Debilski/pelita-rust-wrapper which converts parts of the Python Bot API to Rust. (Unfortunately, we cannot deal with the JSON directly for now.)
It comes with a macro `pelita_player!` to set up the Python package for us (we need a `move` function and a `TEAM_NAME`). The Python package is then build using mathurin.

## Example use

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


## Further possibilities

It is already possible to import the maze into ndarrays:

    fn make_array(shape: Shape, walls: HashSet<Pos>) -> Maze {
        Array::from_shape_fn(shape, |(i, j)| {
            walls.contains(&(i, j))
        })
    }

    println!("{:?}", make_array(bot.shape, bot.walls));
