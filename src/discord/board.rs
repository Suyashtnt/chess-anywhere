use std::iter::once;

use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter, EmojiId};
use shakmaty::{Board, Color, File, Move, Piece, Rank, Role, Square};

use crate::backend::chess::MoveStatus;

pub struct BoardDrawer<'a> {
    board: &'a Board,
    move_status: &'a MoveStatus,
}

impl<'a> BoardDrawer<'a> {
    pub fn new(board: &'a Board, move_status: &'a MoveStatus) -> Self {
        Self { board, move_status }
    }

    pub fn draw(&self) -> String {
        let status_emoji = match self.move_status {
            MoveStatus::Move(Move::Castle { .. }) => 981219896128073728,
            MoveStatus::Move(Move::EnPassant { .. }) => 981219896153223198,
            MoveStatus::Move(Move::Normal { capture, .. }) if capture.is_some() => {
                981223257984348170
            }
            MoveStatus::Move(Move::Normal { promotion, .. }) if promotion.is_some() => {
                981213862139420742
            }
            MoveStatus::Move(Move::Normal { to, .. }) => {
                let piece = self.board.piece_at(*to);
                Self::emoji_for_square(*to, piece).get()
            }
            MoveStatus::Move(_) => unreachable!("No fairy chess pieces yet"),
            MoveStatus::GameStart => 979399119644799026,
            MoveStatus::Check => 981209797716238366,
            MoveStatus::Checkmate => 981209797712035920,
            MoveStatus::Stalemate => 979399119644799026,
            MoveStatus::DrawOffer(_) => 981223404520763522,
            MoveStatus::Draw => 981223404520763522,
        }
        .into();

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
                        Self::emoji_for_square(square, piece)
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

    fn emoji_for_square(square: Square, piece: Option<Piece>) -> EmojiId {
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
            ) => 979396454944751656,
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

pub fn create_board_embed(
    current_player_name: &str,
    other_player_name: &str,
    our_color: &Color,
    board: &Board,
    move_status: &MoveStatus,
    is_our_turn: bool,
) -> CreateEmbed {
    let board_drawer = BoardDrawer::new(board, move_status);

    let mut embed = CreateEmbed::default()
        .title(format!("{} vs {}", current_player_name, other_player_name))
        .description(board_drawer.draw())
        .footer(CreateEmbedFooter::new(
            "Use `/move` to make a move using SAN. you can use `/move =` to offer draw.",
        ));

    match move_status {
        MoveStatus::GameStart => {
            let current_player = if is_our_turn {
                current_player_name
            } else {
                other_player_name
            };

            embed = embed
                .field("The game has started!", "Good luck!", true)
                .field("Current player", current_player, true);
        }
        MoveStatus::Check => {
            let other_player = if is_our_turn {
                other_player_name
            } else {
                current_player_name
            };

            embed = embed.field("Check!", other_player, true);
        }
        MoveStatus::Stalemate => {
            embed = embed.field("Stalemate", "The game is a stalemate", true);
        }
        MoveStatus::Checkmate => {
            let current_player = if is_our_turn {
                current_player_name
            } else {
                other_player_name
            };

            embed = embed.field("Checkmate!", current_player, true);
        }
        MoveStatus::Move(_) => {
            let current_player = if is_our_turn {
                current_player_name
            } else {
                other_player_name
            };

            embed = embed.field("Current player", current_player, true);
        }
        MoveStatus::DrawOffer(color) => {
            if our_color == color {
                embed = embed.field("Draw offer", "Waiting for opponent to accept", true);
            } else {
                embed = embed.field(
                    "Draw offer",
                    format!("{} offered a draw! use `/move =` to accept, or make any other move to decline", other_player_name),
                    true,
                );
            }
        }
        MoveStatus::Draw => {
            embed = embed.field("Draw!", "The game is a draw.", true);
        }
    }

    embed
}
