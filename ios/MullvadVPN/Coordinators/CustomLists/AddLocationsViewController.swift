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
import MullvadSettings
import MullvadTypes
import UIKit

protocol AddLocationsViewControllerDelegate: AnyObject, Sendable {
    func didBack()
}

class AddLocationsViewController: UIViewController {
    private var dataSource: AddLocationsDataSource?
    private let nodes: [LocationNode]
    private let subject: CurrentValueSubject<CustomListViewModel, Never>

    weak var delegate: AddLocationsViewControllerDelegate?
    private let tableView: UITableView = {
        let tableView = UITableView()
        tableView.separatorColor = .secondaryColor
        tableView.separatorInset = .zero
        tableView.estimatedRowHeight = UIMetrics.TableView.rowHeight
        tableView.rowHeight = UITableView.automaticDimension
        tableView.indicatorStyle = .white
        tableView.setAccessibilityIdentifier(.editCustomListEditLocationsTableView)
        return tableView
    }()

    init(
        allLocationsNodes: [LocationNode],
        subject: CurrentValueSubject<CustomListViewModel, Never>
    ) {
        self.nodes = allLocationsNodes
        self.subject = subject
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        view.setAccessibilityIdentifier(.editCustomListEditLocationsView)
        tableView.backgroundColor = view.backgroundColor
        view.backgroundColor = .secondaryColor
        addConstraints()
        setUpDataSource()
    }

    override func didMove(toParent parent: UIViewController?) {
        super.didMove(toParent: parent)

        if parent == nil {
            delegate?.didBack()
        }
    }

    private func addConstraints() {
        view.addConstrainedSubviews([tableView]) {
            tableView.pinEdgesToSuperview()
        }
    }

    private func setUpDataSource() {
        dataSource = AddLocationsDataSource(
            tableView: tableView,
            allLocationNodes: nodes.copy(),
            subject: subject
        )
    }
}

fileprivate extension [LocationNode] {
    func copy() -> Self {
        map {
            let copy = $0.copy()
            copy.flattened.forEach { $0.showsChildren = false }
            return copy
        }
    }
}
