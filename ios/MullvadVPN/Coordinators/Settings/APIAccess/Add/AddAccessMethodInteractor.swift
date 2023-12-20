//
//  AddAccessMethodInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 22/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import MullvadSettings

struct AddAccessMethodInteractor: AddAccessMethodInteractorProtocol {
    let subject: CurrentValueSubject<AccessMethodViewModel, Never>
    let repo: AccessMethodRepositoryProtocol
    let proxyConfigurationTester: ProxyConfigurationTesterProtocol

    func addMethod() {
        guard let persistentMethod = try? subject.value.intoPersistentAccessMethod() else { return }
        repo.add(persistentMethod)
    }

    func startProxyConfigurationTest(_ completion: ((Bool) -> Void)?) {
        guard let config = try? subject.value.intoPersistentProxyConfiguration() else { return }

        let subject = subject
        subject.value.testingStatus = .inProgress

        proxyConfigurationTester.start(configuration: config) { error in
            let succeeded = error == nil

            subject.value.testingStatus = succeeded ? .succeeded : .failed

            completion?(succeeded)
        }
    }

    func cancelProxyConfigurationTest() {
        subject.value.testingStatus = .initial

        proxyConfigurationTester.cancel()
    }
}
