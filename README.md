# Kenny

*A KenKen puzzle generator and solver.*

## Features

* Generate KenKen puzzles
* Solve KenKen puzzles
* Robust constraint propagation algorithm to solve most puzzles without backtracking
* Backtracking to solve harder puzzles
* Save SVG images of puzzles
* Save an image at each step of the solution

## Generate a puzzle

    kenny --generate --width 5 --save-all

Run the above command to generate a new, random KenKen puzzle. An image of the puzzle will be saved as well as a text version of the puzzle which may be passed back to kenny for solving the puzzle.

## Solve a puzzle

    kenny --generate --solve --save-all

Use the `--solve` flag to solve the puzzle. This may be used together with the `--generate` flag to generate and solve at once. In the output, you will find an SVG image of the solved puzzle as well as an image for every step of the solution in a folder named "steps".

## More

    kenny --help

Run with `--help` to see more options

## Future Goals?

* Detect puzzle difficulty level
* Support no-op puzzles
* Specify operators to be used in the puzzle
