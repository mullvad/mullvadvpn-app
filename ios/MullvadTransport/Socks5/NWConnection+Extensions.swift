//
//  NWConnection+Extensions.swift
//  MullvadTransport
//
//  Created by pronebird on 20/10/2023.
//

import Foundation
import Network

extension NWConnection {
    /**
     Read exact number of bytes from connection.

     - Parameters:
        - exactLength: exact number of bytes to read.
        - completion:  a completion handler.
     */
    func receive(exactLength: Int, completion: @escaping (Data?, ContentContext?, Bool, NWError?) -> Void) {
        receive(minimumIncompleteLength: exactLength, maximumLength: exactLength, completion: completion)
    }
}
