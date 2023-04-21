//
//  ShadowSocksProxy.swift
//  MullvadREST
//
//  Created by Emils on 19/04/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

public class ShadowSocksProxy: Equatable {
    public static func == (lhs: ShadowSocksProxy, rhs: ShadowSocksProxy) -> Bool {
        lhs.uuid == rhs.uuid
    }
    
    
    private var proxyConfig: ProxyHandle
    private let remoteAddress: IPAddress
    private let remotePort: UInt16
    private let password: String
    private let cipher: String
    public let uuid = UUID()
    
    public init(remoteAddress: IPAddress, remotePort: UInt16, password: String, cipher: String ) {
        self.proxyConfig = ProxyHandle(context: nil, port: 0)
        self.remoteAddress = remoteAddress
        self.remotePort = remotePort
        self.password = password
        self.cipher = cipher
    }
    
    public func localPort() -> UInt16 {
        self.proxyConfig.port
    }
    
    /// Start the socks proxy
    public func start() {
        // Get the raw bytes of `addr.rawValue`
        remoteAddress.rawValue.withUnsafeBytes { unsafeAddressPointer in
            
            // Rebind the raw bytes to an array of bytes, and get a pointer to its beginning
            let rawAddr = unsafeAddressPointer.bindMemory(to: UInt8.self).baseAddress
            
            // Get the raw bytes access to  `proxyConfig`
            _ = withUnsafeMutablePointer(to: &proxyConfig) { config in
                start_shadowsocks_proxy(rawAddr, UInt(remoteAddress.rawValue.count), remotePort, password, UInt(password.count), cipher, UInt(cipher.count), config)
                
            }
            
            print("Proxy config port: \(proxyConfig.port)")
        }
    }
    
    /// Stop the socks proxy
    public func stop() {
        let _ = withUnsafePointer(to: self.proxyConfig) { pointer in
            stop_shadowsocks_proxy(UnsafeMutablePointer(mutating: pointer))
            stop_shadowsocks_proxy(UnsafeMutablePointer(mutating: pointer))
        }
    }
}

