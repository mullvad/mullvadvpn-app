// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadLogging
import MullvadSettings
import MullvadTypes
import SwiftUI

@MainActor
protocol ConfigureApiAccessMethodViewModelProtocol: AnyObject, Observable {

    // MARK: - Data

    var originalAccessMethodViewModel: AccessMethodViewModel { get }
    var accessMethodViewModel: AccessMethodViewModel { get set }
    var hasChanges: Bool { get }
    var ciphers: [String] { get }
    var hasInvalidCipher: Bool { get }
    var canSubmit: Bool { get }
    var isTesting: Bool { get }

    // MARK: - State

    var testState: ApiAccessMethodTestState { get }
    var validationState: AccessMethodValidationState { get }

    // MARK: - Validation

    func hasError(for field: AccessMethodFieldValidationError.Field) -> AccessMethodFieldValidationError?
    func clearErrors(for fields: Set<AccessMethodFieldValidationError.Field>)

    // MARK: - Actions

    func save()
    func discard()
    func startTesting() async
    func cancelTesting() async
}

enum AccessMethodValidationState: Equatable {
    case valid
    case invalid(AccessMethodValidationError)
}

@Observable
@MainActor
final class ConfigureApiAccessMethodViewModel: ConfigureApiAccessMethodViewModelProtocol {

    // MARK: - State

    private(set) var testState: ApiAccessMethodTestState = .idle
    private(set) var validationState: AccessMethodValidationState = .valid

    // MARK: - Dependencies

    let interactor: EditAccessMethodInteractorProtocol
    let logger = Logger(label: "ConfigureApiAccessMethodViewModel")

    // MARK: - Data
    private(set) var originalAccessMethodViewModel: AccessMethodViewModel
    var accessMethodViewModel: AccessMethodViewModel
    var currentTestID: UUID?

    var hasChanges: Bool {
        accessMethodViewModel != originalAccessMethodViewModel
    }

    var isTesting: Bool {
        testState == .testing
    }

    init(
        accessMethodViewModel: AccessMethodViewModel,
        interactor: EditAccessMethodInteractorProtocol
    ) {
        self.originalAccessMethodViewModel = accessMethodViewModel
        self.accessMethodViewModel = accessMethodViewModel
        self.interactor = interactor
    }

    var ciphers: [String] {
        interactor.shadowsocksCiphers
    }

    var hasInvalidCipher: Bool {
        interactor.hasMethodsWithInvalidCiphers
    }

    var canSubmit: Bool {
        do {
            _ = try accessMethodViewModel.intoPersistentAccessMethod(shadowsocksCiphers: ciphers)
            return true
        } catch {
            return false
        }
    }

    // MARK: - Validation

    func hasError(
        for field: AccessMethodFieldValidationError.Field
    ) -> AccessMethodFieldValidationError? {
        guard case .invalid(let validationError) = validationState else {
            return nil
        }

        return validationError.fieldErrors.first { $0.field == field }
    }

    func clearErrors(
        for fields: Set<AccessMethodFieldValidationError.Field>
    ) {
        guard case .invalid(let validationError) = validationState else {
            return
        }

        let remainingErrors = validationError.fieldErrors.filter {
            !fields.contains($0.field)
        }

        validationState =
            remainingErrors.isEmpty
            ? .valid
            : .invalid(
                AccessMethodValidationError(fieldErrors: remainingErrors)
            )
    }

    // MARK: - Actions

    func save() {
        guard let method = validatedAccessMethod() else {
            return
        }

        interactor.save(method)
        originalAccessMethodViewModel = accessMethodViewModel
    }

    func discard() {
        accessMethodViewModel = originalAccessMethodViewModel
    }

    func startTesting() async {
        guard let method = validatedAccessMethod() else {
            return
        }

        let testID = UUID()
        currentTestID = testID
        testState = .testing
        do {
            try await interactor.test(method)
            guard currentTestID == testID else {
                return
            }
            testState = .reachable
        } catch {
            guard currentTestID == testID else {
                return
            }
            testState = .unreachable
        }
    }

    func cancelTesting() async {
        await interactor.cancelTesting()
        testState = .idle
        currentTestID = nil
    }

    // MARK: - Private

    private func validatedAccessMethod() -> PersistentAccessMethod? {
        validationState = .valid

        do {
            return try accessMethodViewModel.intoPersistentAccessMethod(
                shadowsocksCiphers: ciphers
            )
        } catch let error as AccessMethodValidationError {
            validationState = .invalid(error)
        } catch {
            logger.error(error: error)
        }

        return nil
    }
}
