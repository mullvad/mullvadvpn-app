//
//  TrafficGenerator.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-06-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Network
import XCTest

class TrafficGenerator {
    let destinationHost: String
    let port: Int
    var connection: NWConnection
    let dispatchQueue = DispatchQueue(label: "TrafficGeneratorDispatchQueue")
    var sendDataTimer: DispatchSourceTimer

    init(destinationHost: String, port: Int) {
        self.destinationHost = destinationHost
        self.port = port

        sendDataTimer = DispatchSource.makeTimerSource(queue: dispatchQueue)
        let params = NWParameters.udp
        connection = NWConnection(
            host: NWEndpoint.Host(destinationHost),
            port: NWEndpoint.Port(integerLiteral: UInt16(port)),
            using: params
        )
        setupOtherHandlers()
    }

    func reconnect() {
        print("Attempting to reconnect")
        connection.forceCancel()

        connection = createConnection()
        setupConnection()
        setupOtherHandlers()
    }

    func createConnection() -> NWConnection {
        let params = NWParameters.udp
        return NWConnection(
            host: NWEndpoint.Host(destinationHost),
            port: NWEndpoint.Port(integerLiteral: UInt16(port)),
            using: params
        )
    }

    func setupOtherHandlers() {
        connection.pathUpdateHandler = { newPath in
            let availableInterfaces = newPath.availableInterfaces.map { $0.customDebugDescription }
            let availableGateways = newPath.gateways.map { $0.customDebugDescription }

            print("New interfaces available: \(availableInterfaces)")
            print("New gateways available: \(availableGateways)")
        }

        connection.viabilityUpdateHandler = { newViability in
            print("Connection is viable: \(newViability)")
        }

        connection.betterPathUpdateHandler = { betterPathAvailable in
            print("A better path is available: \(betterPathAvailable)")
        }
    }

    func setupConnection() {
        print("Setting up connection...")
        let doneAttemptingConnectExpecation = XCTestExpectation(description: "Done attemping to connect")

        connection.stateUpdateHandler = { state in
            switch state {
            case .ready:
                print("Ready")
                self.sendDataTimer.resume()
                doneAttemptingConnectExpecation.fulfill()
            case let .failed(error):
                print("Failed to connect: \(error)")
                self.sendDataTimer.cancel()
                self.reconnect()
            case .preparing:
                print("Preparing connection...")
            case .setup:
                print("Setting upp connection...")
            case let .waiting(error):
                print("Waiting to connect: \(error)")
            case .cancelled:
                self.sendDataTimer.suspend()
                print("Cancelled connection")
                self.reconnect()
            default:
                break
            }
        }
        connection.start(queue: dispatchQueue)

        XCTWaiter().wait(for: [doneAttemptingConnectExpecation], timeout: 10.0)
    }

    func stopConnection() {
        connection.stateUpdateHandler = { @Sendable _ in }
        connection.cancel()
    }

    public func startGeneratingUDPTraffic(interval: TimeInterval) {
        setupConnection()
        sendDataTimer.schedule(deadline: .now(), repeating: interval)

        sendDataTimer.setEventHandler {
            let data = Data("dGhpcyBpcyBqdXN0IHNvbWUgZHVtbXkgZGF0YSB0aGlzIGlzIGp1c3Qgc29tZSBkdW".utf8)

            print("Attempting to send data...")

            if self.connection.state != .ready {
                print("Not connected, won't send data")
            } else {
                self.connection.send(content: data, completion: .contentProcessed { error in
                    if let error = error {
                        print("Failed to send data: \(error)")
                    } else {
                        print("Data sent")
                    }
                })
            }
        }

        sendDataTimer.activate()
    }

    public func stopGeneratingUDPTraffic() {
        sendDataTimer.setEventHandler(handler: {})
        sendDataTimer.cancel()
        stopConnection()
    }
}

extension NWInterface {
    var customDebugDescription: String {
        "type: \(type) name: \(self.name) index: \(index)"
    }
}

extension NWInterface.InterfaceType: @retroactive CustomDebugStringConvertible {
    public var debugDescription: String {
        switch self {
        case .cellular: "Cellular"
        case .loopback: "Loopback"
        case .other: "Other"
        case .wifi: "Wifi"
        case .wiredEthernet: "Wired Ethernet"
        @unknown default: "Unknown interface type"
        }
    }
}

extension NWEndpoint {
    var customDebugDescription: String {
        switch self {
        case let .hostPort(host, port): "host: \(host.customDebugDescription) port: \(port)"
        case let .opaque(endpoint): "opaque: \(endpoint.description)"
        case let .url(url): "url: \(url)"
        case let .service(
            name,
            type,
            domain,
            interface
        ): "service named:\(name), type:\(type), domain:\(domain), interface:\(interface?.customDebugDescription ?? "[No interface]")"
        case let .unix(path): "unix: \(path)"
        @unknown default: "Unknown NWEndpoint type"
        }
    }
}

extension NWEndpoint.Host {
    var customDebugDescription: String {
        switch self {
        case let .ipv4(IPv4Address): "IPv4: \(IPv4Address)"
        case let .ipv6(IPv6Address): "IPv6: \(IPv6Address)"
        case let .name(name, interface): "named: \(name), \(interface?.customDebugDescription ?? "[No interface]")"
        @unknown default: "Unknown host"
        }
    }
}
