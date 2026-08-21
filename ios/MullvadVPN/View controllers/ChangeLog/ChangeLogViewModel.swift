// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadLogging
import SwiftUI

protocol ChangeLogViewModelProtocol: ObservableObject {
    var changeLog: ChangeLogModel? { get }
    func getLatestChanges()
}

class ChangeLogViewModel: ChangeLogViewModelProtocol {
    private let logger = Logger(label: "ChangeLogViewModel")
    private let changeLogReader: ChangeLogReaderProtocol

    @Published var changeLog: ChangeLogModel?

    init(changeLogReader: ChangeLogReaderProtocol) {
        self.changeLogReader = changeLogReader
    }

    func getLatestChanges() {
        do {
            changeLog = ChangeLogModel(title: Bundle.main.productVersion, changes: try changeLogReader.read())
        } catch {
            logger.error(error: error, message: "Cannot read change log from bundle.")
        }
    }
}

class MockChangeLogViewModel: ChangeLogViewModelProtocol {
    @Published var changeLog: ChangeLogModel?
    func getLatestChanges() {
        changeLog = ChangeLogModel(
            title: "2025.1",
            changes: [
                "Introduced a dark mode for better accessibility and user experience.",
                "Added two-factor authentication (2FA) for all user accounts.",
            ])
    }
}
