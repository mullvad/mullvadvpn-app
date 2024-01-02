//
//  EditAccessMethodInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 23/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import MullvadSettings

struct EditAccessMethodInteractor: EditAccessMethodInteractorProtocol {
    let subject: CurrentValueSubject<AccessMethodViewModel, Never>
    let repository: AccessMethodRepositoryProtocol
    let proxyConfigurationTester: ProxyConfigurationTesterProtocol

    var directAccess: PersistentAccessMethod {
        repository.directAccess
    }

    public var publisher: AnyPublisher<[PersistentAccessMethod], Never> {
        repository.publisher.eraseToAnyPublisher()
    }

    func saveAccessMethod() {
        guard let persistentMethod = try? subject.value.intoPersistentAccessMethod() else { return }

        repository.save(persistentMethod)
    }

    func deleteAccessMethod() {
        repository.delete(id: subject.value.id)
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
