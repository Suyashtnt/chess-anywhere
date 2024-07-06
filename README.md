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
  - [/] Store games in a database for future queries
  - [/] REST API
    - [ ] Challenge other web users
    - [ ] Challenge discord users
    - [ ] Stream chess games
    - [x] Play move
    - [ ] Get game state
    - [ ] FEN/PGN support
    - [ ] Change username
    - [ ] Manage account connections
  - [ ] CapnProto API
    - [ ] Challenge other web users
    - [ ] Challenge discord users
    - [ ] Stream chess games
    - [ ] Play move
    - [ ] Get game state
    - [ ] FEN/PGN support
    - [ ] Change Username
    - [ ] Manage account connections
- [ ] Minecraft (possibly interact with <https://www.curseforge.com/minecraft/mc-mods/table-top-craft-fabric>)
  - [ ] Probably require logging into an existing account rather than make one in-service
- [ ] Slack (bot)
- [ ] Lichess (bot; requires timers to be implemented first which might be impossible based on discord and slack. This could also be used to get the players initial ELO if it's the first game they've played)
  - [ ] Requires logging into an existing account rather than make one in-service

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
