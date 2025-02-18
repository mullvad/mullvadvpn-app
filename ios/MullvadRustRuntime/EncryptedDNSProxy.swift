//
//  EncryptedDNSProxy.swift
//  MullvadRustRuntime
//
//  Created by Emils on 24/09/2024.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadRustRuntimeProxy

public enum EncryptedDnsProxyError: Error {
    case start(err: Int32)
}

public class EncryptedDNSProxy {
    private var proxyConfig: ProxyHandle
    private var stateLock = NSLock()
    private var didStart = false
    private let state: OpaquePointer
    private let domain: String

    public init(domain: String) {
        self.domain = domain
        state = encrypted_dns_proxy_init(domain)
        proxyConfig = ProxyHandle(context: nil, port: 0)
    }

    public func localPort() -> UInt16 {
        stateLock.lock()
        defer { stateLock.unlock() }
        return proxyConfig.port
    }

    public func start() throws {
        stateLock.lock()
        defer { stateLock.unlock() }
        guard didStart == false else { return }

        let err = encrypted_dns_proxy_start(state, &proxyConfig)
        if err != 0 {
            throw EncryptedDnsProxyError.start(err: err)
        }
        didStart = true
    }

    public func stop() {
        stateLock.lock()
        defer { stateLock.unlock() }
        guard didStart == true else { return }
        didStart = false

        encrypted_dns_proxy_stop(&proxyConfig)
    }

    deinit {
        if didStart {
            encrypted_dns_proxy_stop(&proxyConfig)
        }

        encrypted_dns_proxy_free(state)
    }
}
