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

    var accessMethodViewModel: AccessMethodViewModel { get set }
    var ciphers: [String] { get }
    var hasInvalidCipher: Bool { get }
    var canSubmit: Bool { get }

    // MARK: - State

    var testState: ApiAccessMethodTestState { get }
    var validationState: AccessMethodValidationState { get }

    // MARK: - Validation

    func hasError(for field: AccessMethodFieldValidationError.Field) -> AccessMethodFieldValidationError?
    func borderStyle(for field: AccessMethodFieldValidationError.Field) -> BorderStyle
    func clearErrors(for fields: Set<AccessMethodFieldValidationError.Field>)

    // MARK: - Actions

    func save()
    func startTesting()
    func stopTesting()
}

enum AccessMethodValidationState {
    case valid
    case invalid(AccessMethodValidationError)
}

@Observable
final class ConfigureApiAccessMethodViewModel: ConfigureApiAccessMethodViewModelProtocol {
    private(set) var testState: ApiAccessMethodTestState = .idle
    private(set) var validationState: AccessMethodValidationState = .valid

    let logger = Logger(label: "ConfigureApiAccessMethodViewModel")

    var accessMethodViewModel: AccessMethodViewModel
    let interactor: EditAccessMethodInteractorProtocol

    init(
        accessMethodViewModel: AccessMethodViewModel,
        interactor: EditAccessMethodInteractorProtocol
    ) {
        self.interactor = interactor
        self.accessMethodViewModel = accessMethodViewModel
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
        } catch let error as AccessMethodValidationError {
            return !error.fieldErrors.contains { $0.kind == .emptyValue }
        } catch {
            return true
        }
        return true
    }

    func clearErrors(for fields: Set<AccessMethodFieldValidationError.Field>) {
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

    func hasError(for field: AccessMethodFieldValidationError.Field) -> AccessMethodFieldValidationError? {
        guard case .invalid(let validationError) = validationState else {
            return nil
        }

        return validationError.fieldErrors.first { $0.field == field }
    }

    func borderStyle(for field: AccessMethodFieldValidationError.Field) -> BorderStyle {
        hasError(for: field) != nil ? .error : .none
    }

    func startTesting() {
        guard let persistentAccessMethod = toPersistentAccessMethod() else { return }
        testState = .testing
        interactor.startTesting(persistentAccessMethod) { [weak self] result in
            Task { @MainActor in
                guard let self else { return }
                switch result {
                case .success:
                    self.testState = .reachable
                case .failure:
                    self.testState = .unreachable
                }
            }
        }
    }

    func save() {
        guard let persistentAccessMethod = toPersistentAccessMethod() else { return }
        interactor.save(persistentAccessMethod)
    }

    func stopTesting() {
        testState = .idle
        interactor.stopTesting()
    }

    private func toPersistentAccessMethod() -> PersistentAccessMethod? {
        do {
            validationState = .valid
            return try accessMethodViewModel.intoPersistentAccessMethod(shadowsocksCiphers: ciphers)
        } catch let error as AccessMethodValidationError {
            validationState = .invalid(error)
        } catch {
            logger.error(error: error)
        }
        return nil
    }
}
