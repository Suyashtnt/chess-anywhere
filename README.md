# chess-anywhere

## Global Chess games

Are you on Discord, but your friend really wants to play a game of chess on Minecraft?
Well no fear, because chess-anywhere is here!

This is the most cursed project idea I've had in a while.

## Platforms

- [x] Discord (bot)
  - [x] Challenge other discord users
  - [ ] Challenge web users
  - [x] Link to web profiles
  - [x] Change username
- [/] Web (chess-anywhere.wobbl.in)
  - [/] REST Api
    - [ ] Stream chess games
    - [x] Play move
    - [ ] Get game state
    - [/] Store games in a database for future queries
    - [ ] FEN/PGN support
    - [ ] Change username
  - [ ] CapnProto API
    - [ ] Stream chess games
    - [ ] Play move
    - [ ] Get game state
    - [ ] Store games in a database for future queries
    - [ ] FEN/PGN support
    - [ ] Change Username
  - [ ] Challenge other web users
  - [ ] Challenge discord users
- [ ] Minecraft (possibly interact with <https://www.curseforge.com/minecraft/mc-mods/table-top-craft-fabric>)
- [ ] Slack (bot)
- [ ] Lichess (bot; requires timers to be implemented first. This could also be used to get the players initial ELO if it's the first game they've played)

## Resource list

- [x] Web authorization (Yes I am overengineering this)
  - [ ] <https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html>
    - [-] Make sure to always hash passwords
    - [x] Vague error messages + password reset
  - [ ] use <https://github.com/zxcvbn-ts/zxcvbn> to enforce secure passwords if we're storing them
  - [-] <https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html>
    - [-] <https://docs.rs/argon2/latest/argon2/>
  - [-] <https://cheatsheetseries.owasp.org/cheatsheets/Forgot_Password_Cheat_Sheet.html>
  - [ ] <https://cheatsheetseries.owasp.org/cheatsheets/Multifactor_Authentication_Cheat_Sheet.html>
  - [x] <https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html>
    - [x] seems that <https://github.com/maxcountryman/tower-sessions> knows what they're doing
