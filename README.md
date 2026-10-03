# SBDeaths
A cross-platform (Windows, Linux) soulsborne death counter.

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
~~SBDeaths' game compatibility is volatile, it will *probably* break on every single update pushed for a game.~~

SBDeaths' game compatibility isn't as volatile as it once was. Since the introduction of signature scanning,
it is less likely for a game update to break SBDeaths. However, it is still possible, so if you have any
issues please report them [here](https://codeberg.org/misterj05/SBDeaths/issues), ESPECIALLY if there was a game
update recently.

## Repo Mirrors
> [!IMPORTANT]
> This repo is hosted on [Codeberg](https://codeberg.org/misterj05/SBDeaths)
> and is mirrored onto other forges for discoverability.
>
> Issues will get ignored on forges other than Codeberg.
> PRs might still get merged off-platform via git with authorship preserved.
> Discussion boards will likely see activity as Codeberg does not have such a feature yet.
> Overall I would just prefer to work on Codeberg if possible.

## Credits
[DSDeaths](https://github.com/Quidrex/DSDeaths) for inspiration and the initial offsets.

## Licensing
SBDeaths is licensed under the [GPL-3.0-or-later](https://www.gnu.org/licenses/gpl-3.0.txt).
