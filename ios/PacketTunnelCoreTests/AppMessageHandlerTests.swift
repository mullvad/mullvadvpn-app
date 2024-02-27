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
import XCTest

final class AppMessageHandlerTests: XCTestCase {
    func testHandleAppMessageForSendURLRequest() async throws {
        let sendRequestExpectation = expectation(description: "Expect sending request")

        let urlRequestProxy = URLRequestProxyStub(sendRequestExpectation: sendRequestExpectation)
        let appMessageHandler = createAppMessageHandler(urlRequestProxy: urlRequestProxy)

        let url = URL(string: "localhost")!
        let urlRequest = ProxyURLRequest(
            id: UUID(),
            urlRequest: URLRequest(url: url)
        )!

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.sendURLRequest(urlRequest).encode()
        )

        await fulfillment(of: [sendRequestExpectation], timeout: 1)
    }

    func testHandleAppMessageForCancelURLRequest() async throws {
        let cancelRequestExpectation = expectation(description: "Expect cancelling request")

        let urlRequestProxy = URLRequestProxyStub(cancelRequestExpectation: cancelRequestExpectation)
        let appMessageHandler = createAppMessageHandler(urlRequestProxy: urlRequestProxy)

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.cancelURLRequest(UUID()).encode()
        )

        await fulfillment(of: [cancelRequestExpectation], timeout: 1)
    }

    func testHandleAppMessageForTunnelStatus() async throws {
        let stateExpectation = expectation(description: "Expect getting state")

        let actor = PacketTunnelActorStub(stateExpectation: stateExpectation)
        let appMessageHandler = createAppMessageHandler(actor: actor)

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.getTunnelStatus.encode()
        )

        await fulfillment(of: [stateExpectation], timeout: 1)
    }

    func testHandleAppMessageForKeyRotation() async throws {
        let keyRotationExpectation = expectation(description: "Expect key rotation")

        let actor = PacketTunnelActorStub(keyRotationExpectation: keyRotationExpectation)
        let appMessageHandler = createAppMessageHandler(actor: actor)

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.privateKeyRotation.encode()
        )

        await fulfillment(of: [keyRotationExpectation], timeout: 1)
    }

    func testHandleAppMessageForReconnectTunnel() async throws {
        let reconnectExpectation = expectation(description: "Expect reconnecting state")

        let actor = PacketTunnelActorStub(reconnectExpectation: reconnectExpectation)
        let appMessageHandler = createAppMessageHandler(actor: actor)

        let relayConstraints = RelayConstraints(
            locations: .only(RelayLocations(locations: [.hostname("se", "sto", "se6-wireguard")]))
        )
        let selectorResult = try XCTUnwrap(try? RelaySelector.evaluate(
            relays: ServerRelaysResponseStubs.sampleRelays,
            constraints: relayConstraints,
            numberOfFailedAttempts: 0
        ))

        let selectedRelay = SelectedRelay(
            endpoint: selectorResult.endpoint,
            hostname: selectorResult.relay.hostname,
            location: selectorResult.location,
            retryAttempts: 0
        )

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.reconnectTunnel(.preSelected(selectedRelay)).encode()
        )

        await fulfillment(of: [reconnectExpectation], timeout: 1)
    }
}

extension AppMessageHandlerTests {
    func createAppMessageHandler(
        actor: PacketTunnelActorProtocol = PacketTunnelActorStub(),
        urlRequestProxy: URLRequestProxyProtocol = URLRequestProxyStub()
    ) -> AppMessageHandler {
        return AppMessageHandler(
            packetTunnelActor: actor,
            urlRequestProxy: urlRequestProxy
        )
    }
}
