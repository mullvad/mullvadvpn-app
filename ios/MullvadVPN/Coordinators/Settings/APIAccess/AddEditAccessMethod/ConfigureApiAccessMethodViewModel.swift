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

    // MARK: - Presentation

    var title: LocalizedStringKey { get }
    var dismissButtonImage: Image { get }
    var mode: ConfigureApiAccessMode { get }

    // MARK: - Data

    var accessMethodViewModel: AccessMethodViewModel { get set }
    var ciphers: [String] { get }
    var hasInvalidCipher: Bool { get }

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

enum ConfigureApiAccessMode {
    case add
    case edit
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

    let mode: ConfigureApiAccessMode
    var accessMethodViewModel: AccessMethodViewModel
    let interactor: EditAccessMethodInteractorProtocol

    init(
        mode: ConfigureApiAccessMode,
        accessMethodViewModel: AccessMethodViewModel,
        interactor: EditAccessMethodInteractorProtocol
    ) {
        self.mode = mode
        self.interactor = interactor
        self.accessMethodViewModel = accessMethodViewModel

    }

    var title: LocalizedStringKey {
        mode == .add ? "Add method" : "Edit method"
    }

    var dismissButtonImage: Image {
        mode == .add ? Image(.iconCross) : Image(.iconChevronLeft)
    }

    var ciphers: [String] {
        interactor.shadowsocksCiphers
    }

    var hasInvalidCipher: Bool {
        interactor.hasMethodsWithInvalidCiphers
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
