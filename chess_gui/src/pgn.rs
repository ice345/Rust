use crate::board::Board;
use crate::types::*;

/// Convert a move to Standard Algebraic Notation (SAN).
/// Must be called BEFORE the move is made on the board.
pub fn move_to_san(board: &Board, mv: &Move, color: Color) -> String {
    let piece = match board.get_piece(mv.from) {
        Some(p) => p,
        None => return Board::move_to_uci(mv),
    };

    // Castling
    if piece.piece_type == PieceType::King {
        let col_diff = mv.to.1 as i32 - mv.from.1 as i32;
        if col_diff == 2 {
            return "O-O".to_string();
        }
        if col_diff == -2 {
            return "O-O-O".to_string();
        }
    }

    let mut san = String::new();

    let is_capture = board.get_piece(mv.to).is_some()
        || (piece.piece_type == PieceType::Pawn
            && mv.from.1 != mv.to.1
            && board.get_piece(mv.to).is_none());

    // Piece letter (not for pawns)
    if piece.piece_type != PieceType::Pawn {
        san.push(match piece.piece_type {
            PieceType::King => 'K',
            PieceType::Queen => 'Q',
            PieceType::Rook => 'R',
            PieceType::Bishop => 'B',
            PieceType::Knight => 'N',
            _ => unreachable!(),
        });

        // Disambiguation: check if another piece of the same type can reach the same square
        let all_moves = board.generate_moves(color);
        let ambiguous: Vec<&Move> = all_moves
            .iter()
            .filter(|m| {
                m.to == mv.to
                    && m.from != mv.from
                    && board
                        .get_piece(m.from)
                        .map(|p| p.piece_type == piece.piece_type)
                        .unwrap_or(false)
            })
            .collect();

        if !ambiguous.is_empty() {
            let same_col = ambiguous.iter().any(|m| m.from.1 == mv.from.1);
            let same_row = ambiguous.iter().any(|m| m.from.0 == mv.from.0);

            if !same_col {
                // File is enough
                san.push((b'a' + mv.from.1 as u8) as char);
            } else if !same_row {
                // Rank is enough
                san.push((b'1' + (7 - mv.from.0) as u8) as char);
            } else {
                // Need both
                san.push((b'a' + mv.from.1 as u8) as char);
                san.push((b'1' + (7 - mv.from.0) as u8) as char);
            }
        }
    } else if is_capture {
        // Pawn captures include file of origin
        san.push((b'a' + mv.from.1 as u8) as char);
    }

    if is_capture {
        san.push('x');
    }

    // Destination square
    san.push_str(&Board::square_to_algebraic(mv.to.0, mv.to.1));

    // Promotion
    if let Some(promo) = mv.promotion {
        san.push('=');
        san.push(match promo {
            PieceType::Queen => 'Q',
            PieceType::Rook => 'R',
            PieceType::Bishop => 'B',
            PieceType::Knight => 'N',
            _ => '?',
        });
    }

    // Check / checkmate detection: apply the move and test
    let mut test_board = board.clone();
    test_board.make_move(*mv);
    let opponent = color.opposite();
    if test_board.is_in_check(opponent) {
        let opponent_moves = test_board.generate_moves(opponent);
        if opponent_moves.is_empty() {
            san.push('#');
        } else {
            san.push('+');
        }
    }

    san
}

pub fn generate_pgn(
    moves: &[MoveRecord],
    white: &str,
    black: &str,
    result: &str,
    date: &str,
) -> String {
    let mut pgn = String::new();

    pgn.push_str("[Event \"Casual Game\"]\n");
    pgn.push_str("[Site \"Chess GUI\"]\n");
    pgn.push_str(&format!("[Date \"{}\"]\n", date));
    pgn.push_str(&format!("[White \"{}\"]\n", white));
    pgn.push_str(&format!("[Black \"{}\"]\n", black));
    pgn.push_str(&format!("[Result \"{}\"]\n", result));
    pgn.push('\n');

    let mut line = String::new();
    for (i, record) in moves.iter().enumerate() {
        let move_text = if record.color == Color::White {
            format!("{}. {}", i / 2 + 1, record.san)
        } else {
            record.san.clone()
        };

        if line.len() + move_text.len() + 1 > 80 {
            pgn.push_str(&line);
            pgn.push('\n');
            line.clear();
        }

        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(&move_text);
    }

    if !line.is_empty() {
        pgn.push_str(&line);
    }
    pgn.push(' ');
    pgn.push_str(result);
    pgn.push('\n');

    pgn
}

