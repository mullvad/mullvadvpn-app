//
//  TunnelIPCResponse.swift
//  TunnelIPCResponse
//
//  Created by pronebird on 27/07/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension TunnelIPC {
    /// A container type around Tunnel IPC response.
    /// The primary purpose of this type is to provide a top level object for `JSONEncoder`.
    struct Response<T: Codable>: Codable {
        var value: T
    }
}
