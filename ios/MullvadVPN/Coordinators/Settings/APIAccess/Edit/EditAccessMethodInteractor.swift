// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
import Combine
import MullvadSettings

final class EditAccessMethodInteractor: EditAccessMethodInteractorProtocol {
    let subject: CurrentValueSubject<AccessMethodViewModel, Never>
    let repository: AccessMethodRepositoryProtocol
    let proxyConfigurationTester: ProxyConfigurationTesterProtocol
    private var proxyConfigurationTestTask: Task<Void, Never>?

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

    func startProxyConfigurationTest(
        _ completion: (@Sendable (Bool) -> Void)?
    ) {
        guard
            let config = try? subject.value.intoPersistentAccessMethod(
                shadowsocksCiphers: shadowsocksCiphers
            )
        else {
            return
        }

        let subject = subject
        subject.value.testingStatus = .inProgress

        proxyConfigurationTestTask?.cancel()

        proxyConfigurationTestTask = Task {
            do {
                try await proxyConfigurationTester.start(
                    configuration: config
                )

                guard !Task.isCancelled else {
                    return
                }

                subject.value.testingStatus = .succeeded
                completion?(true)
            } catch {
                guard !Task.isCancelled else {
                    return
                }

                subject.value.testingStatus = .failed
                completion?(false)
            }
        }
    }

    func cancelProxyConfigurationTest() {
        subject.value.testingStatus = .initial
        proxyConfigurationTestTask?.cancel()
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
