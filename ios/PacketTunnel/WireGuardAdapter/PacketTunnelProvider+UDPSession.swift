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

class TunUdpSession {
    var session: NWUDPSession
    private let rustContext: UnsafeMutableRawPointer
    
    private var betterPathAvailable = false
    private var betterPathObserver: NSKeyValueObservation? = nil
    
    private var isViableObserver: NSKeyValueObservation? = nil
    
    var isReady = false
    var stateObserver: NSKeyValueObservation? = nil
    

    
    init(endpoint: NWHostEndpoint,  provider: PacketTunnelProvider, rustContext: UnsafeMutableRawPointer) {
        self.rustContext = rustContext
        
        session = provider.createUDPSession(to: endpoint, from: nil)
        setupWatchers()
    }
    
    func setupWatchers() {
        // Clear previous observers
        stateObserver?.invalidate()
        stateObserver = nil
        betterPathObserver?.invalidate()
        betterPathObserver = nil
        isViableObserver?.invalidate()
        isViableObserver = nil
        
        stateObserver = session.observe(\.state, options: [.new]) { [weak self] session, _ in
            guard let self else { return }
                
            if session.state == .ready {
                excluded_udp_session_ready(rustContext)
            }

            if session.state == .cancelled || session.state == .failed {
                excluded_udp_session_not_ready(rustContext)
            }
            }
        
        
            betterPathObserver = session.observe(\.hasBetterPath, options: [.new]) { [weak self] session, _ in
                guard let self else { return }
                if session.hasBetterPath {
                    self.session = NWUDPSession(upgradeFor: session)
                    setupWatchers()
                    excluded_udp_session_ready(rustContext)
                }
            }

            isViableObserver = session.observe(\.isViable, options: [.new]) {  session, _ in
                if session.isViable {
                    excluded_udp_session_ready(self.rustContext)
                } else {
                    excluded_udp_session_not_ready(self.rustContext)
                }
            }

            session.setReadHandler({ [weak self] readData, maybeError in
                guard let self else { return }
                if let maybeError {
                    NSLog("\(maybeError.localizedDescription)")
                    excluded_udp_session_recv_err(self.rustContext, Int32(UDPSessionInformation.readHandlerError.rawValue))
                    return
                }
                guard let readData else {
                    NSLog("No data was read")
                    excluded_udp_session_recv_err(rustContext, Int32(UDPSessionInformation.failedReadingData.rawValue))
                    return
                }
                let rawData = DataArray(readData).toRaw()
                excluded_udp_session_recv(rustContext, rawData)
            }, maxDatagrams: 2000)
        
    }

        
}

@_cdecl("swift_log_crash")
func udpLogStuff() -> UInt32 {
    var interesting = "called ";
    interesting += "3"
    return UInt32(0)
}

// Creates a UDP session
// `addr` is pointer to a valid UTF-8 string representing the socket address
// `addrLen` is representing the length of the `addr` string in bytes
// `rustContext` is a pointer to the Rust context.
@_cdecl("swift_nw_excluded_udp_session_create")
func udpSessionCreate(
    addr: UnsafeMutableRawPointer,
    addrLen: UInt64,
    port: UInt16,
    packetTunnelContext: UnsafeMutableRawPointer,
    rustContext: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer {
    let addressData = Data(bytes: addr, count: Int(addrLen))
    let address = String(decoding: addressData, as: UTF8.self)
    let endpoint = NWHostEndpoint(hostname: address, port: "\(port)")

    let packetTunnel = Unmanaged<PacketTunnelProvider>.fromOpaque(packetTunnelContext).takeUnretainedValue()
    let session = TunUdpSession(endpoint: endpoint, provider: packetTunnel, rustContext: rustContext)

    return Unmanaged.passUnretained(session).toOpaque()
}

// Will be called from the Rust side to send data.
// `session` is a pointer to Self
// `data` is a pointer to a DataArray (AbstractTunData.swift)
@_cdecl("swift_nw_excluded_udp_session_send")
func udpSessionSend(session: UnsafeMutableRawPointer, data: UnsafeMutableRawPointer, completionToken: UnsafeMutableRawPointer) {
    let tun = Unmanaged<TunUdpSession>.fromOpaque(session).takeUnretainedValue()
    let dataArray = Unmanaged<DataArray>.fromOpaque(data).takeUnretainedValue()
    tun.session.writeMultipleDatagrams(dataArray.arr) { maybeError in
        if let maybeError {
            NSLog("\(maybeError.localizedDescription)")
        }
        excluded_udp_session_send_complete(completionToken, 0)
    }
}

// Should destroy current UDP session
// After this call, no callbacks into rust should be made with the rustContext pointer.
@_cdecl("swift_nw_excluded_udp_session_destroy")
func udpSessionDestroy(session: UnsafeMutableRawPointer) {
    let udpSession = Unmanaged<NWUDPSession>.fromOpaque(session).takeRetainedValue()
    udpSession.cancel()
}
