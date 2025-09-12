# ft_minecaft

## todo ft_minecraft

- volcanos
- true gpu sampler mipmaps
- anisotropic filtering
- collisions
- animations (water, lava, ...)
- transparency (water, ...)
- launch animation (camera drops to block at (0, 0) with proper gravity)
- block about to be deleted should look a bit darker (hard given chunk tree structure)
- frustum culling during face generation instead of after checking chunks
- use cubemap instead of panorama for skybox
- multithreading for mesh generation
- compute shader for world generation
- ray tracing/casting/marching/whatever (https://www.youtube.com/watch?v=gXSHtBZFxEI, https://www.youtube.com/watch?v=P2bGF6GPmfc)
- generative skyboxes
- make our own pixel art
- sun(s) (from `scratch` branch) (text shadow depends on it)
- fix dead pixels (should be automatic with ray marching)
- port to wasm
