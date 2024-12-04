//
//  Jittered.swift
//  MullvadREST
//
//  Created by Mojgan on 2023-12-08.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

struct Jittered<InnerIterator: IteratorProtocol>: IteratorProtocol
    where InnerIterator.Element == Duration {
    private var inner: InnerIterator

    init(_ inner: InnerIterator) {
        self.inner = inner
    }

    mutating func next() -> Duration? {
        guard let interval = inner.next() else { return nil }

        let jitter = Double.random(in: 0.0 ... 1.0)
        let millis = interval.milliseconds
        let millisWithJitter = millis.saturatingAddition(Int(Double(millis) * jitter))

        return .milliseconds(millisWithJitter)
    }
}

/// Iterator that applies a transform function to the result of another iterator.
struct Transformer<Inner: IteratorProtocol>: IteratorProtocol {
    typealias Element = Inner.Element
    private var inner: Inner
    private let transformer: (Inner.Element?) -> Inner.Element?

    init(inner: Inner, transform: @escaping @Sendable (Inner.Element?) -> Inner.Element?) {
        self.inner = inner
        self.transformer = transform
    }

    mutating func next() -> Inner.Element? {
        transformer(inner.next())
    }
}
