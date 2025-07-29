# SBDeaths
A SoulsBorne death counter. (With Linux Support!)

It reads your deaths directly from the game's memory and both prints it to the screen and writes it to a file.

#### Lil tidbits:
1. SBDeaths does not display the deaths on the main menu or
if your death count is 0 (when you first start a new game)
so don't freak out if you think it's not working at first.

2. You do NOT need to run this during your whole playthrough,
if you die X amount of times while SBDeaths is closed,
it will still read those deaths properly and pick them up when
SBDeaths is started again, so it can be used or not used at
any point throughout a playthrough.

## Streaming
SBDeaths writes your death count to SBDeaths.txt in the same folder as the program, you can add this file to your streaming software of choice.
#### OBS
Add a Text Source to your Scene, tick the "From File" option in it's properties, select the SBDeaths.txt file next to the program and viola!
#### Templating
The template.txt file will be used as the template for the final .txt file, all instances of "{}" will be replaced with the death count.

This is useful for streaming so you don't have to setup another Text Source just to get the look you want on your stream.

Examples: "{} Deaths" will become "10 Deaths", "Death Count: {}" will become "Death Count: 10".

## Game Compatibility
SBDeaths supports every mainline soulsborne game.

**Except for:**
1. Nightreign. (Not really a soulsborne)
2. Bloodborne. (Need to investigate)

## Game Updates
SBDeaths' game compatibility is very volatile, it will *probably* break on every single update pushed for a game.

Please report any issues you find [here](https://codeberg.org/misterj05/SBDeaths/issues), ESPECIALLY if there was a game update recently.

## Credits
[DSDeaths](https://github.com/Quidrex/DSDeaths) for inspiration and the initial offsets.

## Licensing
- SBDeaths is under the [GPLv3](https://codeberg.org/misterj05/SBDeaths/src/branch/main/LICENSE).
- The offsets.yaml file is under no license and is provided AS IS.
