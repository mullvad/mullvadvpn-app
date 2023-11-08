//
//  PeekableIterator.swift
//  MullvadVPN
//
//  Created by pronebird on 03/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Iterator that can look one element ahead without consuming it.
struct PeekableIterator<Wrapped: IteratorProtocol>: IteratorProtocol {
    typealias Element = Wrapped.Element

    private var base: Wrapped
    private var nextElement: Wrapped.Element?

    init(_ base: Wrapped) {
        self.base = base
    }

    mutating func next() -> Element? {
        if let nextElement = nextElement {
            self.nextElement = nil
            return nextElement
        } else {
            return base.next()
        }
    }

    mutating func peek() -> Element? {
        if let nextElement = nextElement {
            return nextElement
        } else {
            nextElement = base.next()
            return nextElement
        }
    }
}
