use std::iter::once;

use poise::serenity_prelude::EmojiId;
use shakmaty::{Board, Color, File, Move, Piece, Rank, Role, Square};

pub struct BoardDrawer<'a> {
    board: &'a Board,
    move_status: Option<Move>,
    turn: Color,
}

impl<'a> BoardDrawer<'a> {
    pub fn new(board: &'a Board, turn: Color, move_status: Option<Move>) -> Self {
        Self {
            board,
            move_status,
            turn,
        }
    }

    pub fn draw_discord(&self) -> String {
        let status_emoji = match self.move_status {
            Some(Move::Castle { .. }) => EmojiId::new(981219896128073728),
            Some(Move::EnPassant { .. }) => EmojiId::new(981219896153223198),
            Some(Move::Normal { capture, .. }) if capture.is_some() => {
                EmojiId::new(981223257984348170)
            }
            Some(Move::Normal { promotion, .. }) if promotion.is_some() => {
                EmojiId::new(981213862139420742)
            }
            Some(Move::Normal { to, .. }) => {
                let piece = self.board.piece_at(to);
                Self::discord_emoji_for_square(to, piece)
            }
            Some(_) => todo!(),
            None => EmojiId::new(979399119644799026),
        };

        // there are 10 rows 10 cols, Each with an emoji
        // The first vec is cols, the inner vec is rows
        let mut board_cols: Vec<Vec<EmojiId>> = Vec::with_capacity(10);

        let get_rank_emoji = |rank: &Rank| {
            match rank {
                Rank::First => 980682450726424586,
                Rank::Second => 980682450562859038,
                Rank::Third => 980682450646728797,
                Rank::Fourth => 980682450613182534,
                Rank::Fifth => 980682450617368626,
                Rank::Sixth => 980682451322019881,
                Rank::Seventh => 980682450562875412,
                Rank::Eighth => 980682450583830558,
            }
            .into()
        };

        let get_file_emoji = |file: &File| {
            match file {
                File::A => 980682450860666880,
                File::B => 980682450822922270,
                File::C => 980682450839695402,
                File::D => 980682450890002502,
                File::E => 980682450386694205,
                File::F => 980682450743205979,
                File::G => 980682450885820436,
                File::H => 980682451091341312,
            }
            .into()
        };

        // {status_emoji}{Ranks}{status_emoji}
        board_cols.push(
            once(status_emoji)
                .chain(Rank::ALL.iter().map(get_rank_emoji))
                .chain(once(status_emoji))
                .collect(),
        );

        for file in File::ALL {
            board_cols.push(
                once(get_file_emoji(&file))
                    .chain(Rank::ALL.into_iter().map(|rank| {
                        let square = Square::from_coords(file, rank);
                        let piece = self.board.piece_at(square);
                        Self::discord_emoji_for_square(square, piece)
                    }))
                    .chain(once(get_file_emoji(&file)))
                    .collect(),
            );
        }

        board_cols.push(
            once(status_emoji)
                .chain(Rank::ALL.iter().map(get_rank_emoji))
                .chain(once(status_emoji))
                .collect(),
        );

        // Since we want the rows, we need to transpose the cols
        (0..10)
            .map(|i| {
                board_cols
                    .iter()
                    .map(|col| format!("<:emoji:{}>", col[i].get()))
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn discord_emoji_for_square(square: Square, piece: Option<Piece>) -> EmojiId {
        match (square.is_light(), piece) {
            (true, None) => 979399119644799026,
            (false, None) => 979399119397355520,
            (
                true,
                Some(Piece {
                    color: Color::White,
                    role: Role::Pawn,
                }),
            ) => 979396455162863676,
            (
                true,
                Some(Piece {
                    color: Color::Black,
                    role: Role::Pawn,
                }),
            ) => 979396455255138304,
            (
                true,
                Some(Piece {
                    color: Color::White,
                    role: Role::Knight,
                }),
            ) => 979396455196426280,
            (
                true,
                Some(Piece {
                    color: Color::Black,
                    role: Role::Knight,
                }),
            ) => 979396455095750686,
            (
                true,
                Some(Piece {
                    color: Color::White,
                    role: Role::Bishop,
                }),
            ) => 979396455162863676,
            (
                true,
                Some(Piece {
                    color: Color::Black,
                    role: Role::Bishop,
                }),
            ) => 979396455129313370,
            (
                true,
                Some(Piece {
                    color: Color::White,
                    role: Role::Rook,
                }),
            ) => 979396454986702848,
            (
                true,
                Some(Piece {
                    color: Color::Black,
                    role: Role::Rook,
                }),
            ) => 979396454965731369,
            (
                true,
                Some(Piece {
                    color: Color::White,
                    role: Role::Queen,
                }),
            ) => 979396455112532019,
            (
                true,
                Some(Piece {
                    color: Color::Black,
                    role: Role::Queen,
                }),
            ) => 979442406992801822,
            (
                true,
                Some(Piece {
                    color: Color::White,
                    role: Role::King,
                }),
            ) => 979396455162871878,
            (
                true,
                Some(Piece {
                    color: Color::Black,
                    role: Role::King,
                }),
            ) => 979396454726647819,
            (
                false,
                Some(Piece {
                    color: Color::White,
                    role: Role::Pawn,
                }),
            ) => 979396455192223824,
            (
                false,
                Some(Piece {
                    color: Color::Black,
                    role: Role::Pawn,
                }),
            ) => 979396455498416158,
            (
                false,
                Some(Piece {
                    color: Color::White,
                    role: Role::Knight,
                }),
            ) => 979397693493346344,
            (
                false,
                Some(Piece {
                    color: Color::Black,
                    role: Role::Knight,
                }),
            ) => 979397693619187733,
            (
                false,
                Some(Piece {
                    color: Color::White,
                    role: Role::Bishop,
                }),
            ) => 979396455280304199,
            (
                false,
                Some(Piece {
                    color: Color::Black,
                    role: Role::Bishop,
                }),
            ) => 979396454986711040,
            (
                false,
                Some(Piece {
                    color: Color::White,
                    role: Role::Rook,
                }),
            ) => 979397692914557012,
            (
                false,
                Some(Piece {
                    color: Color::Black,
                    role: Role::Rook,
                }),
            ) => 979397693279453255,
            (
                false,
                Some(Piece {
                    color: Color::White,
                    role: Role::Queen,
                }),
            ) => 979442406992797716,
            (
                false,
                Some(Piece {
                    color: Color::Black,
                    role: Role::Queen,
                }),
            ) => 979396454957338634,
            (
                false,
                Some(Piece {
                    color: Color::White,
                    role: Role::King,
                }),
            ) => 979397693375926272,
            (
                false,
                Some(Piece {
                    color: Color::Black,
                    role: Role::King,
                }),
            ) => 979396454886035477,
        }
        .into()
    }
}
