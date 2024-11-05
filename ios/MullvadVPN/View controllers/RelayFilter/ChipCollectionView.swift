//
//  ChipCollectionView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-10-31.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class ChipCollectionView: UIView {
    private var chips: [ChipConfiguration] = []
    private let cellReuseIdentifier = String(describing: ChipViewCell.self)

    private(set) lazy var collectionView: UICollectionView = {
        let collectionView = UICollectionView(frame: .zero, collectionViewLayout: ChipFlowLayout())
        collectionView.contentInset = .zero
        collectionView.backgroundColor = .clear
        collectionView.translatesAutoresizingMaskIntoConstraints = false
        return collectionView
    }()

    init() {
        super.init(frame: .zero)
        setupCollectionView()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        setupCollectionView()
    }

    private func setupCollectionView() {
        collectionView.dataSource = self
        collectionView.register(UICollectionViewCell.self, forCellWithReuseIdentifier: cellReuseIdentifier)
        addConstrainedSubviews([collectionView]) {
            collectionView.pinEdgesToSuperview()
        }
    }

    func setChips(_ values: [ChipConfiguration]) {
        chips = values
        collectionView.reloadData()
    }
}

extension ChipCollectionView: UICollectionViewDataSource {
    func collectionView(_ collectionView: UICollectionView, numberOfItemsInSection section: Int) -> Int {
        return chips.count
    }

    func collectionView(
        _ collectionView: UICollectionView,
        cellForItemAt indexPath: IndexPath
    ) -> UICollectionViewCell {
        let cell = collectionView.dequeueReusableCell(withReuseIdentifier: cellReuseIdentifier, for: indexPath)
        cell.contentConfiguration = chips[indexPath.row]
        return cell
    }
}
