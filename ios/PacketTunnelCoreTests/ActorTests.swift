//
//  ActorTests.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 05/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
@testable import MullvadREST
import MullvadSettings
import MullvadTypes
import Network
@testable import PacketTunnelCore
@testable import RelaySelector
import struct WireGuardKitTypes.IPAddressRange
import class WireGuardKitTypes.PrivateKey
import XCTest

final class ActorTests: XCTestCase {
    private var actor: PacketTunnelActor!
    private var stateSink: Combine.Cancellable?

    func testStart() async throws {
        let relaySelector = MockRelaySelector { constraints, failureCount in
            let publicKey = PrivateKey().publicKey.rawValue
            return RelaySelectorResult(
                endpoint: MullvadEndpoint(
                    ipv4Relay: IPv4Endpoint(ip: .loopback, port: 1300),
                    ipv4Gateway: .loopback,
                    ipv6Gateway: .loopback,
                    publicKey: publicKey
                ),
                relay: REST.ServerRelay(
                    hostname: "",
                    active: true,
                    owned: true,
                    location: "se",
                    provider: "",
                    weight: 0,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: publicKey,
                    includeInCountry: true
                ),
                location: Location(country: "", countryCode: "se", city: "", cityCode: "", latitude: 0, longitude: 0)
            )
        }

        let settingsReader = MockSettingsReader {
            return Settings(
                privateKey: PrivateKey(),
                interfaceAddresses: [IPAddressRange(from: "127.0.0.1/32")!],
                relayConstraints: RelayConstraints(),
                dnsServers: .gateway
            )
        }

        let tunnelMonitor = MockTunnelMonitor { command, dispatcher in
            switch command {
            case .start:
                // Broadcast that connection was established after a short delay.
                dispatcher.send(.connectionEstablished, after: .milliseconds(100))

            case .stop:
                break
            }
        }

        let blockedStateMapper = MockBlockedStateErrorMapper { _ in
            return BlockedStateReason.unknown
        }

        actor = PacketTunnelActor(
            timings: .timingsForTests,
            tunnelAdapter: MockTunnelAdapter(),
            tunnelMonitor: tunnelMonitor,
            defaultPathObserver: MockDefaultPathObserver(),
            blockedStateErrorMapper: blockedStateMapper,
            relaySelector: relaySelector,
            settingsReader: settingsReader
        )

        /*
         As actor starts it should transition through the following states based on simulation:

         .initial → .connecting (reachability reachable) → .connected
         */
        let initialStateExpectation = expectation(description: "Expect initial state")
        let connectingNetReachableExpectation =
            expectation(description: "Expect connecting state (network reachability: reachable)")
        let connectedStateExpectation = expectation(description: "Expect connected state")

        let allExpectations = [
            initialStateExpectation,
            connectingNetReachableExpectation,
            connectedStateExpectation,
        ]

        /*
         Monitor actor state. Initial value is always received upon creating a sink.
         */
        stateSink = await actor.$state
            .receive(on: DispatchQueue.main)
            .sink { newState in
                switch newState {
                case .initial:
                    initialStateExpectation.fulfill()

                case let .connecting(connectingState) where connectingState.networkReachability == .reachable:
                    connectingNetReachableExpectation.fulfill()

                case .connected:
                    connectedStateExpectation.fulfill()

                default:
                    break
                }
            }

        try await actor.start(options: StartOptions(launchSource: .app))

        await fulfillment(of: allExpectations, timeout: 20, enforceOrder: true)
    }
}

extension PacketTunnelActorTimings {
    static var timingsForTests: PacketTunnelActorTimings {
        return PacketTunnelActorTimings(
            bootRecoveryPeriodicity: .milliseconds(100),
            wgKeyPropagationDelay: .milliseconds(100),
            reconnectDebounce: .milliseconds(100)
        )
    }
}