pub fn parse_san_move(board: &Board, color: Color, san: &str) -> Option<Move> {
    let mut s = san.trim().to_string();
    if s.is_empty() {
        return None;
    }

    // Remove check/mate/eval annotations
    while s.ends_with('+') || s.ends_with('#') || s.ends_with('!') || s.ends_with('?') {
        s.pop();
    }

    // Handle castling
    if s == "O-O" || s == "0-0" {
        let moves = board.generate_moves(color);
        return moves.into_iter().find(|m| {
            let p = board.get_piece(m.from).unwrap();
            p.piece_type == PieceType::King && m.to.1 as i32 - m.from.1 as i32 == 2
        });
    } else if s == "O-O-O" || s == "0-0-0" {
        let moves = board.generate_moves(color);
        return moves.into_iter().find(|m| {
            let p = board.get_piece(m.from).unwrap();
            p.piece_type == PieceType::King && m.to.1 as i32 - m.from.1 as i32 == -2
        });
    }

    // Identify promotion
    let mut promotion = None;
    if let Some(pos) = s.find('=') {
        if pos + 1 < s.len() {
            let promo_char = s.chars().nth(pos + 1).unwrap();
            promotion = Some(match promo_char {
                'Q' => PieceType::Queen,
                'R' => PieceType::Rook,
                'B' => PieceType::Bishop,
                'N' => PieceType::Knight,
                _ => return None,
            });
        }
        s.truncate(pos); // removes '=' and the following character
    }

    s = s.replace("x", "");

    if s.len() < 2 {
        return None;
    }

    // Destination square
    let dest_str = &s[s.len() - 2..];
    let dest_col = dest_str.chars().next().unwrap();
    let dest_row = dest_str.chars().nth(1).unwrap();
    if !('a'..='h').contains(&dest_col) || !('1'..='8').contains(&dest_row) {
        return None;
    }
    let to_c = (dest_col as u8 - b'a') as usize;
    let to_r = 7 - (dest_row as u8 - b'1') as usize;
    let dest = (to_r, to_c);

    // Piece type and disambiguation
    let mut piece_type = PieceType::Pawn;
    let mut disambig_file = None;
    let mut disambig_rank = None;

    let prefix = &s[..s.len() - 2];
    if !prefix.is_empty() {
        let first_char = prefix.chars().next().unwrap();
        if first_char.is_uppercase() {
            piece_type = match first_char {
                'K' => PieceType::King,
                'Q' => PieceType::Queen,
                'R' => PieceType::Rook,
                'B' => PieceType::Bishop,
                'N' => PieceType::Knight,
                _ => return None,
            };

            // Disambiguation
            if prefix.len() > 1 {
                let d_str = &prefix[1..];
                for c in d_str.chars() {
                    if ('a'..='h').contains(&c) {
                        disambig_file = Some((c as u8 - b'a') as usize);
                    } else if ('1'..='8').contains(&c) {
                        disambig_rank = Some(7 - (c as u8 - b'1') as usize);
                    }
                }
            }
        } else {
            // Pawn move with disambiguation (e.g. captures: exd5)
            for c in prefix.chars() {
                if ('a'..='h').contains(&c) {
                    disambig_file = Some((c as u8 - b'a') as usize);
                }
            }
        }
    }

    let legal_moves = board.generate_moves(color);
    let mut matching_moves = Vec::new();

    for mv in legal_moves {
        if mv.to != dest {
            continue;
        }
        if mv.promotion != promotion {
            continue;
        }

        if let Some(p) = board.get_piece(mv.from) {
            if p.piece_type != piece_type {
                continue;
            }

            if let Some(f) = disambig_file
                && mv.from.1 != f
            {
                continue;
            }
            if let Some(r) = disambig_rank
                && mv.from.0 != r
            {
                continue;
            }
            matching_moves.push(mv);
        }
    }

    // Sometimes there are multiple matching moves due to pins not being evaluated aggressively
    // by generate_moves? generate_moves generates strictly legal moves so it should evaluate pins,
    // but just in case, we'll take the first if there's exactly 1.
    if matching_moves.len() == 1 {
        Some(matching_moves[0])
    } else {
        None
    }
}

