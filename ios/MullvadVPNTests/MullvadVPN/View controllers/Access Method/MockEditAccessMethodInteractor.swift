// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Combine
import MullvadSettings
import MullvadTypes

final class MockEditAccessMethodInteractor: EditAccessMethodInteractorProtocol, @unchecked Sendable {

    private let accessMethodSubject = CurrentValueSubject<PersistentAccessMethod?, Never>(nil)

    var currentAccessMethod: PersistentAccessMethod? {
        didSet {
            accessMethodSubject.send(currentAccessMethod)
        }
    }

    var isLastActive = false

    var isLastActiveAccessMethod: Bool {
        isLastActive
    }

    var hasMethodsWithInvalidCiphers = false

    var shadowsocksCiphers: [String] = []

    private(set) var setEnabledCalled = false
    private(set) var lastEnabledValue: Bool?
    private(set) var startTestingCalled = false
    private(set) var stopTestingCalled = false

    private(set) var deleteCalled = false
    private(set) var deletedMethod: PersistentAccessMethod?

    var testResult: Result<Void, Error> = .success(())

    // MARK: - Access Method Publisher

    func accessMethodPublisher(for id: UUID) -> AnyPublisher<PersistentAccessMethod?, Never> {
        accessMethodSubject
            .eraseToAnyPublisher()
    }

    func sendAccessMethod(_ method: PersistentAccessMethod?) {
        accessMethodSubject.send(method)
    }

    // MARK: - Access Method Lookup

    func isLastActiveAccessMethod(_ method: PersistentAccessMethod) -> Bool {
        isLastActive
    }

    func fetch(_ id: UUID) throws -> PersistentAccessMethod {
        guard let currentAccessMethod,
            currentAccessMethod.id == id
        else {
            throw MockEditAccessMethodError.failed
        }

        return currentAccessMethod
    }

    // MARK: - Save

    func save(_ method: PersistentAccessMethod) {
        currentAccessMethod = method
    }

    // MARK: - Set Enabled

    func setEnabled(_ isEnabled: Bool, method: PersistentAccessMethod) throws {
        setEnabledCalled = true
        lastEnabledValue = isEnabled
    }

    // MARK: - Delete

    func delete(_ method: PersistentAccessMethod) {
        deleteCalled = true
        deletedMethod = method
    }

    // MARK: - Start Testing
    func test(_ method: PersistentAccessMethod) async throws {
        startTestingCalled = true
        try testResult.get()
    }

    // MARK: - Stop Testing
    func cancelTesting() async {
        stopTestingCalled = true
    }
}

enum MockEditAccessMethodError: Error {
    case failed
}
