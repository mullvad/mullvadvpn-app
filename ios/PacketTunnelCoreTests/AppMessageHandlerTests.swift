//
//  AppMessageHandlerTests.swift
//  PacketTunnelCoreTests
//
//  Created by Jon Petersson on 2023-09-28.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
@testable import MullvadMockData
@testable import MullvadREST
import MullvadTypes
import PacketTunnelCore
import XCTest

final class AppMessageHandlerTests: XCTestCase {
    // MARK: URLRequest

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

        await fulfillment(of: [sendRequestExpectation], timeout: .UnitTest.timeout)
    }

    func testHandleAppMessageForCancelURLRequest() async throws {
        let cancelRequestExpectation = expectation(description: "Expect cancelling request")

        let urlRequestProxy = URLRequestProxyStub(cancelRequestExpectation: cancelRequestExpectation)
        let appMessageHandler = createAppMessageHandler(urlRequestProxy: urlRequestProxy)

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.cancelURLRequest(UUID()).encode()
        )

        await fulfillment(of: [cancelRequestExpectation], timeout: .UnitTest.timeout)
    }

    // MARK: APIRequest

    func testHandleAppMessageForSendAPIRequest() async throws {
        let sendRequestExpectation = expectation(description: "Expect sending request")

        let apiRequestProxy = APIRequestProxyStub(sendRequestExpectation: sendRequestExpectation)
        let appMessageHandler = createAppMessageHandler(apiRequestProxy: apiRequestProxy)

        let apiRequest = ProxyAPIRequest(
            id: UUID(),
            request: .getAddressList(.default)
        )

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.sendAPIRequest(apiRequest).encode()
        )

        await fulfillment(of: [sendRequestExpectation], timeout: .UnitTest.timeout)
    }

    func testHandleAppMessageForCancelAPIRequest() async throws {
        let cancelRequestExpectation = expectation(description: "Expect cancelling request")

        let apiRequestProxy = APIRequestProxyStub(cancelRequestExpectation: cancelRequestExpectation)
        let appMessageHandler = createAppMessageHandler(apiRequestProxy: apiRequestProxy)

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.cancelAPIRequest(UUID()).encode()
        )

        await fulfillment(of: [cancelRequestExpectation], timeout: .UnitTest.timeout)
    }

    // MARK: Other

    func testHandleAppMessageForTunnelStatus() async throws {
        let stateExpectation = expectation(description: "Expect getting state")

        let actor = PacketTunnelActorStub(stateExpectation: stateExpectation)
        let appMessageHandler = createAppMessageHandler(actor: actor)

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.getTunnelStatus.encode()
        )

        await fulfillment(of: [stateExpectation], timeout: .UnitTest.timeout)
    }

    func testHandleAppMessageForKeyRotation() async throws {
        let keyRotationExpectation = expectation(description: "Expect key rotation")

        let actor = PacketTunnelActorStub(keyRotationExpectation: keyRotationExpectation)
        let appMessageHandler = createAppMessageHandler(actor: actor)

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.privateKeyRotation.encode()
        )

        await fulfillment(of: [keyRotationExpectation], timeout: .UnitTest.timeout)
    }

    func testHandleAppMessageForReconnectTunnel() async throws {
        let reconnectExpectation = expectation(description: "Expect reconnecting state")

        let actor = PacketTunnelActorStub(reconnectExpectation: reconnectExpectation)
        let appMessageHandler = createAppMessageHandler(actor: actor)

        let relayConstraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se6-wireguard")]))
        )

        let candidates = try RelaySelector.WireGuard.findCandidates(
            by: relayConstraints.exitLocations,
            in: ServerRelaysResponseStubs.sampleRelays,
            filterConstraint: relayConstraints.filter,
            daitaEnabled: false
        )

        let match = try RelaySelector.WireGuard.pickCandidate(
            from: candidates,
            wireguard: ServerRelaysResponseStubs.sampleRelays.wireguard,
            portConstraint: relayConstraints.port,
            numberOfFailedAttempts: 0
        )

        let selectedRelays = SelectedRelays(
            entry: nil,
            exit: SelectedRelay(
                endpoint: match.endpoint,
                hostname: match.relay.hostname,
                location: match.location
            ),
            retryAttempt: 0
        )

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.reconnectTunnel(.preSelected(selectedRelays)).encode()
        )

        await fulfillment(of: [reconnectExpectation], timeout: .UnitTest.timeout)
    }
}

extension AppMessageHandlerTests {
    func createAppMessageHandler(
        actor: PacketTunnelActorProtocol = PacketTunnelActorStub(),
        urlRequestProxy: URLRequestProxyProtocol = URLRequestProxyStub(),
        apiRequestProxy: APIRequestProxyProtocol = APIRequestProxyStub()
    ) -> AppMessageHandler {
        return AppMessageHandler(
            packetTunnelActor: actor,
            urlRequestProxy: urlRequestProxy,
            apiRequestProxy: apiRequestProxy
        )
    }
}
