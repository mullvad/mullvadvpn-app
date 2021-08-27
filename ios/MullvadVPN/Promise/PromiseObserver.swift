//
//  PromiseObserver.swift
//  PromiseObserver
//
//  Created by pronebird on 22/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol PromiseObserver {
    associatedtype Value

    func receiveCompletion(_ completion: PromiseCompletion<Value>)
}

final class AnyPromiseObserver<Value>: PromiseObserver {
    private let onReceiveCompletion: (PromiseCompletion<Value>) -> Void

    init(_ receiveCompletionHandler: @escaping (PromiseCompletion<Value>) -> Void) {
        onReceiveCompletion = receiveCompletionHandler
    }

    func receiveCompletion(_ completion: PromiseCompletion<Value>) {
        onReceiveCompletion(completion)
    }
}
