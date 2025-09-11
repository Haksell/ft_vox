# ft_vox

## todo ft_vox

- block deletion
- mipmap to avoid moire patterns
- more cave openings
- random rare textures in caves
- args (seed, fullscreen, display fps, fast boost, ...)

## mandatory ft_vox

- [x] Your program will have to run in full screen mode. Reduce the default frame buffer is prohibited.
- [x] You must be able to create a very large procedural world. For this project, user should be able to visit at least 16384\*256\*16384 cubes (256 is the height).
- [x] Some cubes may be empty, others can have different types, like grass, ground, sand, etc...
- [x] Except for the empty cubes, they will all be opaque but will have their own textures.
- [ ] There should be hills.
- [ ] There should be mountains.
- [ ] There should be caves.
- [ ] There should be lakes.
- [x] This generation has to be determinist, which means the same seed will spawn the exact same map.
- [x] Each visited piece of terrain must be saved in the memory up to some limit you will set yourself and after which you can start deleting cubes from the memory.
- [x] In the open, minimal distance render will be 10 chunks
- [x] Each cube must be textured, and you must have at least 2 different textures and 2 different types of cubes.
- [x] FoV must be 80 degrees.
- [x] You will set up a skybox.
- [x] The mouse must be able to control the camera on 2 axis at least.
- [x] You will set 4 keys that will make the camera go forth, back, right and left according to its rotation.
- [x] Of course, the user must be able to keep going if he keeps pressing a key.
- [x] There should be a default camera speed and an accelerated camera speed.

## bonus ft_vox

- [x] Have a render distance always higher than 14 chunks and always have a smooth display.
- [x] A FPS counter is displayed.
- [ ] Render is smooth and doesn't freeze, at x20 speed.
- [ ] Being able to delete blocks with the mouse.
- [ ] Having a lot of different biomes.

## push check ft_vox

- [x] You're free to use your language, but keep an eye on its performances (Rust)
- [x] You must work directly with the APIs (WebGPU)
- [x] You can use a library to load 3D objects and pictures (image)
- [x] You can use a windowing library (winit)
- [x] You can use a mathematics library for your calculations (glam)
- [ ] The render should always be SMOOTH
- [ ] Any crash (uncaught exception, segfault, abort ...) will disqualify you
- [ ] Your program must be able to run for hours without eating the whole memory or slowing down. Manage your RAM as well as VRAM very carefully.
- [ ] Full screen mode by default
- [ ] Camera speeds should be 1.0 and 20.0

## todo ft_minecraft

- collisions
- animations (water, lava, ...)
- transparency (water, ...)
- launch animation (camera drops to block at (0, 0) with proper gravity)
- block about to be deleted should look a bit darker
- only delete block if raycast at start and end of click touch the same block 
- frustum culling during face generation instead of after checking chunks
- use cubemap instead of panorama for skybox
- compute shader for world generation
- ray tracing/casting/marching/whatever (https://www.youtube.com/watch?v=gXSHtBZFxEI, https://www.youtube.com/watch?v=P2bGF6GPmfc)
- generative skyboxes
- transparent water
- make our own pixel art
- sun(s) (from `scratch` branch) (text shadow depends on it)
- fix dead pixels
- anisotropic filtering
- port to wasm
