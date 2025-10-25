const N: usize = 8;  // board size

fn main() {
    let mut board = init_board();
    print_board(&board);

    let mut turn = 0;  // turn counts
    let mut pass_count = 0;  // pass_count = 2 game over

    loop {
        let cur = if turn % 2 == 0 { 'B' } else { 'W' };  // current player
        let oppo = if cur == 'B' { 'W' } else { 'B' };  // opposite player
        let (count_black, count_white, valid_cur, _) = analyze_board(&board, cur, oppo);

        // has no valid move
        if !valid_cur {
            println!("{} player has no valid move.", cur);
            pass_count += 1;
            if pass_count == 2 {
                print_winner(count_black, count_white);
                break;
            }
            turn += 1;
            continue;
        }

        // has valid move
        pass_count = 0;

        use std::io::{self, Write};
        print!("Enter move for colour {} (RowCol): ", cur);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }

        if let Some((r, c)) = parse_move(input.trim()) {
            if board[r][c] == '.'
                && is_valid_move(r as i32, c as i32, &mut board, cur, oppo, true)
            {
                turn += 1;
                print_board(&board);
            } else {
                println!("Invalid move. Try again.");
                print_board(&board);
            }
        } else {
            println!("Invalid move. Try again.");
            print_board(&board);
        }
    }
}

fn init_board() -> [[char; N]; N] {
    let mut board = [['.'; N]; N];
    board[3][3] = 'W';
    board[4][4] = 'W';
    board[3][4] = 'B';
    board[4][3] = 'B';
    board
}

fn analyze_board(board: &[[char; N]; N], color: char, oppo: char) -> (i32, i32, bool, bool) {
    let mut black = 0;
    let mut white = 0;
    let mut valid_cur = false;
    let mut valid_oppo = false;

    for i in 0..N {
        for j in 0..N {
            match board[i][j] {
                'B' => black += 1,
                'W' => white += 1,
                '.' => {
                    if is_valid_move(i as i32, j as i32, &mut board.clone(), color, oppo, false) {
                        valid_cur = true;
                    }
                    if is_valid_move(i as i32, j as i32, &mut board.clone(), oppo, color, false) {
                        valid_oppo = true;
                    }
                }
                _ => {}
            }
        }
    }
    (black, white, valid_cur, valid_oppo)
}

fn print_winner(black: i32, white: i32) {
    if black > white {
        println!("Black wins by {} points!", black - white);
    } else if white > black {
        println!("White wins by {} points!", white - black);
    } else {
        println!("Draw!");
    }
}

fn parse_move(s: &str) -> Option<(usize, usize)> {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() != 2 {
        return None;
    }
    let r = (chars[0] as u8).wrapping_sub(b'a');
    let c = (chars[1] as u8).wrapping_sub(b'a');
    if r < 8 && c < 8 {
        Some((r as usize, c as usize))
    } else {
        None
    }
}

fn is_valid_move(
    bi: i32,
    bj: i32,
    board: &mut [[char; N]; N],
    color: char,
    oppo_color: char,
    update: bool,
) -> bool {
    let dirs = [
        [-1, 0],
        [1, 0],
        [0, -1],
        [0, 1],
        [-1, -1],
        [-1, 1],
        [1, -1],
        [1, 1],
    ];
    let mut found = false;

    for dir in dirs {
        let ni = bi + dir[0];
        let nj = bj + dir[1];
        if is_inside_board(ni, nj) && board[ni as usize][nj as usize] == oppo_color {
            if check(ni, nj, board, color, oppo_color, update, &dir) {
                found = true;
                if update {
                    board[bi as usize][bj as usize] = color;
                }
            }
        }
    }
    found
}

fn check(
    ni: i32,
    nj: i32,
    board: &mut [[char; N]; N],
    color: char,
    oppo_color: char,
    update: bool,
    dir: &[i32; 2],
) -> bool {
    if !is_inside_board(ni, nj) {
        return false;
    }
    match board[ni as usize][nj as usize] {
        x if x == color => true,
        x if x == oppo_color => {
            if check(
                ni + dir[0],
                nj + dir[1],
                board,
                color,
                oppo_color,
                update,
                dir,
            ) {
                if update {
                    board[ni as usize][nj as usize] = color;
                }
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn is_inside_board(i: i32, j: i32) -> bool {
    i >= 0 && i < N as i32 && j >= 0 && j < N as i32
}

fn print_board(board: &[[char; N]; N]) {
    println!("  abcdefgh");
    for i in 0..N {
        let row_label = (b'a' + i as u8) as char;
        print!("{} ", row_label);
        for j in 0..N {
            print!("{}", board[i][j]);
        }
        println!();
    }
}
