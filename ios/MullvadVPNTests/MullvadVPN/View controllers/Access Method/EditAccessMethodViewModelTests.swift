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
@Suite("EditAccessMethodViewModel Tests")
struct EditAccessMethodViewModelTests {

    private func makeSUT(accessMethod: PersistentAccessMethod) -> (
        sut: EditAccessMethodViewModel,
        interactor: MockEditAccessMethodInteractor
    ) {
        let method = accessMethod

        let interactor = MockEditAccessMethodInteractor()
        interactor.currentAccessMethod = method

        let sut = EditAccessMethodViewModel(
            accessMethod: method,
            interactor: interactor
        )

        return (sut, interactor)
    }

    // MARK: - Initial State
    @Test("Initial access method is correctly assigned")
    func initialAccessMethodIsAssigned() {
        let method = PersistentAccessMethod.makePersistentAccessMethod()

        let (sut, _) = makeSUT(accessMethod: method)

        #expect(sut.accessMethod == method)
    }

    @Test("Initial enabled state matches access method")
    func initialEnabledStateMatchesAccessMethod() {
        var method = PersistentAccessMethod.makePersistentAccessMethod()
        method.isEnabled = true

        let (sut, _) = makeSUT(accessMethod: method)

        #expect(sut.isEnabled)
    }

    @Test("Name returns access method name")
    func nameReturnsAccessMethodName() {
        let method = PersistentAccessMethod.makePersistentAccessMethod()

        let (sut, _) = makeSUT(accessMethod: method)

        #expect(sut.name == method.name)
    }

    // MARK: - Enabled State

    @Test("Changing enabled state updates interactor")
    func changingEnabledStateUpdatesInteractor() {
        var method = PersistentAccessMethod.makePersistentAccessMethod()
        method.isEnabled = false

        let (sut, interactor) = makeSUT(accessMethod: method)
        sut.isEnabled = true

        #expect(interactor.setEnabledCalled)
        #expect(interactor.lastEnabledValue == true)
    }

    @Test("Setting same enabled state does not update interactor")
    func settingSameEnabledStateDoesNotUpdateInteractor() {
        var method = PersistentAccessMethod.makePersistentAccessMethod()
        method.isEnabled = true

        let (sut, interactor) = makeSUT(accessMethod: method)

        sut.isEnabled = true

        #expect(!interactor.setEnabledCalled)
    }

    @Test("Disabling access method updates interactor")
    func disablingAccessMethodUpdatesInteractor() {
        var method = PersistentAccessMethod.makePersistentAccessMethod()
        method.isEnabled = true

        let (sut, interactor) = makeSUT(accessMethod: method)

        sut.isEnabled = false

        #expect(interactor.setEnabledCalled)
        #expect(interactor.lastEnabledValue == false)
    }

    // MARK: - Editable State

    @Test("Editable state reflects whether access method is permanent")
    func editableStateReflectsAccessMethodKind() {
        let method = PersistentAccessMethod.makePersistentAccessMethod()
        let (sut, _) = makeSUT(accessMethod: method)

        #expect(sut.isEditable == !sut.accessMethod.kind.isPermanent)
    }

    // MARK: - Disabled State

    @Test("Disabled state reflects interactor last active access method")
    func disabledStateReflectsInteractor() {
        let method = PersistentAccessMethod.makePersistentAccessMethod()
        let (sut, interactor) = makeSUT(accessMethod: method)

        interactor.isLastActive = true

        #expect(sut.isDisabled)

        interactor.isLastActive = false

        #expect(!sut.isDisabled)
    }

    // MARK: - Information Configuration
    @Test("Unknown access method has no information header configuration")
    func unknownAccessMethodHasNoInfo() {
        let method = PersistentAccessMethod.makeCustomAccessMethod()
        let (sut, _) = makeSUT(accessMethod: method)

        #expect(sut.infoHeaderConfig == nil)
        #expect(sut.infoModalConfig == nil)
    }

    // MARK: - Access Method Publisher
    @Test("Access method updates when publisher emits a new value")
    func accessMethodUpdatesFromPublisher() async {
        let method = PersistentAccessMethod.makePersistentAccessMethod()
        let (sut, interactor) = makeSUT(accessMethod: method)

        var updatedMethod = method
        updatedMethod.isEnabled = !method.isEnabled

        interactor.sendAccessMethod(updatedMethod)

        #expect(sut.accessMethod == updatedMethod)
    }

    // MARK: - Start Testing
    @Test("Successful testing changes state to reachable")
    func successfulTestingChangesStateToReachable() async {
        let method = PersistentAccessMethod.makePersistentAccessMethod()
        let (sut, interactor) = makeSUT(accessMethod: method)
        interactor.testResult = .success(())

        await sut.startTesting()

        #expect(sut.testState == .reachable)
        #expect(interactor.startTestingCalled)
    }

    @Test("Failed testing changes state to unreachable")
    func failedTestingChangesStateToUnreachable() async {
        let method = PersistentAccessMethod.makePersistentAccessMethod()
        let (sut, interactor) = makeSUT(accessMethod: method)
        interactor.testResult = .failure(MockEditAccessMethodError.failed)
        await sut.startTesting()
        #expect(sut.testState == .unreachable)
    }

    // MARK: - Delete

    @Test("Delete calls interactor with access method")
    func deleteCallsInteractor() {
        let method = PersistentAccessMethod.makePersistentAccessMethod()
        let (sut, interactor) = makeSUT(accessMethod: method)

        sut.delete()

        #expect(interactor.deleteCalled)
        #expect(interactor.deletedMethod == method)
    }
}
