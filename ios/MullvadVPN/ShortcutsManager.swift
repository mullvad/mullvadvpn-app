//
//  ShortcutsManager.swift
//  MullvadVPN
//
//  Created by Nikolay Davydov on 23.08.2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import IntentsUI
import Logging

protocol ShortcutsManagerDelegate: AnyObject {
    func shortcutsManager(
        _ shortcutsManager: ShortcutsManager,
        didReceiveVoiceShortcuts voiceShortcuts: [INVoiceShortcut]
    )
}

final class ShortcutsManager {
    static let shared = ShortcutsManager()

    private init() {}

    private let logger = Logger(label: "ShortcutsManager")

    private var voiceShortcutsByID = [UUID: INVoiceShortcut]() {
        didSet {
            let voiceShortcuts = voiceShortcutsByID.map { $0.value }
            delegate?.shortcutsManager(self, didReceiveVoiceShortcuts: voiceShortcuts)
        }
    }

    weak var delegate: ShortcutsManagerDelegate?

    func updateVoiceShortcuts() {
        guard delegate != nil else { return }
        INVoiceShortcutCenter.shared.getAllVoiceShortcuts { [weak self] voiceShortcuts, error in
            guard let self = self else { return }
            if let error = error {
                self.logger.error(
                    error: error,
                    message: "Failed to fetch voice shortcuts."
                )
                return
            }
            let voiceShortcuts = voiceShortcuts ?? []
            let voiceShortcutsByID = voiceShortcuts
                .reduce(into: [UUID: INVoiceShortcut]()) { result, voiceShortcut in
                    result[voiceShortcut.identifier] = voiceShortcut
                }
            DispatchQueue.main.async {
                self.voiceShortcutsByID = voiceShortcutsByID
            }
        }
    }

    func addVoiceShortcut(_ voiceShortcut: INVoiceShortcut) {
        voiceShortcutsByID[voiceShortcut.identifier] = voiceShortcut
    }

    func deleteVoiceShortcut(withIdentifier identifier: UUID) {
        voiceShortcutsByID[identifier] = nil
    }
}
