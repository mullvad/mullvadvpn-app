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
    var title: LocalizedStringKey { get }
    var saveButtonTitle: LocalizedStringKey { get }
    var dismissButtonTitle: LocalizedStringKey { get }
    var dismissButtonImage: Image? { get }
    var mode: ConfigureApiAccessMode { get }
    var accessMethodViewModel: AccessMethodViewModel { get set }
    var testState: ApiAccessMethodTestState { get }
    var validationState: AccessMethodValidationState { get }
    var ciphers: [String] { get }
    var hasInvalidCipher: Bool { get }
    func stopTesting()
    func save()
}

enum ConfigureApiAccessMode {
    case add
    case edit
}

enum AccessMethodValidationState: Equatable {
    case valid
    case invalid(String)
}

@Observable
class ConfigureApiAccessMethodViewModel: ConfigureApiAccessMethodViewModelProtocol {
    var testState: ApiAccessMethodTestState = .idle
    var validationState: AccessMethodValidationState = .valid

    let logger = Logger(label: "ConfigureApiAccessMethodViewModel")

    let mode: ConfigureApiAccessMode
    var accessMethodViewModel: AccessMethodViewModel
    let interactor: EditAccessMethodInteractorProtocol

    init(
        usage: ConfigureApiAccessMode,
        accessMethodViewModel: AccessMethodViewModel,
        interactor: EditAccessMethodInteractorProtocol
    ) {
        self.mode = usage
        self.interactor = interactor
        self.accessMethodViewModel = accessMethodViewModel

    }

    var title: LocalizedStringKey {
        mode == .add ? "Add method" : "Edit method"
    }

    var dismissButtonTitle: LocalizedStringKey {
        mode == .add ? "Cancel" : "Back"
    }

    var dismissButtonImage: Image? {
        mode == .add ? nil : Image(.iconBack)
    }

    var saveButtonTitle: LocalizedStringKey {
        mode == .add ? "Add" : "Save"
    }

    var ciphers: [String] {
        interactor.shadowsocksCiphers
    }

    var hasInvalidCipher: Bool {
        interactor.hasMethodsWithInvalidCiphers
    }

    func save() {
        do {
            let persistentAccessMethod = try accessMethodViewModel.intoPersistentAccessMethod(
                shadowsocksCiphers: ciphers)

            validationState = .valid
            testState = .testing
            interactor.startTesting(persistentAccessMethod) { [weak self] result in
                Task { @MainActor in
                    guard let self else { return }
                    switch result {
                    case .success:
                        self.testState = .reachable
                        self.interactor.save(persistentAccessMethod)
                    case .failure:
                        self.testState = .unreachable
                    }
                }
            }

        } catch let error as AccessMethodValidationError {
            validationState = .invalid(error.localizedDescription)
        } catch {
            logger.error(error: error)
        }
    }

    func stopTesting() {
        interactor.stopTesting()
    }

}
