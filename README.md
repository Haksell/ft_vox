# ft_vox

## todo

- atlas offsets as int
- top and bottom different offsets
- more textures
- greedy meshing
- mipmap to avoid moire patterns
- use cubemap instead of panorama for skybox 
- better assets for everything

## mandatory

- [x] Your program will have to run in full screen mode. Reduce the default frame buffer is prohibited.
- [ ] You must be able to create a very large procedural world. For this project, user should be able to visit at least 16384\*256\*16384 cubes (256 is the height).
- [ ] Some cubes may be empty, others can have different types, like grass, ground, sand, etc...
- [x] Except for the empty cubes, they will all be opaque but will have their own textures.
- [ ] There should be hills.
- [ ] There should be mountains.
- [ ] There should be caves.
- [ ] There should be lakes.
- [x] This generation has to be determinist, which means the same seed will spawn the exact same map.
- [x] Each visited piece of terrain must be saved in the memory up to some limit you will set yourself and after which you can start deleting cubes from the memory.
- [x] In the open, minimal distance render will be 160 cubes
- [ ] Each cube must be textured, and you must have at least 2 different textures and 2 different types of cubes.
- [x] FoV must be 80 degrees.
- [ ] You will set up a skybox.
- [x] The mouse must be able to control the camera on 2 axis at least.
- [x] You will set 4 keys that will make the camera go forth, back, right and left according to its rotation.
- [x] Of course, the user must be able to keep going if he keeps pressing a key.
- [ ] The default camera speed should be 1 cube/second.
- [ ] There should be a key to multiply the speed to 20 cubes/second.

## bonus
- [ ] Have a render distance always higher than 14 (what?) and always have a smooth display.
- [ ] A fps counter is displayed.
- [ ] Render is smooth and doesn't freeze, event at x20 speed.
- [ ] Being able to delete blocks with the mouse.
- [ ] Having a lot of different biomes.

## push check

- [x] You're free to use your language, but keep an eye on its performances (Rust)
- [x] You must work directly with the APIs (WebGPU)
- [ ] You can use a library to load 3D objects and pictures, a windowing library and a mathematics library for your matrix/quaternions/vectors calculations
- [ ] The render should always be SMOOTH
- [ ] Any crash (Uncaught exception, segfault, abort ...) will disqualify you
- [ ] Your program must be able to run for hours without eating the whole memory or slowing down. Manage your RAM as well as VRAM very carefully.