pub fn parse_pgn(pgn: &str) -> Option<Vec<Move>> {
    let mut clean_pgn = String::new();
    let mut in_comment = false;
    let mut in_tag = false;
    let mut in_variation = 0;

    let mut chars = pgn.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            in_comment = true;
            continue;
        }
        if c == '}' {
            in_comment = false;
            continue;
        }
        if c == '[' {
            in_tag = true;
            continue;
        }
        if c == ']' {
            in_tag = false;
            continue;
        }
        if c == '(' {
            in_variation += 1;
            continue;
        }
        if c == ')' {
            in_variation -= 1;
            continue;
        }

        if c == ';' {
            while let Some(&n) = chars.peek() {
                if n == '\n' || n == '\r' {
                    break;
                }
                chars.next();
            }
            continue;
        }

        if !in_comment && !in_tag && in_variation <= 0 {
            clean_pgn.push(c);
        }
    }

    let mut moves = Vec::new();
    let mut board = Board::new();
    let mut color = Color::White;

    let tokens: Vec<&str> = clean_pgn.split_whitespace().collect();
    for token in tokens {
        // Skip game results
        if token == "1-0" || token == "0-1" || token == "1/2-1/2" || token == "*" {
            break;
        }

        // Handle e.g., "1.", "1.e4", or "1..."
        let mut san = token;
        if token.contains('.') {
            // Check if there is actual move text after the dots
            let after_dots = token.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.');
            if after_dots.is_empty() {
                continue; // It was just "1."
            }
            san = after_dots;
        }

        // Just extra safety in case split left something blank
        if san.is_empty() {
            continue;
        }

        if let Some(mv) = parse_san_move(&board, color, san) {
            moves.push(mv);
            board.make_move(mv);
            color = color.opposite();
        } else {
            // Handle error, or return what we collected so far?
            // Returning early what we collected can be more robust if there's a parse error late in the game
            break;
        }
    }

    if moves.is_empty() { None } else { Some(moves) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_san_pawn_move() {
        let board = Board::new();
        let mv = Move {
            from: (6, 4),
            to: (4, 4),
            promotion: None,
        };
        let san = move_to_san(&board, &mv, Color::White);
        assert_eq!(san, "e4");
    }

    #[test]
    fn test_san_knight_move() {
        let board = Board::new();
        let mv = Move {
            from: (7, 6),
            to: (5, 5),
            promotion: None,
        };
        let san = move_to_san(&board, &mv, Color::White);
        assert_eq!(san, "Nf3");
    }

    #[test]
    fn test_san_castling() {
        let mut board = Board::new();
        board.set_piece((7, 5), None);
        board.set_piece((7, 6), None);
        let mv = Move {
            from: (7, 4),
            to: (7, 6),
            promotion: None,
        };
        let san = move_to_san(&board, &mv, Color::White);
        assert_eq!(san, "O-O");
    }

    #[test]
    fn test_generate_pgn_basic() {
        let moves = vec![
            MoveRecord {
                mv: Move {
                    from: (6, 4),
                    to: (4, 4),
                    promotion: None,
                },
                san: "e4".to_string(),
                fen_after: String::new(),
                color: Color::White,
            },
            MoveRecord {
                mv: Move {
                    from: (1, 4),
                    to: (3, 4),
                    promotion: None,
                },
                san: "e5".to_string(),
                fen_after: String::new(),
                color: Color::Black,
            },
        ];
        let pgn = generate_pgn(&moves, "Player", "Stockfish", "*", "2026.03.20");
        assert!(pgn.contains("1. e4 e5"));
        assert!(pgn.contains("[White \"Player\"]"));
    }
}
