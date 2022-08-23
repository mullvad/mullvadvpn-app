//
//  ShortcutsViewController.swift
//  MullvadVPN
//
//  Created by Nikolay Davydov on 20.08.2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import IntentsUI
import UIKit

final class ShortcutsViewController: UITableViewController,
    ShortcutsDataSourceDelegate,
    INUIAddVoiceShortcutViewControllerDelegate,
    INUIEditVoiceShortcutViewControllerDelegate
{
    private let dataSource = ShortcutsDataSource()

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    init() {
        super.init(style: .grouped)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        tableView.backgroundColor = .secondaryColor
        tableView.separatorColor = .secondaryColor
        tableView.rowHeight = UITableView.automaticDimension
        tableView.estimatedRowHeight = 60

        dataSource.tableView = tableView
        dataSource.delegate = self

        navigationItem.title = NSLocalizedString(
            "NAVIGATION_TITLE",
            tableName: "Shortcuts",
            value: "Shortcuts",
            comment: ""
        )
    }

    // MARK: - ShortcutsDataSourceDelegate

    func shortcutsDataSource(
        _ dataSource: ShortcutsDataSource,
        didSelectItem item: ShortcutsDataSource.Item
    ) {
        let controller: UIViewController
        if let voiceShortcut = item.voiceShortcut {
            let editShortcutController = INUIEditVoiceShortcutViewController(
                voiceShortcut: voiceShortcut
            )
            editShortcutController.delegate = self
            controller = editShortcutController
        } else {
            let addShortcutController = INUIAddVoiceShortcutViewController(
                shortcut: item.shortcut
            )
            addShortcutController.delegate = self
            controller = addShortcutController
        }
        controller.modalPresentationStyle = .formSheet
        present(controller, animated: true)
    }

    // MARK: - INUIAddVoiceShortcutViewControllerDelegate

    func addVoiceShortcutViewController(
        _ controller: INUIAddVoiceShortcutViewController,
        didFinishWith voiceShortcut: INVoiceShortcut?,
        error: Error?
    ) {
        if let voiceShortcut = voiceShortcut {
            ShortcutsManager.shared.addVoiceShortcut(voiceShortcut)
        }
        controller.dismiss(animated: true)
    }

    func addVoiceShortcutViewControllerDidCancel(_ controller: INUIAddVoiceShortcutViewController) {
        controller.dismiss(animated: true)
    }

    // MARK: - INUIEditVoiceShortcutViewControllerDelegate

    func editVoiceShortcutViewController(
        _ controller: INUIEditVoiceShortcutViewController,
        didUpdate voiceShortcut: INVoiceShortcut?,
        error: Error?
    ) {
        controller.dismiss(animated: true)
    }

    func editVoiceShortcutViewController(
        _ controller: INUIEditVoiceShortcutViewController,
        didDeleteVoiceShortcutWithIdentifier deletedVoiceShortcutIdentifier: UUID
    ) {
        ShortcutsManager.shared.deleteVoiceShortcut(
            withIdentifier: deletedVoiceShortcutIdentifier
        )
        controller.dismiss(animated: true)
    }

    func editVoiceShortcutViewControllerDidCancel(
        _ controller: INUIEditVoiceShortcutViewController
    ) {
        controller.dismiss(animated: true)
    }
}
