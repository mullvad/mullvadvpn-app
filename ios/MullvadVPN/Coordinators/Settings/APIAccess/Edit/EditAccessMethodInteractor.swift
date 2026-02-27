//
//  EditAccessMethodInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 23/11/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

@preconcurrency import Combine
import MullvadSettings

struct EditAccessMethodInteractor: EditAccessMethodInteractorProtocol {
    let subject: CurrentValueSubject<AccessMethodViewModel, Never>
    let repository: AccessMethodRepositoryProtocol
    let proxyConfigurationTester: ProxyConfigurationTesterProtocol

    var shadowsocksCiphers: [String] {
        repository.shadowsocksCiphers
    }

    var shouldShowBreadcrumb: Bool {
        if subject.value.shadowsocks.cipher.isEmpty {
            false
        } else {
            !shadowsocksCiphers.contains(subject.value.shadowsocks.cipher)
        }
    }

    init(
        subject: CurrentValueSubject<AccessMethodViewModel, Never>,
        repository: AccessMethodRepositoryProtocol,
        proxyConfigurationTester: ProxyConfigurationTesterProtocol,
    ) {
        self.subject = subject
        self.repository = repository
        self.proxyConfigurationTester = proxyConfigurationTester

        checkIfSwitchCanBeToggled()

        // Populate with default cipher if empty. Should only ever happen when adding a new Shadowsocks configuration.
        if subject.value.shadowsocks.cipher.isEmpty {
            subject.value.shadowsocks.cipher = shadowsocksCiphers.first ?? ""
        }
    }

    func saveAccessMethod() {
        guard
            let persistentMethod = try? subject.value.intoPersistentAccessMethod(shadowsocksCiphers: shadowsocksCiphers)
        else { return }

        repository.save(persistentMethod, notifyingAPI: true)
        checkIfSwitchCanBeToggled()
    }

    func deleteAccessMethod() {
        repository.delete(id: subject.value.id)
        // Enable direct access if all methods are disabled
        if repository.fetchAll().count(where: { $0.isEnabled }) == 0 {
            repository.save(repository.directAccess, notifyingAPI: true)
        }
    }

    func startProxyConfigurationTest(_ completion: (@Sendable (Bool) -> Void)?) {
        guard let config = try? subject.value.intoPersistentAccessMethod(shadowsocksCiphers: shadowsocksCiphers) else {
            return
        }

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

    // The access method can only be disabled if at least one other method is enabled
    private func checkIfSwitchCanBeToggled() {
        let enabledMethodsCount = repository.fetchAll().count { $0.isEnabled }
        if enabledMethodsCount < 2 {
            subject.value.canBeToggled = !subject.value.isEnabled
        } else {
            subject.value.canBeToggled = true
        }
    }
}
