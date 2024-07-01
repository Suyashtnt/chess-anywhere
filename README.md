# chess-anywhere

## Global Chess games

Are you on Discord, but your friend really wants to play a game of chess on Minecraft?
Well no fear, because chess-anywhere is here!

This is the most cursed project idea I've had in a while.

## Platforms

- [x] Discord (bot)
- [ ] Minecraft (possibly interact with <https://www.curseforge.com/minecraft/mc-mods/table-top-craft-fabric>)
- [ ] Slack (bot)
- [ ] Web (chess.wobbl.in)
- [ ] Lichess (bot; requires timers to be implemented first. This could also be used to get the players initial ELO if it's the first game they've played)

## Resource list

- [ ] Web authorization (Yes I am overengineering this)
  - [ ] <https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html>
    - [ ] Make sure to always hash passwords
    - [ ] Vague error messages + password reset
  - [ ] use <https://github.com/zxcvbn-ts/zxcvbn> to enforce secure passwords if we're storing them
  - [ ] <https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html>
    - [ ] <https://docs.rs/argon2/latest/argon2/>
  - [ ] <https://cheatsheetseries.owasp.org/cheatsheets/Forgot_Password_Cheat_Sheet.html>
  - [ ] <https://cheatsheetseries.owasp.org/cheatsheets/Multifactor_Authentication_Cheat_Sheet.html>
  - [ ] <https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html>
    - [ ] seems that <https://github.com/maxcountryman/tower-sessions> knows what they're doing
