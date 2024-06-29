use drawer::BoardDrawer;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use shakmaty::Board;

use crate::chess::MoveStatus;

mod drawer;

pub fn create_board_embed(
    current_player_name: &str,
    other_player_name: &str,
    board: &Board,
    move_status: &MoveStatus,
    is_our_turn: bool,
) -> CreateEmbed {
    let board_drawer = BoardDrawer::new(board, move_status);

    let mut embed = CreateEmbed::default()
        .title(format!("{} vs {}", current_player_name, other_player_name))
        .description(board_drawer.draw())
        .footer(CreateEmbedFooter::new(
            "Run /move to make a move on Discord using SAN",
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
    }

    embed
}
