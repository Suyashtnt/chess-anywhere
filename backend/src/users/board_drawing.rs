use std::iter::once;

use poise::serenity_prelude::EmojiId;
use shakmaty::{Board, Color, File, Move, Piece, Rank, Square};

pub struct BoardDrawer<'a> {
    board: &'a Board,
    move_status: Option<Move>,
    turn: Color,
}

impl BoardDrawer<'_> {
    pub fn new(board: &Board, turn: Color, move_status: Option<Move>) -> Self {
        Self {
            board,
            move_status,
            turn,
        }
    }

    pub fn draw_discord(&self) -> String {
        let status_emoji = match self.move_status {
            _ => todo!(), // MoveStatus::PieceMoved(piece) => Self::discord_emoji_for_square(
                          //     if self.turn == Color::White {
                          //         Square::B1
                          //     } else {
                          //         Square::A1
                          //     },
                          //     Some(piece),
                          // ),
                          // MoveStatus::GameStart => EmojiId::new(979399119644799026),
                          // MoveStatus::Castle => EmojiId::new(981219896128073728),
                          // MoveStatus::EnPassant => EmojiId::new(981219896153223198),
                          // MoveStatus::Capture => EmojiId::new(981223257984348170),
                          // MoveStatus::Check => EmojiId::new(todo!()),
                          // MoveStatus::Checkmate => EmojiId::new(todo!()),
                          // MoveStatus::Stalemate => EmojiId::new(todo!()),
        };

        // there are 10 rows 10 cols, Each with an emoji
        // The first vec is cols, the inner vec is rows
        let mut board_cols: Vec<Vec<EmojiId>> = Vec::with_capacity(10);

        // starting col: {status_emoji}{Ranks}{status_emoji}
        board_cols.push(
            once(status_emoji)
                .chain(Rank::ALL.iter().map(|rank| match rank {
                    Rank::First => EmojiId::new(980682450726424586),
                    Rank::Second => EmojiId::new(980682450562859038),
                    Rank::Third => EmojiId::new(980682450646728797),
                    Rank::Fourth => EmojiId::new(980682450613182534),
                    Rank::Fifth => EmojiId::new(980682450617368626),
                    Rank::Sixth => EmojiId::new(980682451322019881),
                    Rank::Seventh => EmojiId::new(980682450562875412),
                    Rank::Eighth => EmojiId::new(980682450583830558),
                }))
                .chain(once(status_emoji))
                .collect(),
        );

        for file in File::ALL {
            for rank in Rank::ALL {
                let square = Square::from_coords(file, rank);
                let piece = self.board.piece_at(square);

                todo!()
            }
        }

        // ending col
        board_cols.push(
            once(status_emoji)
                .chain(Rank::ALL.iter().map(|rank| match rank {
                    Rank::First => EmojiId::new(980682450726424586),
                    Rank::Second => EmojiId::new(980682450562859038),
                    Rank::Third => EmojiId::new(980682450646728797),
                    Rank::Fourth => EmojiId::new(980682450613182534),
                    Rank::Fifth => EmojiId::new(980682450617368626),
                    Rank::Sixth => EmojiId::new(980682451322019881),
                    Rank::Seventh => EmojiId::new(980682450562875412),
                    Rank::Eighth => EmojiId::new(980682450583830558),
                }))
                .chain(once(status_emoji))
                .collect(),
        );

        // join each col from left to right
        // we know there are 10, so we can transpose the board

        todo!()
    }

    fn discord_emoji_for_square(square: Square, piece: Option<Piece>) -> EmojiId {
        todo!()
    }
}
