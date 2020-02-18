# Mullvad VPN app architecture

This document describes the code architecture and how everything fits together.

For security and anonymity properties, please see [security](security.md).

## Mullvad vs talpid

Explain the differences between these layers and why the distinction exists.
My thought was that after this section every aspect of the app is explained
under either the Mullvad or the Talpid header. So it's clear which part they
belong to. I yet don't know if this makes sense though.


## Mullvad part of daemon

### Frontend <-> system service communication

### Talking to api.mullvad.net

### Selecting relay and bridge servers

### Problem reports


## Talpid part of daemon

### Tunnel state machine

The tunnel state machine is the part of Talpid that coordinates the events for establishing a VPN
connection. It acts upon requests for establishing a secure VPN connection or for disconnecting an
already established connection and returning the system to its initial state. This involves also
using other parts of Talpid to configure the system so that the security principles are applied and
so that the connection works correctly without any further manual configuration necessary.

The tunnel state machine starts in an initial `Disconnected` state. On this state, the system is in
its default configuration (no changes are made) and no security principles are applied. When a
request is sent to the state machine to establish a connection, the state machine will progress
first into a `Connecting` state that will configure the system and setup a tunnel with a connection
to a VPN server. Once the configuration is complete and the connection is verified to be working,
the state machine then proceeds to a `Connected` state.

A request can be made to close the VPN connection. Such request will lead the state machine into
a `Disconnecting` state, which will close the connection to the VPN server and restore the system to
its original configuration. After the process is complete, the state machine returns to the
`Disconnected` state.

If an error occurs in the `Connecting` or `Connected` states, the state machine may proceed to an
`Error` state. In this state, the system is configured to block all connections to avoid leaking any
data after a request has been made to secure the system and while a request to disconnect hasn't
been sent. This implements the functionality that is sometimes described as a "kill-switch".

A high-level overview of the tunnel state machine can be seen in the diagram below:

                         
                    +--------------+   Request to connect    +------------+
      Start ------->| Disconnected +------------------------>| Connecting |
                    +--------------+                         +----+--+--+-+
                        ^                                      ^  |  ^  |                          
                        |           Will attempt to reconnect  |  |  |  |                          
                        |   .----------------------------------'  |  |  |                          
                        |   |                                     |  |  |                          
                        |   |                   .-----------------'  |  |                          
                        |   |                   | Unrecoverable      |  |                          
                        |   |                   |     error          |  |                          
                        |   |    Request to     V                    |  |                          
     System is restored |   |    disconnect +-------+                |  | Connection is configured 
       to its initial   |   |   .-----------+ Error +----------------'  |       and working        
        configuration   |   |   |           +-------+  Request to       |                          
                        |   |   |               ^       connect         |                          
                        |   |   |               |                       |                          
                        |   |   |  .------------'                       |                          
                        |   |   |  | Unrecoverable                      |                          
                        |   |   |  |  error while                       |                          
                        |   |   |  |  in connected                      |                          
                        |   |   V  |     state                          V                          
                     +--+---+------+-+                         +-----------+
                     | Disconnecting |<------------------------+ Connected |
                     +---------------+  Request to disconnect  +-----------+
                                          or unrecoverable
                                               error

### System DNS management

### Firewall integration

### Detecting device offline

### OpenVPN plugin and communication back to system service


## Frontends

### Desktop Electron app

### Android

### iOS

### CLI
