# 2023-UNIST-Rust-Minecraft

2023-UNIST-Rust-Minecraft is the material(lecture notes, examples and assignments) repository for learning Rust programming language and making simple Minecraft clone game that I'll teach the club 'HeXA' at UNIST in the winter of 2023.

## Contents

- Week 0 (1/11) [[Lecture]](./1%20-%20Lecture/230111%20-%20Rust%20Basic%20%2B%20Make%20Minecraft%2C%20Week%200.pdf)
  - Introduction
- Week 1 (1/18) [[Lecture]](./1%20-%20Lecture/230118%20-%20Rust%20Basic%20%2B%20Make%20Minecraft%2C%20Week%201.pdf) [[Assignment]](./3%20-%20Assignment/230118%20-%20Rust%20Basic%20%2B%20Make%20Minecraft%2C%20Week%201/) [[Solution]](./4%20-%20Solution/230118%20-%20Rust%20Basic%20%2B%20Make%20Minecraft%2C%20Week%201/)
  - A Tour of Rust, Part 1
  - Assignment #1
- Week 2 (1/25) [[Lecture]](./1%20-%20Lecture/230125%20-%20Rust%20Basic%20%2B%20Make%20Minecraft%2C%20Week%202.pdf) [[Assignment]](./3%20-%20Assignment/230125%20-%20Rust%20Basic%20%2B%20Make%20Minecraft%2C%20Week%202/) [[Solution]](./4%20-%20Solution/230125%20-%20Rust%20Basic%20%2B%20Make%20Minecraft%2C%20Week%202/)
  - Explain Solution of Assignment #1
  - A Tour of Rust, Part 2
  - Assignment #2
- Week 3 (2/1)
  - A Tour of Rust, Part 2
  - Assignment #2
- Week 4 (2/8)
  - Explain Solution of Assignment #2
- Week 5 (3/1) [[Lecture]](./1%20-%20Lecture/230301%20-%20Rust%20Basic%20%2B%20Make%20Minecraft%2C%20Week%205.pdf) [[Assignment]](./3%20-%20Assignment/230301%20-%20Rust%20Basic%20%2B%20Make%20Minecraft%2C%20Week%205/) [[Solution]](./4%20-%20Solution/230301%20-%20Rust%20Basic%20%2B%20Make%20Minecraft%2C%20Week%205/)
  - A Tour of Rust, Part 3
  - Assignment #3
- Week 6 (3/10) [[Lecture]](./1%20-%20Lecture/230310%20-%20Rust%20Basic%20%2B%20Make%20Minecraft%2C%20Week%206.pdf)
  - A Tour of Rust, Part 4
- Week 7 (3/15) [[Assignment]](./3%20-%20Assignment/230310%20-%20Rust%20Basic%20%2B%20Make%20Minecraft%2C%20Week%206/) [[Solution]](./4%20-%20Solution/230310%20-%20Rust%20Basic%20%2B%20Make%20Minecraft%2C%20Week%206/)
  - A Tour of Rust, Part 4
  - Assignment #4
- Week 8 (3/22)
  - A Tour of Rust, Part 5
  - Assignment #5
- Week 9 (5/10) [[Code]](./2%20-%20Example/230510%20-%20Rust%20Basic%20%2B%20Make%20Minecraft%2C%20Week%209/)
  - Making Minecraft, Part 1
    - Project Setup
    - Make a Simple Window using `gl` and `glfw`
      - The Main Loop
      - Double Buffering
      - Event Pooling
- Week 10 (7/18)
  - Making Minecraft, Part 2
    - Color Batch Rendering
      - Drawing a Quad
      - Make a Shader
        - Vertex Shader
        - Fragment Shader
    - Event Processing
- Week 11 (8/1)
  - Making Minecraft, Part 3
    - Texture Batch Rendering
      - Create a Texture
      - Texture Mapping  
    - Debugging
    - Show FPS
