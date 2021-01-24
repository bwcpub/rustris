/*

A no-frills Tetris implementation written in Rust,
using the Piston game engine, and Rodio for music.

Tetris was invented by Alexey Pajitnov and Vladimir Pokhilko. Tetris(TM) and associated copyrights are owned by Tetris Holding LLC.

Classic NES Tetris Type 3 background music is composed by Hirokazu Tanaka and is (C) Nintendo. All rights for the background music belong to them.

MIT License

Copyright (c) 2021 Ben Cantrick

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

Tetris was invented by Alexey Pajitnov and Vladimir Pokhilko. Tetris(TM) and associated copyrights are owned by Tetris Holding LLC.

Classic NES Tetris Type 3 background music is composed by Hirokazu Tanaka and is (C) Nintendo. All rights for the background music belong to them.

*/

extern crate piston_window;
extern crate rand;

use piston_window::*;

use rand::seq::SliceRandom;
use rand::thread_rng;

use std::io::BufReader;
use std::fs::File;

#[derive(PartialEq, Copy, Clone)]
enum TetriminoKind { I, J, L, O, S, T, Z }


#[derive(Copy, Clone)]
struct Tetrimino {
    kind: TetriminoKind,
    color: [f32; 4],    // R, G, B, A
    shape: [[u8; 4]; 4]
}


impl Tetrimino
{
    const fn new(kind: TetriminoKind) -> Self
    {
        match kind
        {
            TetriminoKind::I => Tetrimino { kind: TetriminoKind::I,
                                            color: [ 1.0, 1.0, 1.0, 1.0 ],    // white
                                            shape: [[0, 0, 1, 0],
                                                    [0, 0, 1, 0],
                                                    [0, 0, 1, 0],
                                                    [0, 0, 1, 0]] },

            TetriminoKind::J => Tetrimino { kind: TetriminoKind::J,
                                            color: [ 0.0, 0.0, 1.0, 1.0 ],    // blue
                                            shape: [[ 1, 0, 0, 0 ],
                                                    [ 1, 1, 1, 0 ],
                                                    [ 0, 0, 0, 0 ],
                                                    [ 0, 0, 0, 0 ]] },

            TetriminoKind::L => Tetrimino { kind: TetriminoKind::L,
                                            color: [ 0.0, 1.0, 1.0, 1.0 ],    // cyan
                                            shape: [[ 0, 0, 1, 0 ],
                                                    [ 1, 1, 1, 0 ],
                                                    [ 0, 0, 0, 0 ],
                                                    [ 0, 0, 0, 0 ]] },

            TetriminoKind::S => Tetrimino { kind: TetriminoKind::S,
                                            color: [ 1.0, 0.0, 1.0, 1.0 ],    // magenta
                                            shape: [[ 0, 1, 1, 0 ],
                                                    [ 1, 1, 0, 0 ],
                                                    [ 0, 0, 0, 0 ],
                                                    [ 0, 0, 0, 0 ]] },

            TetriminoKind::Z => Tetrimino { kind: TetriminoKind::Z,
                                            color: [ 1.0, 0.0, 0.0, 1.0 ],    // red
                                            shape: [[ 1, 1, 0, 0 ],
                                                    [ 0, 1, 1, 0 ],
                                                    [ 0, 0, 0, 0 ],
                                                    [ 0, 0, 0, 0 ]] },

            TetriminoKind::O => Tetrimino { kind: TetriminoKind::O,
                                            color: [ 0.0, 1.0, 0.0, 1.0 ],    // green
                                            shape: [[ 0, 0, 0, 0 ],
                                                    [ 0, 0, 0, 0 ],
                                                    [ 0, 1, 1, 0 ],
                                                    [ 0, 1, 1, 0 ]] },

            TetriminoKind::T => Tetrimino { kind: TetriminoKind::T,
                                            color: [ 1.0, 1.0, 0.0, 1.0 ],    // yellow
                                            shape: [[ 0, 1, 0, 0 ],
                                                    [ 1, 1, 1, 0 ],
                                                    [ 0, 0, 0, 0 ],
                                                    [ 0, 0, 0, 0 ]] }
        }
    }
}

