//
//  PacketTunnelProvider+UDPSession.swift
//  PacketTunnel
//
//  Created by Marco Nikic on 2023-12-06.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

enum UDPSessionInformation: UInt64 {
    case betterPathAvailable
    case canReadOrWriteData
    case cannotReadOrWriteData
    case readHandlerError
    case failedReadingData
}

// Creates a UDP session
// `addr` is pointer to a valid UTF-8 string representing the socket address
// `addrLen` is representing the length of the `addr` string in bytes
// `rustContext` is a pointer to the Rust context.
@_cdecl("swift_nw_udp_session_create")
func udpSessionCreate(
    addr: UnsafeMutableRawPointer,
    addrLen: UInt64,
    packetTunnelContext: UnsafeMutableRawPointer,
    rustContext: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer {
    let address = addr.withMemoryRebound(to: Data.self, capacity: 1) {
        String(data: $0.pointee, encoding: .utf8).unsafelyUnwrapped
    }

    let endpoint = NWHostEndpoint(hostname: address, port: "")

    let packetTunnel = Unmanaged<PacketTunnelProvider>.fromOpaque(packetTunnelContext).takeUnretainedValue()
    let session = packetTunnel.createUDPSession(to: endpoint, from: nil)
    setupSessionObservers(for: session, packetTunnel: packetTunnel, rustContext: rustContext)
//    packetTunnel.localUDPSession = session

    return Unmanaged.passUnretained(session).toOpaque()
}

func setupSessionObservers(
    for session: NWUDPSession,
    packetTunnel: PacketTunnelProvider,
    rustContext: UnsafeMutableRawPointer
) {
    // Clear previous observers
//    packetTunnel.localUDPSessionStateObserver = nil
//    packetTunnel.localUDPSessionBetterPathObserver = nil

    let stateObserver = session.observe(\.state, options: [.new]) { session, _ in
        if session.state == .ready {
//            guard packetTunnel.localUDPSessionIsReady == false else { return }
            udp_session_ready(rustContext: rustContext)
//            packetTunnel.localUDPSessionIsReady = true
        }

        if session.state == .cancelled || session.state == .failed {
//            packetTunnel.localUDPSessionIsReady = false
        }
    }
//    packetTunnel.localUDPSessionStateObserver = stateObserver

    let pathObserver = session.observe(\.hasBetterPath, options: [.new]) { session, _ in
        if session.hasBetterPath {
            let upgradedSession = NWUDPSession(upgradeFor: session)
            setupSessionObservers(for: upgradedSession, packetTunnel: packetTunnel, rustContext: rustContext)
//            packetTunnel.localUDPSession = upgradedSession
            let rawSession = Unmanaged.passUnretained(upgradedSession).toOpaque()
            udp_session_upgrade(rustContext: rustContext, newSession: rawSession)
        }
    }
//    packetTunnel.localUDPSessionBetterPathObserver = pathObserver

    let isViableObserver = session.observe(\.isViable, options: [.new]) { session, _ in
        let isViable: UDPSessionInformation = session.isViable ? .canReadOrWriteData : .cannotReadOrWriteData
        udp_session_error(rustContext: rustContext, status: isViable.rawValue)
    }
//    packetTunnel.localUDPSessionIsViableObserver = isViableObserver

    session.setReadHandler({ readData, maybeError in
        if let maybeError {
            NSLog("\(maybeError.localizedDescription)")
            udp_session_error(rustContext: rustContext, status: UDPSessionInformation.readHandlerError.rawValue)
            return
        }
        guard let readData else {
            NSLog("No data was read")
            udp_session_error(rustContext: rustContext, status: UDPSessionInformation.failedReadingData.rawValue)
            return
        }
        let rawData = DataArray(readData).toRaw()
        udp_session_recv(rustContext: rustContext, data: rawData)
    }, maxDatagrams: 2000)
}

// Will be called from the Rust side to send data.
// `session` is a pointer to Self
// `data` is a pointer to a DataArray (AbstractTunData.swift)
@_cdecl("swift_nw_udp_session_send")
func udpSessionSend(session: UnsafeMutableRawPointer, data: UnsafeMutableRawPointer) {
    let udpSession = Unmanaged<NWUDPSession>.fromOpaque(session).takeUnretainedValue()
    let dataArray = Unmanaged<DataArray>.fromOpaque(data).takeUnretainedValue()
    udpSession.writeMultipleDatagrams(dataArray.arr) { maybeError in
        if let maybeError {
            NSLog("\(maybeError.localizedDescription)")
            // TODO: maybe get a rust context here in case of error ?
        }
    }
}

// Should destroy current UDP session
// After this call, no callbacks into rust should be made with the rustContext pointer.
@_cdecl("swift_nw_udp_session_destroy")
func udpSessionDestroy(session: UnsafeMutableRawPointer) {
    let udpSession = Unmanaged<NWUDPSession>.fromOpaque(session).takeUnretainedValue()
    udpSession.cancel()
    // TODO: Maybe pass a pointer to the packet tunnel handler so it can be cleaned here too
}

// TODO: Remove these once the rust code is in place
// Callback into Rust when new data is received.
func udp_session_recv(rustContext: UnsafeMutableRawPointer, data: UnsafeMutableRawPointer) {}
// Callback to call when UDP session state changes to `ready`. Only expected to be called once.
func udp_session_ready(rustContext: UnsafeMutableRawPointer) {}
// An error callback to be called when non-transient errors are present,
// i.e. state of session changes, or if the session has a better path
func udp_session_error(rustContext: UnsafeMutableRawPointer, status: UInt64) {}
// Callback into rust when a better path is available.
func udp_session_upgrade(rustContext: UnsafeMutableRawPointer, newSession: UnsafeMutableRawPointer) {}
