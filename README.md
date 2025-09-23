# ft_minecaft

## todo

- fix trackpad issue on laptop
- fix vsync issue on lenovo
- fix fullscreen broken animation (load textures before window creation?)
- `build.rs` for shaders?
- true gpu sampler mipmaps
- anisotropic filtering
- animations (water, lava, ...)
- menger sponge block (requires transparency)
- launch animation (camera drops to block at (0, 0) with proper gravity)
- block about to be deleted should look a bit darker (hard given chunk tree structure)
- frustum culling during face generation instead of after checking chunks
- multithreading for mesh generation
- compute shader for world generation
- ray tracing/casting/marching/whatever (https://www.youtube.com/watch?v=gXSHtBZFxEI, https://www.youtube.com/watch?v=P2bGF6GPmfc)
- 64-bit brickmap instead of naive tree
- use cubemap instead of panorama for skybox
- generative skyboxes
- make our own pixel art
- sun(s) (from `scratch` branch) (text shadow depends on it)
- fix dead pixels (should be free with ray marching)
- port to wasm
- planets? (https://www.bowerbyte.com/posts/blocky-planet/)
- rings outside some planets
- resizing the window should properly update camera
- particles when removing/placing blocks
- malbolge interpreter with voxels (liam tkt)

## mandatory

- [x] the world must be generated on demand
- [x] you should be able to navigate through at least 5,000,000 cubes on the xz (xy for us) plane
- [ ] a minimum of 5 unique biomes is required: (e.g., mountain, desert, canyon, swamp, sequoia forest, island, savanna, etc.)
- [ ] each biome should have unique geography, elevation, vegetation, and distinct characteristics that make them feel truly unique
- [ ] biomes should transition smoothly and naturally without abrupt change
- [ ] there should be small plants, flowers, and mushrooms scattered throughout the world
- [ ] there should be procedurally generated trees. each tree should be generated with varying parameters like shape, height, width, and leaf density to ensure uniqueness
- [ ] different biomes should have different vegetation
- [ ] there must be lakes and rivers meandering across the world, as well as natural cave entrances visible from the surface
- [ ] these caves should feature realistic formations (wormhole style) and contain clusters of rare ores like gold and diamonds, not just simple noise-based distribution
- [ ] monsters (like creepers or zombies) should spawn and chase you when you get close
- [ ] 3d clouds should float across the world. they can either be represented as blocks (purely visual with no interaction) or as shaders
- [ ] you should be able to pick up blocks after destroying them and place them wherever you want
- [ ] destroyed or placed blocks must be persistent
- [ ] minimum render distance is 260 (16.25 blocks)
- [ ] you may use a sky shader instead of a skybox if desired
- [ ] directional lighting
- [ ] shadows
- [ ] screen space ambient occlusion (might not make sense for ray marching)
- [ ] transparent water surfaces
- [ ] far distance fog for better immersion
- [ ] the keyboard should allow forward, backward, strafe right, and strafe left movement, relative to the camera's orientation
- [ ] there should be a simple gravity system that handles block collisions (excluding water)
- [ ] the ability to swim and dive, with optional slowed movement underwater
- [ ] visual rendering adaptations for underwater and cloud exploration (color filters, reduced visibility)
- [ ] basic animations for walking and attacking, minecraft-like in simplicity
- [ ] there should be collisions. (impossible to go inside a block)
- [ ] ability to jump
- [ ] ability to sprint (2x speed)
- [ ] toggleable fly-mode (disables gravity, 20x speed)
- [x] 360-degree mouse control on the y-axis, with the ability to look up and down
- [ ] each biome should have its own unique ambient music, with smooth transitions between them
- [ ] both players and monsters must have sounds for actions like walking, attacking, and swimming
- [ ] the sound volume should dynamically adjust based on distance from the source
- [ ] your server should allow at least four players to join simultaneously
- [ ] players should be visible in the world, performing any actions such as walking, attacking, destroying blocks, and even getting killed by monsters
- [ ] all modifications to the world (block placement, block destruction) must be synchronized across all players and persistent even after reloading
- [ ] entity states (like monsters) should also be synchronized
- [ ] you are free to decide how the server-side is managed, either procedurally generate the world on the server and dispatch it to clients, or have each client generate the world and synchronize modifications with other clients
- [ ] fps, triangles (impossible if raymarching), cubes and chunk counts must be displayed on-screen with a key toggle
- [ ] a list of all connected players should also be available with a key toggle

## suggested bonus

- [ ] procedurally generated villages
- [ ] crafting system
- [ ] realistic water simulation (dynamic flow and spreading)
- [ ] growing plants (from seeds to maturity)
- [ ] a bow and arrow system similar to minecraft
- [ ] nether portal that teleports you to another dimension. (or planet?)
- [ ] cross-platform support (windows, mac, linux)
- [ ] stereo sound implementation. (or ray-traced sound?)
- [ ] an online map interface (like minecraft's dynmap)

## additional bonus

- [ ] planets
- [ ] sun(s)
- [ ] volcano with eruptions
- [ ] port and host on the web
- [ ] hollow blocks (e.g. menger sponge, requires transparency)
- [ ] generative skyboxes

## push check

- [x] you're free to use your language, but keep an eye on its performances (rust)
- [x] you must work directly with the api (webgpu)
- [x] you can use a library to load 3d objects and pictures (image)
- [x] you can use a windowing library (winit)
- [x] you can use a mathematics library for your calculations (glam)
- [x] using pre-built libraries for terrain or biome generation is strictly forbidden. you
must implement everything from scratch
- [ ] the render should always be smooth with a minimum of 60 fps
- [ ] any crash (uncaught exception, segfault, abort ...) will disqualify you
- [ ] your program must be able to run for hours without eating the whole memory or slowing down. manage your ram as well as vram very carefully
- [ ] your program must run at 1080p or higher. reducing the framebuffer resolution is
not allowed
- [ ] if your assets exceed 42 mb, you must provide a script to download or manually copy
them
- [ ] camera speeds should be 1.0 (walking), 2.0 (sprinting) and 20.0 (flying)
- [ ] ctrl+f todo
- [ ] no warnings
- [ ] delete school subject readme sections and write proper readme