// A Tetris playfield is known as a "Well".
// It is composed of 24 rows, each of which is 10 columns wide.
// Usually only the bottom 20 rows are fully visible.
// If possible, a bit of row 21 should be shown also.
// (https://tetris.fandom.com/wiki/Tetris_Guideline, and
// https://en.wikipedia.org/wiki/Tetris)
//
// Our window is 1280 x 720, but let's only use 700 pixels, to leave some margin.
// 700 pixels / 20 visible rows = 35, so each row will be 35 pixels tall.
// The well is 10 rows wide * 35 pixels/row = 350 pixels in width.

type Well = [[u8; 10]; 24];


struct GameState
{
    game_over: bool,
    fall_counter: u32,
    well: Well,
    ttmo_bag: Vec<Tetrimino>,    // Randomized bag of all 7 tetriminos.
    curr_ttmo: Tetrimino,
    next_ttmo: Tetrimino,
    ttmo_row: i32,        // Curr piece's location in the well.
    ttmo_col: i32,
    key_map: [bool; 6]    // MoveLeft, MoveRight, RotateCCW, RotateCW, SoftDrop, HardDrop
}    


//
// ////////// MAIN //////////
//
fn main()
{
    // Obviously we're going to need a window if we want to display anything.
    let mut window: PistonWindow =
        WindowSettings::new("Rustris", [1280, 720])    // Window title, size.
        .exit_on_esc(true)
        .vsync(true)
        .build().unwrap();

    // By default, Piston sends 120 update events per second. Lower that to 30/sec.
    // (Yes, multiple renderings will happen between each update. Code accordingly!)
    window.events.set_ups(30);

    // Set up the music playing infrastructure. Will be started/repeated/stopped in main loop.
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let music_sink = rodio::Sink::try_new(&stream_handle).unwrap();
    music_sink.set_volume(0.1);    // Should really make the audio file quieter instead.
    // The sound plays in a separate audio thread, so the main thread needs to stay alive.

    // Actual state init.
    let mut blink_counter = 0;

    let mut starter_bag = create_random_bag();
    let starter_first_ttmo = starter_bag.pop().unwrap();
    let starter_second_ttmo = starter_bag.pop().unwrap();

    let mut game_state = GameState {
        game_over: false,
        fall_counter: 0,
        well: [[0u8 ; 10]; 24],
        ttmo_bag: starter_bag,
        curr_ttmo: starter_first_ttmo,
        next_ttmo: starter_second_ttmo,
        ttmo_row: 2,
        ttmo_col: 3,
        key_map: [false; 6]
    };

    // *****
    // ***** MAIN LOOP
    // *****
    while let Some(event) = window.next()
    {
        match event
        {
            // Because vsync is on, render events should happen every screen refresh. (Usually 60 times per second.)
            Event::Loop(Loop::Render(_args_not_used)) => {
                render(&mut window, &event,
                       &game_state.ttmo_row, &game_state.ttmo_col, &game_state.curr_ttmo,
                       &game_state.next_ttmo, &mut game_state.well);
            }

            // Update events are received here. Update the game state accordingly.
            Event::Loop(Loop::Update(_args_also_not_used)) =>
            {
                if game_state.game_over
                {                    
                    if blink_counter == 15 {
                        game_state.well = [[0u8; 10]; 24];
                    }
                    if blink_counter == 30 {
                        game_state.well = [[1u8; 10]; 24];
                        blink_counter = 0;
                    }
                    blink_counter += 1;
                }
                else {

                    game_update(&mut game_state);

                    if game_state.game_over {
                       music_sink.stop();
                    } else {
                        if music_sink.empty() {
                           let music_file = File::open("NESTetrisMusic3.ogg").unwrap();    // Path relative to Cargo.toml
                           let music_source = rodio::Decoder::new(BufReader::new(music_file)).unwrap();
                           music_sink.append(music_source);
                           music_sink.play();
                       }
                   }
                }
            }

            // Keyboard press/release events.
            Event::Input(Input::Button(button_args), _time_stamp) =>
            {                
                if button_args.state == ButtonState::Press {    // We only care about presses, not releases (or others?!).
                    track_keys(&mut game_state.key_map, button_args);
                }
            }

            // Rust forces you to consider all possible Event types. This "discard all other events" clause satisfies that requirement.
            _ => {
                // println!("Other event: {:?}", event);    // Super spammy!
                ()
            }
        }    // match
    }    // while

}    // main



