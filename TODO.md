# Abstraction

- Move drawing logic into something else
- Generic for Renderer
- Traits for Input / State
- Entity Manager for Client / Server



# Level 

- Split up to support drawable
- Better serialization 
    - LevelConfig object with for raw serialization

# Client

- Split up game code 
- Integrate network code into game/net.rs and remove a lot of cruft

# Server 

- Send tick interpolation config to client


# Entities

- Support state rewinding


# General

- Game and Entity Events

