// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import MullvadTypes
import SwiftUI

@MainActor
protocol CustomListViewModel: Observable, AnyObject {
    var customList: CustomList? { get }
    var availableLocations: CustomListSelectableLocationNode { set get }
    var selectionHasChanged: Bool { get }
    var customListName: String { set get }
    var error: CustomRelayListError? { get }

    func delete()
    func save() -> Bool
}

extension CustomListViewModel {
    var hasUnsavedChanges: Bool {
        selectionHasChanged || (customList?.name ?? "" != customListName)
    }

    var isNewList: Bool {
        customList == nil
    }
}

@Observable
@MainActor
class CustomListViewModelImpl: CustomListViewModel {
    var customList: CustomList?

    var availableLocations: CustomListSelectableLocationNode

    var selectionHasChanged: Bool {
        let currentLocationSelection = availableLocations.flattened.compactMap {
            ($0.asCustomListSelectableLocationNode)?.isCustomListSelected
        }
        return currentLocationSelection != initiallyLocationSelection
    }

    var customListName: String {
        didSet {
            if customListName.count > NameInputFormatter.maxLength {
                error = .nameTooLong
            } else if existingNames.contains(customListName) {
                error = .duplicateName
            } else {
                error = nil
            }
        }
    }

    private(set) var error: CustomRelayListError? = nil

    // Used to check if there is unsaved changes
    private let initiallyLocationSelection: [Bool]

    private let existingNames: [String]
    private let interactor: CustomListInteractorProtocol
    private let onDidSave: () -> Void

    init(
        customList: CustomList?,
        interactor: CustomListInteractorProtocol,
        context: MultihopContext,
        onDidSave: @escaping () -> Void
    ) {

        self.customListName = customList?.name ?? ""
        let availableLocations = CustomListSelectableLocationNode(
            fromLocationNodes: interactor.availableLocations(context: context)
        )
        if let customList {
            self.customList = customList
            Self.setupAvailableLocations(availableLocations, customList)
        }
        self.availableLocations = availableLocations
        self.initiallyLocationSelection = availableLocations.flattened.compactMap {
            ($0.asCustomListSelectableLocationNode)?.isCustomListSelected
        }
        self.interactor = interactor
        self.existingNames =
            interactor.fetchAll()
            .map(\.name)
            .filter { $0 != customList?.name }
        self.onDidSave = onDidSave
    }

    private static func setupAvailableLocations(
        _ availableLocations: CustomListSelectableLocationNode,
        _ customList: CustomList
    ) {
        let customListNode = CustomListLocationNodeBuilder(
            customList: customList,
            allLocations: availableLocations.children
        )
        customListNode.customListLocationNode
            .forEachDescendant { selectedLocation in
                availableLocations
                    .descendants
                    .filter {
                        $0 == selectedLocation
                    }
                    .compactMap { $0.asCustomListSelectableLocationNode }
                    .forEach {
                        $0.isCustomListSelected = true
                    }
            }
        availableLocations.forEachDescendant {
            guard let node = $0.asCustomListSelectableLocationNode else {
                return
            }

            if !node.isCustomListSelected {
                let hasSelectedChild = node.flattened.contains {
                    ($0.asCustomListSelectableLocationNode)?.isCustomListSelected ?? false
                }
                node.showsChildren = hasSelectedChild
            }
        }
    }

    func delete() {
        guard let customList else { return }
        interactor.delete(customList: customList)
        onDidSave()
    }

    func save() -> Bool {
        let locations = availableLocations
            .flattened
            .compactMap { $0.asCustomListSelectableLocationNode }
            .filter {
                $0.isCustomListSelected
            }
            .filter {
                if let parent = $0.parent?.asCustomListSelectableLocationNode {
                    !parent.isCustomListSelected
                } else {
                    true
                }
            }
            .flatMap { $0.locations }
        var customList =
            customList
            ?? .init(
                name: customListName,
                locations: locations
            )
        customList.name = customListName
        customList.locations = locations
        do {
            try interactor.save(list: customList)
            onDidSave()
            return true
        } catch is CustomRelayListError {
            self.error = error
        } catch {
            // This should never happen
        }
        return false
    }
}

fileprivate extension LocationNode {
    var codeWithoutCustomList: String {
        code.split(separator: "-").dropFirst().joined(separator: "-")
    }
}

class MockCustomListViewModel: CustomListViewModel {
    let customList: CustomList?

    lazy var availableLocations: CustomListSelectableLocationNode = {
        CustomListSelectableLocationNode(
            fromLocationNode: RootLocationNode(
                children:
                    MockSelectLocationViewModel().entryContext.locations))
    }()

    let selectionHasChanged: Bool = false

    var customListName: String = "MyList"

    let error: CustomRelayListError?

    init(customList: CustomList? = nil, error: CustomRelayListError? = nil) {
        self.customList = customList
        self.error = error
    }

    func delete() {

    }

    func save() -> Bool {
        return false
    }

}