/// Sets bool flags in key_map when a key event notifies us that a key has been pressed.
fn track_keys(key_map: &mut [bool; 6], btn_info: ButtonArgs)
{
    match btn_info.button    // We only care about a few keys, all others are ignored.
    {
        Button::Keyboard(Key::Left)  => key_map[0] = true,    // MoveLeft
        Button::Keyboard(Key::Right) => key_map[1] = true,    // MoveRight
        Button::Keyboard(Key::Up)    => key_map[2] = true,    // RotateCCW
        Button::Keyboard(Key::D)     => key_map[2] = true,    // RotateCCW
        Button::Keyboard(Key::F)     => key_map[3] = true,    // RotateCW
        Button::Keyboard(Key::Down)  => key_map[4] = true,    // SoftDrop
        Button::Keyboard(Key::Space) => key_map[5] = true,    // HardDrop
        _ => ()                                               // Ignore all others
    }
}


/// Implements the main logic of the game. Pieces fall, full rows disappear, etc.
fn game_update(game_state: &mut GameState)
{
    // Pieces fall fairly slowly: 30 ups per sec / 20 ups per fall = 0.66 (repeating, of course) secs per fall.

    if game_state.fall_counter < 20 {
        game_state.fall_counter += 1;    // Not time to fall yet...
    }
    else    // Time to fall!
    {
        game_state.fall_counter = 0;

        if would_collide(&game_state.curr_ttmo, &game_state.well, &(game_state.ttmo_row + 1), &game_state.ttmo_col)
        {
            freeze_to_well(&game_state.curr_ttmo, &mut game_state.well, &game_state.ttmo_row, &game_state.ttmo_col);
            game_state.well = clear_complete_rows(game_state.well);

            if game_state.ttmo_bag.is_empty() { game_state.ttmo_bag = create_random_bag(); }
            game_state.curr_ttmo = game_state.next_ttmo;
            game_state.next_ttmo = game_state.ttmo_bag.pop().unwrap();

            game_state.ttmo_row = 2;    // Place near top...
            game_state.ttmo_col = 3;    // ...and near center.

            // THAT'S IT, MAN! GAME OVER, MAN!!
            if would_collide(&game_state.curr_ttmo, &game_state.well, &game_state.ttmo_row, &game_state.ttmo_col)
            {
                game_state.game_over = true;
            }
        }
          
        else { game_state.ttmo_row += 1; }    // Move curr piece down one row.
    }

    // Keys are checked every update.

    // MoveLeft
    if game_state.key_map[0] && !would_collide(&game_state.curr_ttmo, &game_state.well, &game_state.ttmo_row, &(game_state.ttmo_col - 1))
        { game_state.ttmo_col -= 1; }

    // MoveRight
    if game_state.key_map[1] && !would_collide(&game_state.curr_ttmo, &game_state.well, &game_state.ttmo_row, &(game_state.ttmo_col + 1))
        { game_state.ttmo_col += 1; }

    // SoftDrop
    if game_state.key_map[4] && !would_collide(&game_state.curr_ttmo, &game_state.well, &(game_state.ttmo_row + 1), &game_state.ttmo_col)
        { game_state.ttmo_row += 1; }

    // HardDrop
    if game_state.key_map[5]
    {
        for row in game_state.ttmo_row..24 {
            if would_collide(&game_state.curr_ttmo, &game_state.well, &row, &game_state.ttmo_col) {
                game_state.ttmo_row = row - 1;
                break;
            }
        }
    }

    // RotateCCW
    if game_state.key_map[2] {
        rotate_tetrimino(&mut game_state.curr_ttmo, false);
        if would_collide(&game_state.curr_ttmo, &game_state.well, &game_state.ttmo_row, &game_state.ttmo_col) {
            rotate_tetrimino(&mut game_state.curr_ttmo, true);
        }
    }

    // RotateCW
    if game_state.key_map[3] {
        rotate_tetrimino(&mut game_state.curr_ttmo, true);
        if would_collide(&game_state.curr_ttmo, &game_state.well, &game_state.ttmo_row, &game_state.ttmo_col) {
            rotate_tetrimino(&mut game_state.curr_ttmo, false);
        }
    }

    game_state.key_map = [false; 6];                // all keys now unpressed
}


