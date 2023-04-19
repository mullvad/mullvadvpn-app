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
    
    public init(addr: IPAddress, port: UInt16, password: String, cipher: String ) {
        // TODO() make the FFI call
        self.proxyConfig = ProxyHandle.init(context: nil, port: 0)
    }
    
    func port() -> UInt16 {
        self.proxyConfig.port
    }
    
    deinit {
        let _ = withUnsafePointer(to: self.proxyConfig) { pointer in
            stop_shadowsocks_proxy(UnsafeMutablePointer(mutating: pointer))
        }
    }
    
}
