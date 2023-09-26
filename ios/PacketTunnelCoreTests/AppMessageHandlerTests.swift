//
//  AppMessageHandlerTests.swift
//  PacketTunnelCoreTests
//
//  Created by Jon Petersson on 2023-09-28.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
@testable import MullvadREST
import MullvadTypes
import PacketTunnelCore
import RelaySelector
import XCTest

final class AppMessageHandlerTests: XCTestCase {
    private var stateSink: Combine.Cancellable?

    private lazy var urlRequestProxy: URLRequestProxy = {
        let transportProvider = REST.AnyTransportProvider {
            AnyTransport { Response(delay: 1, statusCode: 200, value: TimeResponse(dateTime: Date())) }
        }

        return URLRequestProxy(
            dispatchQueue: DispatchQueue(label: "AppMessageHandlerTests"),
            transportProvider: transportProvider
        )
    }()

    override func tearDown() async throws {
        stateSink?.cancel()
    }

    func testHandleAppMessageForTunnelStatus() async throws {
        let actor = PacketTunnelActor.mock()
        let appMessageHandler = createAppMessageHandler(actor: actor)
        let reconnectingExpectation = expectation(description: "Expect reconnecting state")

        actor.start(options: StartOptions(launchSource: .app))
        actor.setErrorState(reason: .deviceRevoked)

        stateSink = await actor.$state
            .receive(on: DispatchQueue.main)
            .sink { newState in
                switch newState {
                case .error:
                    Task {
                        let reply = try? await appMessageHandler.handleAppMessage(
                            TunnelProviderMessage.getTunnelStatus.encode()
                        )
                        let tunnelStatus = try? TunnelProviderReply<PacketTunnelStatus>(messageData: reply!)

                        XCTAssertEqual(tunnelStatus?.value.blockedStateReason, .deviceRevoked)
                        reconnectingExpectation.fulfill()
                    }
                default:
                    break
                }
            }

        await fulfillment(of: [reconnectingExpectation], timeout: 1)
    }

    func testHandleAppMessageForURLRequest() async throws {
        let actor = PacketTunnelActor.mock()
        let appMessageHandler = createAppMessageHandler(actor: actor)

        actor.start(options: StartOptions(launchSource: .app))

        let url = URL(string: "localhost")!
        let urlRequest = ProxyURLRequest(
            id: UUID(),
            urlRequest: URLRequest(url: url)
        )!

        let reply = try await appMessageHandler
            .handleAppMessage(TunnelProviderMessage.sendURLRequest(urlRequest).encode())
        let urlResponse = try TunnelProviderReply<ProxyURLResponse>(messageData: reply!)

        XCTAssertEqual(urlResponse.value.response?.url, url)
    }

    func testHandleAppMessageForReconnectTunnel() async throws {
        let actor = PacketTunnelActor.mock()
        let appMessageHandler = createAppMessageHandler(actor: actor)
        let reconnectingExpectation = expectation(description: "Expect reconnecting state")

        actor.start(options: StartOptions(launchSource: .app))

        let relayConstraints = RelayConstraints(location: .only(.hostname("se", "sto", "se6-wireguard")))
        let selectorResult = try? RelaySelector.evaluate(
            relays: ServerRelaysResponseStubs.sampleRelays,
            constraints: relayConstraints,
            numberOfFailedAttempts: 0
        )

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.reconnectTunnel(selectorResult).encode()
        )

        stateSink = await actor.$state
            .receive(on: DispatchQueue.main)
            .sink { newState in
                print(newState)
                switch newState {
                case let .reconnecting(state):
                    XCTAssertEqual(state.selectedRelay.relay, selectorResult?.relay)
                    reconnectingExpectation.fulfill()
                default:
                    break
                }
            }

        await fulfillment(of: [reconnectingExpectation], timeout: 1)
    }

    func testHandleAppMessageForKeyRotation() async throws {
        let actor = PacketTunnelActor.mock()
        let appMessageHandler = createAppMessageHandler(actor: actor)
        let connectedExpectation = expectation(description: "Expect connecting state")

        actor.start(options: StartOptions(launchSource: .app))

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.privateKeyRotation.encode()
        )

        stateSink = await actor.$state
            .receive(on: DispatchQueue.main)
            .sink { newState in
                switch newState {
                case let .connected(state):
                    XCTAssertNotNil(state.lastKeyRotation)
                    connectedExpectation.fulfill()
                default:
                    break
                }
            }

        await fulfillment(of: [connectedExpectation], timeout: 1)
    }
}

extension AppMessageHandlerTests {
    func createAppMessageHandler(actor: PacketTunnelActor) -> AppMessageHandler {
        return AppMessageHandler(
            packetTunnelActor: actor,
            urlRequestProxy: urlRequestProxy
        )
    }
}
