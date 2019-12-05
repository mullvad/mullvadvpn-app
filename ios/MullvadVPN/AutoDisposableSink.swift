//
//  AutoDisposableSink.swift
//  MullvadVPN
//
//  Created by pronebird on 01/11/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Combine
import Foundation

/// A thread safe storage for a set of `AnyCancellable` objects
final class CancellableSet {
    private let lock = NSLock()
    private var storage = Set<AnyCancellable>()

    func append(_ cancellable: AnyCancellable) {
        lock.lock()
        storage.insert(cancellable)
        lock.unlock()
    }

    func remove(_ cancellable: AnyCancellable) {
        lock.lock()
        storage.remove(cancellable)
        lock.unlock()
    }
}

extension Publisher {

    /// Make a `Publishers.Sink` subscriber and put it in the given `CancellableSet`, automatically
    /// remove it upon completion.
    func autoDisposableSink(cancellableSet: CancellableSet, receiveCompletion: @escaping ((Subscribers.Completion<Self.Failure>) -> Void), receiveValue: @escaping ((Self.Output) -> Void)) -> Void {
        var sharedCancellable: AnyCancellable?

        let disposeSubscriber = {
            if let sharedCancellable = sharedCancellable {
                cancellableSet.remove(sharedCancellable)
            }
        }

        let cancellable = handleEvents(receiveCancel: {
            disposeSubscriber()
        }).sink(receiveCompletion: { (completion) in
            receiveCompletion(completion)

            disposeSubscriber()
        }, receiveValue: receiveValue)

        sharedCancellable = cancellable
        cancellableSet.append(cancellable)
    }

}

extension Publisher where Output == Void, Failure: Error {

    func sink(receiveCompletion: @escaping ((Subscribers.Completion<Failure>) -> Void)) -> AnyCancellable {
        return sink(receiveCompletion: receiveCompletion, receiveValue: { _ in })
    }

    func autoDisposableSink(cancellableSet: CancellableSet, receiveCompletion: @escaping ((Subscribers.Completion<Failure>) -> Void)) -> Void {
        return autoDisposableSink(cancellableSet: cancellableSet, receiveCompletion: receiveCompletion, receiveValue: { _ in })
    }

}