/// Creates and returns a vector containing a randomized ordering of the 7 standard tetriminos.
fn create_random_bag() -> Vec<Tetrimino>
{
    let mut tetrimino_bag: Vec<Tetrimino> = vec![ Tetrimino::new(TetriminoKind::I),
                                                  Tetrimino::new(TetriminoKind::J),
                                                  Tetrimino::new(TetriminoKind::L),
                                                  Tetrimino::new(TetriminoKind::O),
                                                  Tetrimino::new(TetriminoKind::S),
                                                  Tetrimino::new(TetriminoKind::T),
                                                  Tetrimino::new(TetriminoKind::Z)  ];
    tetrimino_bag.shuffle(&mut thread_rng());
    tetrimino_bag.shuffle(&mut thread_rng());
    tetrimino_bag.shuffle(&mut thread_rng());    // One randomize was appearing not very random...

    tetrimino_bag
}



/// Rotates the given Tetrimino by 90 degrees, either clockwise or counterCW.
fn rotate_tetrimino(ttmo: &mut Tetrimino, clockwise: bool)
{
    // Rotating the O Tetrimino is pointless.
    if ttmo.kind == TetriminoKind::O { return; }

    let source = ttmo.shape;
    let mut rotated: [[u8; 4]; 4] = [[0; 4]; 4];

    // Only TetriminoKind::I needs all four rows of .shape rotated.
    // The others can be done by only rotating the top-left 3x3 submatrix.
    let matrix_size: usize;
    if ttmo.kind == TetriminoKind::I { matrix_size = 4; } else { matrix_size = 3; }

    for row in 0..matrix_size
    {
        // First row becomes last column, and so on.
        if clockwise {
            for col in 0..matrix_size {
                rotated[col][(matrix_size - 1) - row] = source[row][col];    // matrix_size is 1 based, array index is 0 based.
            }
        }

        // First row becomes first column, but upside down.
        else {
            for col in 0..matrix_size {
                rotated[(matrix_size - 1) - col][row] = source[row][col];
            }
        }
    }

    ttmo.shape = rotated;    
}


/// Returns true if the given Tetrimino, placed in the given playfield,
/// at the given row and col, would collide with something.
fn would_collide(ttmo: &Tetrimino, well: &Well, row: &i32, col: &i32) -> bool
{
    let mut well_row: i32;
    let mut well_col: i32;

    for ttmo_row in 0..4 {
        for ttmo_col in 0..4 {

            // Tetrimino has no square here, collison is not possible.
            if ttmo.shape[ttmo_row][ttmo_col] == 0 { continue; }

            // Compute well coords of ttmo square.
            well_row = ttmo_row as i32 + *row;
            well_col = ttmo_col as i32 + *col;

            // Collisions with well walls, floor.
            if well_col < 0 { return true; }
            if well_col > 9 { return true; }
            if well_row > 23 { return true; }
    
            // Collision with a block already frozen in the well.
            if well[well_row as usize][well_col as usize] != 0 { return true; }
        }
    }

    false
}


/// Copies the given tetrimino's squares into the given well at the given (well_row, well_col).
fn freeze_to_well(ttmo: &Tetrimino, well: &mut Well, well_row: &i32, well_col: &i32)
{
    for row in 0..4 {
        for col in 0..4 {
            if ttmo.shape[row][col] == 0 { continue; }
            // println!("well[{}][{}] = 1", (*well_row + row as i32) as usize, (*well_col + col as i32) as usize);
            well[(*well_row + row as i32) as usize][(*well_col + col as i32) as usize] = ttmo.shape[row][col];
        }
    }
}


