# Propulsion System

Dummy time-based propulsion system

## Overview 

The system consists of two main components running concurrently: 

1. **HTTP Server** (Actix-web)

    The server receives input from any number of clients asynchronously as a `post` request with a `json` body:

    - Commands the mission computer to fire the propulsion system in 5 seconds. 

        ```json
        {
            "countdown": 5
        }
        ```

    - Commands the mission computer to cancel the current countdown.

        ```json
        {
            "countdown": -1
        }
        ```

    The server validates and forwards these commands to the mission computer via channels (`tokio::sync::watch::channel`). These are ideal for this system as only the last sent value is retained in the channel.


2. **Mission Computer** (Background task)

    The mission computer will receive the command from the channel. It has a `counter_task` that will be executing a countdown timer accounting to timing information provided in the command. Anytime a new command is received, the mission computer will cancel the current `counter_task` and start a new one if nesseary (_if countdown is not -1_).



## Build & Run

```
cargo run
```

## Playground  

- Start a `5 second` countdown 

```cmd
curl -X POST http://localhost:8080/ \
  -H "Content-Type: application/json" \
  -d '{"countdown": 5}' \
  -w "\nHTTP Status: %{http_code}\n"
```

- Cancel current countdown

```cmd
curl -X POST http://localhost:8080/ \
  -H "Content-Type: application/json" \
  -d '{"countdown": -1}' \
  -w "\nHTTP Status: %{http_code}\n"
```

- Invalid command 

```cmd
curl -X POST http://localhost:8080/ \
  -H "Content-Type: application/json" \
  -d '{"countdown": -5}' \
  -w "\nHTTP Status: %{http_code}\n"
```