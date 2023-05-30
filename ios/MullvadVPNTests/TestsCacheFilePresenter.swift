//
//  TestsCacheFilePresenter.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-06-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

class TestsCacheFilePresenter: NSObject, NSFilePresenter {
    var presentedItemURL: URL?
    let operationQueue: OperationQueue
    let dispatchQueue = DispatchQueue(label: "com.MullvadVPN.TestsCacheFilePresenter")
    var presentedItemOperationQueue: OperationQueue { operationQueue }

    var onReaderAction: (() -> Void)?
    var onWriterAction: (() -> Void)?

    init(presentedItemURL: URL) {
        operationQueue = OperationQueue()
        self.presentedItemURL = presentedItemURL
        operationQueue.underlyingQueue = dispatchQueue
    }

    func relinquishPresentedItem(toReader reader: @escaping ((() -> Void)?) -> Void) {
        onReaderAction?()
        reader(nil)
    }

    func relinquishPresentedItem(toWriter writer: @escaping ((() -> Void)?) -> Void) {
        onWriterAction?()
        writer(nil)
    }
}
