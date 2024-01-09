//
//  RESTTransportProvider.swift
//  MullvadREST
//
//  Created by pronebird on 24/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

public protocol RESTTransportProvider {
    func makeTransport() -> RESTTransport?
}

extension REST {
    public struct AnyTransportProvider: RESTTransportProvider {
        private let block: () -> RESTTransport?

        public init(_ block: @escaping () -> RESTTransport?) {
            self.block = block
        }

        public func makeTransport() -> RESTTransport? {
            return block()
        }
    }
}
