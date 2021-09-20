//
//  TunnelIPCError.swift
//  MullvadVPN
//
//  Created by pronebird on 16/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension TunnelIPC {
    /// An error type emitted by `TunnelIPC.Session`.
    enum Error: ChainedError {
        /// A failure to encode the request
        case encoding(Swift.Error)

        /// A failure to decode the response
        case decoding(Swift.Error)

        /// A failure to send the IPC request
        case send(Swift.Error)

        /// A failure that's raised when the IPC response does not contain any data however the decoder
        /// expected to receive data for decoding
        case nilResponse

        var errorDescription: String? {
            switch self {
            case .encoding:
                return "Encoding failure"
            case .decoding:
                return "Decoding failure"
            case .send:
                return "Submission failure"
            case .nilResponse:
                return "Unexpected nil response from the tunnel"
            }
        }
    }
}