- Week X (1/24) [[Code]](./2%20-%20Example/240124%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Chunk Manager for Improving Performance
      - Reduce the draw call for regenerating the chunk
- Week X (1/31) [[Code]](./2%20-%20Example/240131%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Improve Performance
      - Block Face Culling
- Week X (2/16) [[Code]](./2%20-%20Example/240216%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Simplex Noise
    - Improve Performance
      - Use raw pointers for the VBO instead of `Vec`
- Week X (2/23) [[Code]](./2%20-%20Example/240223%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Support for different textures per block side
    - Code Refactoring
      - Move some type aliases to another files
- Week X (2/28) [[Code]](./2%20-%20Example/240228%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Transparent Blocks
    - Trees
    - Code Refactoring
      - Delete unused files
    - Improve Performance
      - Reinitialize the VBO each time a chunk has been invalidated
- Week X (3/6) [[Code]](./2%20-%20Example/240306%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - AABB Collision Detection
    - Fix bugs with the player movement
      - Jump key spam issue
- Week X (3/13) [[Code]](./2%20-%20Example/240313%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Fixed Timestep Physics
- Week X (3/21) [[Code]](./2%20-%20Example/240321%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Fix bugs with the player movement
      - Make player physics framerate independent
    - Code Refactoring
      - Replace magic numbers with constants
      - Extract all useful constants to a separate file
- Week X (4/7) [[Code]](./2%20-%20Example/240407%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Code Refactoring
      - Split main into multiple files
      - Move some functions for readability
      - Rename some symbols
- Week X (4/28) [[Code]](./2%20-%20Example/240428%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Code Refactoring
      - Replace triple for loops with `BlockIterators`
    - Add a crosshair in the center of the screen
      - Implement a base system for GUI
- Week X (5/4) [[Code]](./2%20-%20Example/240504%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Add Block Outline
      - The block that the player is looking at
    - Draw The Hotbar Texture at the bottom of the screen
    - Add Basic Lighting System
- Week X (5/12) [[Code]](./2%20-%20Example/240512%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Implement Hotbar and Inventory System
    - Refactor GUI rendering code
- Week X (5/19) [[Code]](./2%20-%20Example/240519%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Implement Ambient Occlusion
    - Fix lag spike when updating chunks
- Week X (6/1) [[Code]](./2%20-%20Example/240601%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Implement Interpolation System using trait `Interpolatable`
    - Apply interpolation to the physics system and FOV
    - Implement `Timer` for global time management and pause/resume
    - Implement flying trigger interval
- Week X (6/30) [[Code]](./2%20-%20Example/240630%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Implement Particle System
    - Spawn white particles when breaking blocks
- Week X (7/10) [[Code]](./2%20-%20Example/240710%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Replace texture atlas with array texture
- Week X (7/13) [[Code]](./2%20-%20Example/240713%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Improve Particle System
      - Use a texture atlas using array texture
      - Add a lifetime to particles
    - Improve performance
      - Batch all particles together into 1 draw call by interleaving the data
        - Don't use instancing because the texture coordinates must change for each particle
      - If the block is not at the edge of the chunk, then skip the chunk manager
    - Change code to spawn block breaking particles more minecraft-like
- Week X (7/20) [[Code]](./2%20-%20Example/240720%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Add new gameplay features
      - Sprinting on Left Control or Double Tap Forward
      - Sneaking on Left Shift
- Week X (7/31) [[Code]](./2%20-%20Example/240731%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Add FPS counter using `log` and `pretty_env_logger`
    - Improve Input System
      - Auto click to place or break blocks
      - Auto jump frequency throttle
    - Code Refactoring
      - Move code for handling input event to `PlayerProperties` and `PlayerPhysicsState`
- Week X (8/7) [[Code]](./2%20-%20Example/240807%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Implement proof of concept for block in hand
- Week X (8/9) [[Code]](./2%20-%20Example/240809%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Code Refactoring
      - Split input and player update into multiple systems (ECS) using `specs`
- Week X (8/12) [[Code]](./2%20-%20Example/240812%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Code Refactoring
      - Make player inventory and hand as ECS components
      - Insert particle systems, timer and texture pack as ECS resources
      - Remove unnecessary components and systems
      - Rename some types and fields to be more descriptive
- Week X (8/21) [[Code]](./2%20-%20Example/240821%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Code Refactoring
      - Put breaking and placing blocks in a separate ECS system
    - Animate main hand when switching items in the hotbar
    - Add mouse event for middle click to pick up blocks
- Week X (9/25) [[Code]](./2%20-%20Example/240925%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Code Refactoring
      - Move all rendering into ECS systems
      - Put chunks into chunk columns
- Week X (10/4) [[Code]](./2%20-%20Example/241004%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Code Refactoring
      - Replace dynamic allocated structures with fixed-size arrays
      - Use `BitVec` for block face visibility
      - Add some timers for performance profiling
- Week X (11/6) [[Code]](./2%20-%20Example/241106%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Code Refactoring
      - Replace `HashMap` with `Vec` for storing ambient occulsion patterns
      - Store active faces and ambient occlusion data in memory in order to avoid recalculations
      - Recalculate them only for the changed surfaces, not the entire chunk
- Week X (11/20) [[Code]](./2%20-%20Example/241120%20-%20Rust%20Basic%20+%20Make%20Minecraft,%20Week%20X/)
  - Making Minecraft, Part X
    - Load one randomized chunk per frame
    - Implement proof of concept for infinite chunk generation

## References

- Beginner
  * [The Rust Programming Language](https://doc.rust-lang.org/book/)
  * [Rustlings](https://github.com/rust-lang/rustlings/)
  * [Rust By Example](https://doc.rust-lang.org/stable/rust-by-example/)
  * [Rust-101 by Ralf Jung](https://www.ralfj.de/projects/rust-101/main.html)
  * [Exercism - Rust](https://exercism.org/tracks/rust)
  * [Book: The Rust Programming Language](http://www.yes24.com/Product/Goods/83075894)
  * [Book: Rust in Action](https://www.manning.com/books/rust-in-action)
  * [Book: Programming Rust](https://www.oreilly.com/library/view/programming-rust-2nd/9781492052586/)
- Intermediate
  * [The Standard Library](https://doc.rust-lang.org/std/index.html)
  * [The Edition Guide](https://doc.rust-lang.org/edition-guide/index.html)
  * [The Cargo Book](https://doc.rust-lang.org/cargo/index.html)
  * [The rustdoc Book](https://doc.rust-lang.org/rustdoc/index.html)
  * [The rustc Book](https://doc.rust-lang.org/rustc/index.html)
  * [Book: Concurrent Programming](http://www.yes24.com/Product/Goods/108570426)
  * [Book: Rust for Rustaceans](https://rust-for-rustaceans.com/)
- Advanced
  * [The Rust Reference](https://doc.rust-lang.org/reference/index.html)
  * [The Rustonomicon](https://doc.rust-lang.org/nomicon/index.html)
  * [The Rust Unstable Book](https://doc.rust-lang.org/nightly/unstable-book/index.html)

## How To Contribute

Contributions are always welcome, either reporting issues/bugs or forking the repository and then issuing pull requests when you have completed some additional coding that you feel will be beneficial to the main project. If you are interested in contributing in a more dedicated capacity, then please contact me.

## Contact

You can contact me via e-mail (utilForever at gmail.com). I am always happy to answer questions or help with any issues you might have, and please be sure to share any additional work or your creations with me, I love seeing what other people are making.

## License

<img align="right" src="http://opensource.org/trademarks/opensource/OSI-Approved-License-100x137.png">

The class is licensed under the [MIT License](http://opensource.org/licenses/MIT):

Copyright &copy; 2023 [Chris Ohk](http://www.github.com/utilForever).

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
