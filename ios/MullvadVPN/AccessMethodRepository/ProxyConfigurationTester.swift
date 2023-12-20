//
//  ProxyConfigurationTester.swift
//  MullvadVPN
//
//  Created by pronebird on 28/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import MullvadSettings

/// A concrete implementation of an access method proxy configuration.
class ProxyConfigurationTester: ProxyConfigurationTesterProtocol {
    private var cancellable: Cancellable?

    static let shared = ProxyConfigurationTester()

    init() {}

    func start(configuration: PersistentProxyConfiguration, completion: @escaping (Error?) -> Void) {
        let workItem = DispatchWorkItem {
            let randomResult = (0 ... 255).randomElement()?.isMultiple(of: 2) ?? true

            completion(randomResult ? nil : URLError(.timedOut))
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + .seconds(2), execute: workItem)

        cancellable = AnyCancellable {
            workItem.cancel()
        }
    }

    func cancel() {
        cancellable = nil
    }
}
