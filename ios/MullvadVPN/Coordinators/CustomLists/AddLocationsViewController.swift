//
//  AddLocationsViewController.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-29.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes
import UIKit

protocol AddLocationsViewControllerDelegate: AnyObject {
    func didUpdateSelectedLocations(locations: [RelayLocation])
}

class AddLocationsViewController: UIViewController {
    private var dataSource: AddLocationsDataSource?
    private let nodes: [LocationNode]
    private let customList: CustomList

    weak var delegate: AddLocationsViewControllerDelegate?
    private let tableView: UITableView = {
        let tableView = UITableView()
        tableView.separatorColor = .secondaryColor
        tableView.separatorInset = .zero
        tableView.estimatedRowHeight = 53
        tableView.indicatorStyle = .white
        tableView.accessibilityIdentifier = .addLocationsView
        tableView.allowsMultipleSelection = true
        return tableView
    }()

    init(
        allLocationsNodes: [LocationNode],
        customList: CustomList
    ) {
        self.nodes = allLocationsNodes
        self.customList = customList
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        tableView.backgroundColor = view.backgroundColor
        view.backgroundColor = .secondaryColor
        addConstraints()
        setUpDataSource()
    }

    private func addConstraints() {
        view.addConstrainedSubviews([tableView]) {
            tableView.pinEdgesToSuperview()
        }
    }

    private func setUpDataSource() {
        dataSource = AddLocationsDataSource(
            tableView: tableView,
            allLocations: nodes,
            customList: customList
        )

        dataSource?.didUpdateCustomList = { [weak self] customListLocationNode in
            guard let self else { return }
            delegate?.didUpdateSelectedLocations(
                locations: customListLocationNode.children.reduce([]) { partialResult, locationNode in
                    partialResult + locationNode.locations
                }
            )
        }
    }
}
