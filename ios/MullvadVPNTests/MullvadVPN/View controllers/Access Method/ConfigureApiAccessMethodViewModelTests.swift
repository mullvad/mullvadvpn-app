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
import MullvadTypes
import Testing

@MainActor
@Suite("ConfigureApiAccessMethodViewModel Tests")
struct ConfigureApiAccessMethodViewModelTests {

    private func makeSUT(
        accessMethodViewModel: AccessMethodViewModel,
        interactor: MockEditAccessMethodInteractor = .init()
    ) -> (
        sut: ConfigureApiAccessMethodViewModel,
        interactor: MockEditAccessMethodInteractor
    ) {
        let sut = ConfigureApiAccessMethodViewModel(
            accessMethodViewModel: accessMethodViewModel,
            interactor: interactor
        )

        interactor.shadowsocksCiphers = [
            "aes-128-gcm",
            "chacha20-ietf-poly1305",
        ]

        return (sut, interactor)
    }

    private var accessMethodViewModel: AccessMethodViewModel {
        let item = PersistentAccessMethod.makeCustomAccessMethod()
        return AccessMethodViewModel(
            id: item.id, name: item.name, isEnabled: item.isEnabled,
            shadowsocks: item.proxyConfiguration.shadowsocksViewModel, )
    }

    @Test("Initializes with idle test state")
    func initialTestStateIsIdle() {
        let (sut, _) = makeSUT(accessMethodViewModel: accessMethodViewModel)

        #expect(sut.testState == .idle)
    }

    @Test("Initializes with valid validation state")
    func initialValidationStateIsValid() {
        let (sut, _) = makeSUT(
            accessMethodViewModel: accessMethodViewModel
        )

        #expect(sut.validationState == .valid)
    }

    @Test("Returns true when existing access methods have invalid ciphers")
    func hasInvalidCipherReturnsTrue() {
        let (sut, interactor) = makeSUT(accessMethodViewModel: accessMethodViewModel)
        interactor.hasMethodsWithInvalidCiphers = true

        #expect(sut.hasInvalidCipher)
    }

    @Test("Returns false when existing access methods have valid ciphers")
    func hasInvalidCipherReturnsFalse() {
        let (sut, interactor) = makeSUT(accessMethodViewModel: accessMethodViewModel)
        interactor.shadowsocksCiphers = [
            "aes-128-gcm",
            "chacha20-ietf-poly1305",
        ]

        #expect(!sut.hasInvalidCipher)
    }

    @Test("Saves valid access method")
    func savePersistsAccessMethod() throws {
        let method = accessMethodViewModel
        let (sut, interactor) = makeSUT(accessMethodViewModel: method)

        sut.save()

        #expect(interactor.currentAccessMethod != nil)
    }

    @Test("Sets test state to unreachable when testing fails")
    func startTestingSetsUnreachableOnFailure() async {
        let (sut, interactor) = makeSUT(accessMethodViewModel: accessMethodViewModel)

        interactor.testResult = .failure(MockEditAccessMethodError.failed)

        await sut.startTesting()

        #expect(sut.testState == .unreachable)
    }

    @Test("Sets test state to idle when testing is cancelled")
    func startTestingSetsIdleOnCancellation() async {
        let (sut, interactor) = makeSUT(accessMethodViewModel: accessMethodViewModel)

        interactor.testResult = .failure(CancellationError())
        await sut.startTesting()

        #expect(sut.testState == .idle)
    }

    @Test("Propagates cancellation to interactor")
    func cancelTestingCallsInteractor() async {
        let (sut, interactor) = makeSUT(accessMethodViewModel: accessMethodViewModel)
        interactor.shadowsocksCiphers = [
            "aes-128-gcm",
            "chacha20-ietf-poly1305",
        ]

        await sut.startTesting()
        await sut.cancelTesting()

        #expect(sut.testState == .idle)
    }
}
