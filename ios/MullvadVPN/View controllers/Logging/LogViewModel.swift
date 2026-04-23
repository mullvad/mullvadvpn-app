//
//  LogViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-04-23.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging

final class LogViewModel {
    var didAddEntry: ((String) -> Void)?
    private let observer: InAppLogBlockObserver

    init(observer: InAppLogBlockObserver) {
        self.observer = observer

        self.observer.didAddLogEntryHandler = { [weak self] entry in
            self?.didAddEntry?(entry)
        }
    }
}
