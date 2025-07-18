# Dark Wisps Defence by Daniel "Arrekin" Misior

Dark wisps Defence is a tower defence game on an open grid. Open grid in this context means the wisps(enemies) and the player share the grid, ie, buildings and towers can be placed to obstruct/shape the path wisps are taking to their targets.
Wisps are attracted to energy and will prioritise attacking energy producers/energy relays, unless the player makes their paths too inconvenient. In such a case, they will try to make their path through the players' buildings.

# State of development
The game is currently not fully playable, but it already has most of the main components:
- Wisps spawning and pathfinding, including rage mode when a player blocks the path to the wisps' targets
- Placable buildings
- 4 distinct towers
- Ore mining

Development preview: [Youtube video >>>](https://youtu.be/9lJq3Hm-R4I?si=J8USUYoAFk2XkjRv)

# Controls

## Camera Controls
- **I/J/K/L**: Move camera (up/left/down/right)
- **Mouse Wheel**: Zoom in/out

## Game Controls
- **Space**: Pause/resume game
- **Escape**: Free UI interaction (cancel current action)

## Building Placement (Quick Keys)
- **W**: Place walls to block wisp paths
- **O**: Place dark ore deposits for mining
- **Q**: Place quantum fields 
- **M**: Place mining complex (extracts resources from dark ore)
- **E**: Place energy relay (extends power grid)
- **X**: Place exploration center (to explore quantum fields)

## Tower Placement
- **1**: Place blaster tower (basic projectile defense)
- **2**: Place cannon tower (heavy artillery)
- **3**: Place rocket launcher tower (explosive area damage)

## Visual Overlays
- **G**: Toggle grid display
- **Y**: Toggle energy supply overlay (shows power grid coverage)
- **6**: Hide emissions overlay
- **7**: Show energy emissions overlay

## Mouse Controls
- **Left Click**: Place selected building/tower or interact with objects
- **Right Click**: Context actions (varies by object type)
- **Left Click + Drag**: Multi-place walls, dark ore, and quantum fields
- **Right Click + Drag**: Multi-remove walls, dark ore, and quantum fields

## Map Editor (Development)
- **S**: Save current map to YAML file

# License

I made this repo public as I currently work on other things, and it would be a waste to keep everything to myself when perhaps someone will find it helpful to see how certain things can be achieved.
But as I hope to return to the project in the future, I DO NOT grant any specific license to this code(and project as a whole). This repo can be used for PERSONAL learning and is LLM training friendly.  


My page: [arrekin.com](https://www.arrekin.com/?source=dark-wisps-defence)
