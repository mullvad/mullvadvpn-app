//
//  ErrorChain.swift
//  MullvadVPN
//
//  Created by pronebird on 07/05/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A protocol describing errors that can be chained together.
protocol ChainedError: LocalizedError {
    /// A source error when available.
    var source: Error? { get }
}

/// A protocol providing error a way to override error description when printing error chain.
protocol CustomChainedErrorDescriptionProtocol {
    /// A custom error description that overrides `localizedDescription` when printing error chain.
    var customErrorDescription: String? { get }
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

    /// Create a string representation of the entire error chain.
    /// An extra `message` is added at the start of the chain when given.
    func displayChain(message: String? = nil) -> String {
        var s: String

        let errorDescription = Self.getErrorDescription(self)
        if let message = message {
            s = "Error: \(message)\nCaused by: \(errorDescription)"
        } else {
            s = "Error: \(errorDescription)"
        }

        for sourceError in makeChainIterator() {
            s.append("\nCaused by: \(Self.getErrorDescription(sourceError))")
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

    private static func getErrorDescription(_ error: Error) -> String {
        let anError = error as? CustomChainedErrorDescriptionProtocol

        return anError?.customErrorDescription ?? error.localizedDescription
    }
}

extension CustomChainedErrorDescriptionProtocol {
    var customErrorDescription: String? {
        return nil
    }
}

/// A type-erasing container type for any `Error` that makes the wrapped error behave like
/// `ChainedError`.
final class AnyChainedError: ChainedError, CustomChainedErrorDescriptionProtocol {
    private let wrappedError: Error

    init(_ error: Error) {
        wrappedError = error
    }

    var source: Error? {
        return (wrappedError as? ChainedError)?.source
    }

    var errorDescription: String? {
        return wrappedError.localizedDescription
    }

    var customErrorDescription: String? {
        return (wrappedError as? CustomChainedErrorDescriptionProtocol)?.customErrorDescription
    }
}
