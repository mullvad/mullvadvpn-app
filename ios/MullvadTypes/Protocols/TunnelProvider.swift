//
//  TunnelProvider.swift
//  MullvadTypes
//
//  Created by Marco Nikic on 2024-07-04.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

public protocol TunnelProvider: AnyObject {
    func tunnelHandle() throws -> Int32
    func wgFunctions() -> WgFunctionPointers
}

public typealias TcpOpenFunction = @convention(c) (Int32, UnsafePointer<CChar>?, UInt64) -> Int32
public typealias TcpCloseFunction = @convention(c) (Int32, Int32) -> Int32
public typealias TcpSendFunction = @convention(c) (Int32, Int32, UnsafePointer<UInt8>?, Int32) -> Int32
public typealias TcpRecvFunction = @convention(c) (Int32, Int32, UnsafeMutablePointer<UInt8>?, Int32) -> Int32

public struct WgFunctionPointers {
    public let open: TcpOpenFunction
    public let close: TcpCloseFunction
    public let receive: TcpRecvFunction
    public let send: TcpSendFunction

    public init(open: TcpOpenFunction, close: TcpCloseFunction, receive: TcpRecvFunction, send: TcpSendFunction) {
        self.open = open
        self.close = close
        self.receive = receive
        self.send = send
    }
}
