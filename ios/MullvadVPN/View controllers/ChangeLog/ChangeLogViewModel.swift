//
//  ChangeLogViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-22.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

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
        changeLog = ChangeLogModel(title: "2025.1", changes: [
            "Introduced a dark mode for better accessibility and user experience.",
            "Added two-factor authentication (2FA) for all user accounts.",
        ])
    }
}
