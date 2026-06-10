//
//  TunnelAsyncMessagingTests.swift
//  MullvadVPNTests
//
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import XCTest

@testable import PacketTunnelCore

final class TunnelAsyncMessagingTests: XCTestCase {
    private var tunnel: MockTunnel!

    override func setUp() {
        tunnel = MockTunnel(
            tunnelProvider: SimulatorTunnelProviderManager(),
            backgroundTaskProvider: UIApplicationStub()
        )
    }

    private func makeEchoResponder(_ reply: Data?) -> (Data, ((Data?) -> Void)?) throws -> Void {
        { _, responseHandler in
            responseHandler?(reply)
        }
    }

    func testSendsImmediatelyWhenConnected() async throws {
        let reply = Data("reply".utf8)
        tunnel.status = .connected
        tunnel.onSendProviderMessage = makeEchoResponder(reply)

        let response = try await tunnel.sendProviderMessage(.getTunnelStatus)

        XCTAssertEqual(response, reply)
    }

    func testSendsImmediatelyWhenReasserting() async throws {
        let reply = Data("reply".utf8)
        tunnel.status = .reasserting
        tunnel.onSendProviderMessage = makeEchoResponder(reply)

        let response = try await tunnel.sendProviderMessage(.getTunnelStatus)

        XCTAssertEqual(response, reply)
    }

    func testThrowsTunnelDownWhenDisconnected() async {
        tunnel.status = .disconnected

        do {
            _ = try await tunnel.sendProviderMessage(.getTunnelStatus)
            XCTFail("Expected tunnelDown error")
        } catch let error as SendTunnelProviderMessageError {
            guard case .tunnelDown = error else {
                return XCTFail("Expected .tunnelDown, got \(error)")
            }
        } catch {
            XCTFail("Expected SendTunnelProviderMessageError, got \(error)")
        }
    }

    func testTimesOutWhenTunnelNeverReplies() async {
        tunnel.status = .connected
        tunnel.onSendProviderMessage = { _, _ in
            // Swallow the message, never respond.
        }

        do {
            _ = try await tunnel.sendProviderMessage(.getTunnelStatus, timeout: .milliseconds(100))
            XCTFail("Expected timeout error")
        } catch let error as SendTunnelProviderMessageError {
            guard case .timeout = error else {
                return XCTFail("Expected .timeout, got \(error)")
            }
        } catch {
            XCTFail("Expected SendTunnelProviderMessageError, got \(error)")
        }
    }

    func testSendsWithoutDelayWhenConnectingWaitHasElapsed() async throws {
        let reply = Data("reply".utf8)
        tunnel.status = .connecting
        // Tunnel started long ago — the freeze-workaround delay has fully elapsed.
        tunnel.startDate = Date(timeIntervalSinceNow: -60)
        tunnel.onSendProviderMessage = makeEchoResponder(reply)

        let startTime = Date()
        let response = try await tunnel.sendProviderMessage(.getTunnelStatus)

        XCTAssertEqual(response, reply)
        XCTAssertLessThan(Date().timeIntervalSince(startTime), 2)
    }

    func testConnectedDuringConnectingWaitSendsImmediately() async throws {
        let reply = Data("reply".utf8)
        tunnel.status = .connecting
        // Freshly started tunnel — full ~5 s wait pending.
        tunnel.startDate = Date()
        tunnel.onSendProviderMessage = makeEchoResponder(reply)

        let startTime = Date()
        let tunnel = self.tunnel!

        async let pendingResponse = tunnel.sendProviderMessage(.getTunnelStatus)

        // Let the helper subscribe, then simulate the tunnel connecting early.
        try await Task.sleep(for: .milliseconds(100))
        tunnel.simulateStatusChange(.connected)

        let response = try await pendingResponse

        XCTAssertEqual(response, reply)
        XCTAssertLessThan(Date().timeIntervalSince(startTime), 3, "Send should not wait out the full delay")
    }

    func testDisconnectedDuringConnectingWaitThrowsTunnelDown() async throws {
        tunnel.status = .connecting
        tunnel.startDate = Date()

        let tunnel = self.tunnel!

        async let pendingResponse = tunnel.sendProviderMessage(.getTunnelStatus)

        try await Task.sleep(for: .milliseconds(100))
        tunnel.simulateStatusChange(.disconnected)

        do {
            _ = try await pendingResponse
            XCTFail("Expected tunnelDown error")
        } catch let error as SendTunnelProviderMessageError {
            guard case .tunnelDown = error else {
                return XCTFail("Expected .tunnelDown, got \(error)")
            }
        } catch {
            XCTFail("Expected SendTunnelProviderMessageError, got \(error)")
        }
    }

    func testCancellationDuringConnectingWait() async throws {
        tunnel.status = .connecting
        tunnel.startDate = Date()

        let sendTask = Task { [tunnel] in
            _ = try await tunnel!.sendProviderMessage(.getTunnelStatus)
        }

        try await Task.sleep(for: .milliseconds(100))
        sendTask.cancel()

        do {
            try await sendTask.value
            XCTFail("Expected cancellation")
        } catch {
            XCTAssertTrue(error is CancellationError, "Expected CancellationError, got \(error)")
        }
    }

    func testReconnectTunnelDecodesObservedState() async throws {
        tunnel.status = .connected
        tunnel.onSendProviderMessage = makeEchoResponder(
            try TunnelProviderReply(ObservedState.disconnected).encode()
        )

        let observedState = try await tunnel.reconnectTunnel(to: .current)

        XCTAssertEqual(observedState, .disconnected)
    }
}
