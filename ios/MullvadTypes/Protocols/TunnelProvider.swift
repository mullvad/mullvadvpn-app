//
//  TunnelProvider.swift
//  MullvadTypes
//
//  Created by Marco Nikic on 2024-07-04.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

public protocol TunnelProvider: AnyObject {
    func tunnelHandle() throws -> Int32
    func wgFuncs() -> WgFuncPointers
}

public typealias TcpOpenFunc = @convention(c) (Int32, UnsafePointer<CChar>?, UInt64) -> Int32
public typealias TcpCloseFunc = @convention(c) (Int32, Int32) -> Int32
public typealias TcpSendFunc = @convention(c) (Int32, Int32, UnsafePointer<UInt8>?, Int32) -> Int32
public typealias TcpRecvFunc = @convention(c) (Int32, Int32, UnsafeMutablePointer<UInt8>?, Int32) -> Int32

public struct WgFuncPointers {
    public let open: TcpOpenFunc
    public let close: TcpCloseFunc
    public let receive: TcpRecvFunc
    public let send: TcpSendFunc

    public init(open: TcpOpenFunc, close: TcpCloseFunc, receive: TcpRecvFunc, send: TcpSendFunc) {
        self.open = open
        self.close = close
        self.receive = receive
        self.send = send
    }
}
