# Untitled Survival Shooter
by Team Blueberry

## Team Members
* Advanced Topic Subteam 1: Multiplayer Networking
	* gcw33: Gordon Wyrostek
	* ams878: Amyia Singh
	* ifo2: Ifemi Olojo-Kosoko
	* zij15: Peter Ju

* Advanced Topic Subteam 2: Lighting Systems
	* mrk161: Matthew Kindja
	* haw102: Hao Wang
	* dva8: Daniel Aleardi
	* vlb56: Vladyslav Bordia

## Game Description

Our untitled survival shooter is a cooperative survival game in which
players progress through levels, fending off enemies and gathering resources necessary to survival.
Players must locate batteries to keep their lights on, or face certain death in darkness.

## Advanced Topic Description

### Multiplayer Networking

Our multiplayer networking implementation will synchronize the actions of up to 4 players across the network.
One client player will also assume the role of the host. Clients will transmit actions to the server, which
will resolve conflicts, simulate the world, and send state updates.

### Lighting Systems

Dynamic 2D lighting will be implemented to provide a more interesting game atmosphere and add an element of
resource/vision management to the mastery curve. Moving light sources, different light colors and color 
blending, and different types of light sources (point, spotlight) will be implemented.

## Midterm Goals

* Player 4-direction movement
* Player orients toward cursor
* Player shooting
* Enemies die when attacked
* Enemy type that moves in straight line toward player
* Players die when colliding with enemies
* Phong lighting implementation
* Point and cone lights illuminate enviornment withing radius
* Server and client connection

## Final Goals

* 15%: Players collide with surface
* 05%: Players open/close doors
* 30%: Complete level with fixed obstacles and enemy placements
* 05%: Win state: all players reach the end of the level
* 05%: Lose state: all players are simultaneously dead
* 10%: Signed-distance-field enabled shadows cast by 2D objects
* 10%: Player input data recording and server validation (lag compensation)

## Stretch Goals

* G-buffer enabled deferred rendering approach
* Enemies pathfind to players
