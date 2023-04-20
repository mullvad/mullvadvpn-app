//
//  HttpProxy.swift
//  MullvadREST
//
//  Created by Emils on 19/04/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network


public class HttpProxy {
    private let proxyConfig: ProxyHandle
    private let addr: IPAddress
    private let password: String
    private let cipher: String
    
    public init(addr: IPAddress, port: UInt16, password: String, cipher: String ) {
        // TODO() make the FFI call
        self.proxyConfig = ProxyHandle(context: nil, port: port)
        self.addr = addr
        self.password = password
        self.cipher = cipher
    }
    
    func port() -> UInt16 {
        self.proxyConfig.port
    }
    
    /// Start the socks proxy
    func start() {
        // Get the raw bytes of `addr.rawValue`
        addr.rawValue.withUnsafeBytes { unsafeAddressPointer in
            
            // Rebind the raw bytes to an array of bytes, and get a pointer to its beginning
            let rawAddr = unsafeAddressPointer.bindMemory(to: UInt8.self).baseAddress
            
            // Get the raw bytes access to  `proxyConfig`
            withUnsafePointer(to: proxyConfig) { config in
                let configPointer = UnsafeMutablePointer(mutating: config)
                start_shadowsocks_proxy(rawAddr, UInt(addr.rawValue.count), port(), password, UInt(password.count), cipher, UInt(cipher.count), configPointer)
            }
        }
    }
    
    /// Stop the socks proxy
    func stop() {
        let _ = withUnsafePointer(to: self.proxyConfig) { pointer in
            stop_shadowsocks_proxy(UnsafeMutablePointer(mutating: pointer))
        }
    }
}

