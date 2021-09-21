//
//  ErrorChain.swift
//  MullvadVPN
//
//  Created by pronebird on 07/05/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A protocol describing errors that can be chained together
protocol ChainedError: LocalizedError {
    /// A source error when available
    var source: Error? { get }
}

final class AnyChainedError: ChainedError {
    private let wrappedError: Error

    init(_ error: Error) {
        wrappedError = error
    }

    var errorDescription: String? {
        return wrappedError.localizedDescription
    }
}

extension ChainedError {

    var source: Error? {
        let reflection = Mirror(reflecting: self)

        if case .enum = reflection.displayStyle {
            for child in reflection.children {
                if let associatedError = child.value as? Error {
                    return associatedError
                }
            }
        }

        return nil
    }

    /// Creates a string representation of the entire error chain.
    /// An extra `message` is added at the start of the chain when given
    func displayChain(message: String? = nil) -> String {
        var s = message.map { "Error: \($0)\nCaused by: \(self.localizedDescription)" }
            ?? "Error: \(self.localizedDescription)"

        for sourceError in makeChainIterator() {
            s.append("\nCaused by: \(sourceError.localizedDescription)")
        }

        return s
    }

    private func makeChainIterator() -> AnyIterator<Error> {
        var current: Error? = self
        return AnyIterator { () -> Error? in
            current = (current as? ChainedError)?.source
            return current
        }
    }
}