/// Clears out complete rows in the given well, and moves the rows above them down.
fn clear_complete_rows(well: Well) -> Well
{
    // Copy partial rows to a new well. Ignore both empty and full rows.
    let mut new_well: Well = [[0; 10]; 24];
    let mut new_well_row: usize = 23;

    for old_well_row in (0..24).rev()    // Start at bottom and work upward.
    {
        // The number of non-empty columns in a row is its "population count".
        let mut pop_count = 0;
        for col in 0..10 {
            if well[old_well_row][col] != 0 { pop_count += 1; }    // Take your fold() and shove it!
        }

        // Totally empty or totally full rows are ignored.
        if pop_count == 0 || pop_count == 10 { continue; }

        // Copy partial row to new well, in lowest row possible.
        if well[old_well_row].iter().sum::<u8>() > 0    // if well row contains blocks
        {    
            new_well[new_well_row] = well[old_well_row];    // Copy row to new well.
            new_well_row -= 1;
        }
    }

    new_well
}


fn render(win: &mut PistonWindow, re: &Event, row: &i32, col: &i32, curr: &Tetrimino, next: &Tetrimino, well: &Well)
{
    // "Clear" window by drawing all pixels grey.
    win.draw_2d(re, |_context, graphics, _device| { clear([0.5; 4], graphics); } );

    // Draw the outline of the playfield. 350 wide + 2 pixel gap on left and right => 354 pixels wide.
    win.draw_2d(re, |context, graphics, _device| { rectangle([0.0, 0.0, 0.0, 1.0], [463.0, -140.0, 354.0, 842.0], context.transform, graphics); } );

    draw_well_blocks(win, re, well);                      // Draw the contents of the playfield.
    draw_tetrimino_well(win, re, row, col, curr);         // Draw the currently falling tetrimino.
    draw_tetrimino_pixel(win, re, 320.0, 115.0, next);    // Draw the next tetrimino, always at the same place.
}


/// Renders the given Tetrimino at the given well coordinates.
fn draw_tetrimino_well(win: &mut PistonWindow, re: &Event, well_row: &i32, well_col: &i32, ttmo: &Tetrimino)
{
    let (x, y) = well_to_pixel(*well_row, *well_col);
    draw_tetrimino_pixel(win, re, x, y, ttmo);
}

/// Renders the given Tetrimino at the given pixel coordinates.
fn draw_tetrimino_pixel(win: &mut PistonWindow, e: &Event, px: f64, py: f64, ttmo: &Tetrimino)
{
    // DEBUG ONLY: Draw transparent grey bounding box around tetrimino.
    // win.draw_2d(e, |context, graphics, _device| { rectangle([0.5; 4], [px, py, 140.0, 140.0], context.transform, graphics); } );

    for ttmo_row in 0..4 {
        for ttmo_col in 0..4 {
            
            if ttmo.shape[ttmo_row][ttmo_col] == 0 { continue; }    // No square to be drawn here.

            let x_offs = px + 35.0 * ttmo_col as f64;    // Each square in the Tetrimino is 35x35 pixels.
            let y_offs = py + 35.0 * ttmo_row as f64;    // Pixel Y coords increase downward.

            win.draw_2d(e,
                |context, graphics, _device| {
                    // Draw 33x33 square inside 35x35 space.
                    rectangle(ttmo.color, [x_offs + 1.0, y_offs + 1.0, 33.0, 33.0], context.transform, graphics);
                }
            );
        }
    }
}


/// Renders the squares of the given playfield.
fn draw_well_blocks(win: &mut PistonWindow, e: &Event, well: &Well)
{
    for row in 0..24 {
        for col in 0..10 {
            
            if well[row][col] == 0 { continue; }    // No square to be drawn here.

            let (x_offs, y_offs) = well_to_pixel(row as i32, col as i32);
            win.draw_2d(e,
                |context, graphics, _device| {
                    // Draw 33x33 square inside 35x35 space.
                    rectangle( [1.0, 1.0, 1.0, 1.0], [x_offs + 1.0, y_offs + 1.0, 33.0, 33.0], context.transform, graphics);
                }
            );
        }
    }
}


/// Takes a well coordinate (row, column) and converts it to a pixel value (x, y).
/// The pixel value is the upper-left-most pixel of the square at the given well coordinate.
fn well_to_pixel(row: i32, col: i32) -> (f64, f64)
{
    ( (col as f64) * 35.0 + 465.0, (row as f64) * 35.0 - 140.0 )
}
