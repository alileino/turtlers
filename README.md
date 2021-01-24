API and a collection of programs for controlling CC turtles in Minecraft over a websocket using Rust. 

## Todo for API 1.0:
- [ ] Simulator for testing programs without minecraft
  - [ ] Move
  - [ ] Dig
  - [ ] Detect
- [ ] Antenna-based gps and miniprograms to find current coordinate easily
- [ ] Sign-based gps to read a sign to instantly determine gps.
  - Place a sign with known gps-coordinate. Turtle will stop execution in front of a sign. On startup it reads a sign in front of it and knows where it is.
  - Haphazardly notice signs and try to determine gps from them?
- [ ] Working "pretty good" non-optimal pathfinder
- [ ] Discovery-helper, that after all movements determines whether it is more useful to do some detecting of surroundings before continuing.
  - Goal: Program can inherit this program and execute everything and be sure that all possible detecting has been done before their execution. 
- [ ] ? Some kind of "features" to add to programs. 
   - For example, add GpsInit feature, that will ensure that gps is initialized in the beginning. Add discovery-feature, that ensures discoveries are made when possible.
   - Add pathfinding feature, which gives high-level API with commands such as "go to (5,2,4)" pathfinding will take care of generating actions until it is finished.



## Todo programs
- [ ] Tunnel mining script that 
