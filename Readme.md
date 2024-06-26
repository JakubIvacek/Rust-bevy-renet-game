# The idea....
The idea behind this project was to build a multiplayer game similiar to an old phone game called "Stack attack".

Our goal was to allow multiple players to cooperate on achieving the challange that this game presented - avoid falling boxes. And not just avoid them but also getting rid of them so that the whole screen does not fill up, players can get rid of the boxes by pushing them around and filling up a horizontal line making the whole line dissapear when it fills up. This mechanic is similiar to one in tetris. 


## requirements
Initially we set up requirements that were too hard to complete, here is the functionality that we did complete:


Networked movement 


Networked animation


Boxes are spawning on predefined positions


Moving boxes around


Box falling on players head results in loss

## High level overview
![image](https://github.com/Iaol12/rust_final_game/assets/113976963/36205962-eb3c-461e-9464-36224a7255f3)
- all game logic is handled on the server - authoritative server/client design
- there is no client side prediction implemented since for this sort of game low latency is not essential
- clients only send keys they pressed, recieve game state to show on screen, this is done by all client having a map of which local - client entity corresponds to which server entity that they just recieved data about.
 


## Design choices
At first we wanted to use https://github.com/gschup/bevy_ggrs this crate for networking which implements rollbacks, doing peer to peer networking for specified entities. We decided to use a server/client approach to make the server decide where boxes should spawn.




## what crates did we use?
- bevy - serves as a game engine
- bevy_kira_audio - audio crate for bevy
- bevy_renet - https://github.com/lucaspoffo/renet networking crate specifically designed for multiplayer games, sends messages through custom defined channels 
- fastrand,rand - for generating random spawn positions
  


## compiling and running the project
The actual multiplayer game is currently in the new_networking branch.

To run in development mode just do 

- for server `cargo run --bin server --features transport`
- for client `cargo run --bin client --features transport`
The server is running without opening a window to save resources

for release build - this will be optimised -> use `cargo build --release --bin server --features transport` and similarly with client
