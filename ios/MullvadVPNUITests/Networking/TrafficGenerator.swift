//
//  TrafficGenerator.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-06-25.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Network
import XCTest

class TrafficGenerator {
    let destinationHost: String
    let port: Int
    let connection: NWConnection
    let dispatchQueue = DispatchQueue(label: "TrafficGeneratorDispatchQueue", qos: .unspecified)
    var timer: DispatchSourceTimer

    init(destinationHost: String, port: Int) {
        self.destinationHost = destinationHost
        self.port = port
        connection = NWConnection(
            host: NWEndpoint.Host(destinationHost),
            port: NWEndpoint.Port(integerLiteral: UInt16(port)),
            using: .udp
        )

        timer = DispatchSource.makeTimerSource(queue: dispatchQueue)

        connect()
    }

    func connect() {
        let doneAttemptingConnectExpecation = XCTestExpectation(description: "Done attemping to connect")

        connection.stateUpdateHandler = { state in
            switch state {
            case .ready:
                print("Ready")
                doneAttemptingConnectExpecation.fulfill()
            case let .failed(error):
                print("Failed to connect: \(error)")
                doneAttemptingConnectExpecation.fulfill()
            default:
                break
            }
        }

        connection.start(queue: dispatchQueue)

        XCTWaiter().wait(for: [doneAttemptingConnectExpecation], timeout: 10.0)
    }

    public func startGeneratingUDPTraffic(interval: TimeInterval) {
        timer.schedule(deadline: .now(), repeating: interval)

        timer.setEventHandler {
            let data = "dGhpcyBpcyBqdXN0IHNvbWUgZHVtbXkgZGF0YSB0aGlzIGlzIGp1c3Qgc29tZSBkdW".data(using: .utf8)
            self.connection.send(content: data, completion: .contentProcessed { error in
                if let error = error {
                    print("Failed to send data: \(error)")
                } else {
                    print("Data sent")
                }
            })
        }

        timer.activate()
    }

    public func stopGeneratingUDPTraffic() {
        timer.cancel()
    }
}
