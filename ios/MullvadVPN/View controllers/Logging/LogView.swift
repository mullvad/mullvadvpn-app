//
//  LogView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-04-23.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import UIKit

class LogView: UIView {
    private let maxPanelHeight: CGFloat
    private let minPanelHeight: CGFloat = 120
    private var panelHeight: CGFloat = 230
    private let safeTop: CGFloat = 60
    private let safeBottom: CGFloat = 40

    private let viewModel: LogViewModel
    private let topHandleView = UIView()
    private let bottomHandleView = UIView()
    private let bottomHandleBar = UIView()
    private let clearButton = IncreasedHitButton()
    private let searchField = UITextField()
    private let tableView = UITableView(frame: .zero, style: .plain)

    private var entries: [String] = []
    private var filteredEntries: [String] = []
    private var searchText = ""
    private var dragStartY: CGFloat = 0
    private var resizeStartHeight: CGFloat = 0

    // Snaps to top, middle and bottom.
    private lazy var snapYPositions: [CGFloat] = {
        let screenHeight = UIScreen.main.bounds.height
        return [safeTop, (screenHeight - panelHeight) / 2, screenHeight - panelHeight - safeBottom]
    }()

    init(viewModel: LogViewModel) {
        self.viewModel = viewModel
        maxPanelHeight = UIScreen.main.bounds.height - safeTop - safeBottom

        super.init(frame: .zero)

        setUp()
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setUp() {
        frame = CGRect(
            x: 8,
            y: snapYPositions.first!,
            width: UIScreen.main.bounds.width - 16,
            height: panelHeight
        )

        backgroundColor = .black.withAlphaComponent(0.7)
        layer.cornerRadius = 12
        clipsToBounds = true

        viewModel.didAddEntry = addEntry

        setUpTopHandle()
        setUpClearButton()
        setUpSearchField()
        setUpTableView()
        setUpBottomHandle()

        addGestureRecognizer(UIPanGestureRecognizer(target: self, action: #selector(handlePan(_:))))
    }

    private func setUpTopHandle() {
        topHandleView.backgroundColor = .white.withAlphaComponent(0.5)
        topHandleView.layer.cornerRadius = 2

        addConstrainedSubviews([topHandleView]) {
            topHandleView.pinEdgeToSuperview(.top(8))
            topHandleView.centerXAnchor.constraint(equalTo: centerXAnchor)
            topHandleView.widthAnchor.constraint(equalToConstant: 36)
            topHandleView.heightAnchor.constraint(equalToConstant: 4)
        }
    }

    private func setUpClearButton() {
        clearButton.setImage(.cross.withRenderingMode(.alwaysOriginal).withTintColor(.white), for: .normal)
        clearButton.addTarget(self, action: #selector(handleClearButton(_:)), for: .touchUpInside)

        addConstrainedSubviews([clearButton]) {
            clearButton.pinEdgeToSuperview(.trailing(8))
            clearButton.centerYAnchor.constraint(equalTo: topHandleView.centerYAnchor)
            clearButton.widthAnchor.constraint(equalToConstant: 16)
            clearButton.heightAnchor.constraint(equalToConstant: 16)
        }
    }

    private func setUpSearchField() {
        let placeholder = "Search logs..."
        searchField.placeholder = placeholder
        searchField.attributedPlaceholder = NSAttributedString(
            string: placeholder,
            attributes: [.foregroundColor: UIColor.white.withAlphaComponent(0.4)]
        )

        searchField.font = .monospacedSystemFont(ofSize: 11, weight: .regular)
        searchField.textColor = .white
        searchField.backgroundColor = .white.withAlphaComponent(0.1)
        searchField.layer.cornerRadius = 6
        searchField.leftView = UIView(frame: CGRect(x: 0, y: 0, width: 8, height: 0))
        searchField.leftViewMode = .always
        searchField.returnKeyType = .search
        searchField.autocorrectionType = .no
        searchField.autocapitalizationType = .none

        let clearImage = UIImage(systemName: "xmark.circle.fill")?
            .withConfiguration(UIImage.SymbolConfiguration(pointSize: 10))
        let clearButton = UIButton(type: .system)
        clearButton.setImage(clearImage, for: .normal)
        clearButton.tintColor = .white.withAlphaComponent(0.5)
        clearButton.addTarget(self, action: #selector(clearSearchField), for: .touchUpInside)
        clearButton.sizeToFit()

        let clearButtonContainer = UIView(
            frame: CGRect(x: 0, y: 0, width: clearButton.frame.width + 16, height: clearButton.frame.height)
        )
        clearButton.frame.origin = .init(x: 8, y: 0)
        clearButtonContainer.addSubview(clearButton)

        searchField.rightView = clearButtonContainer
        searchField.rightViewMode = .whileEditing

        searchField.addTarget(self, action: #selector(searchTextChanged), for: .editingChanged)
        searchField.delegate = self

        addConstrainedSubviews([searchField]) {
            searchField.topAnchor.constraint(equalTo: topHandleView.bottomAnchor, constant: 8)
            searchField.pinEdgesToSuperview(.init([.leading(8), .trailing(8)]))
            searchField.heightAnchor.constraint(equalToConstant: 28)
        }
    }

    private func setUpTableView() {
        tableView.dataSource = self
        tableView.delegate = self
        tableView.register(UITableViewCell.self, forCellReuseIdentifier: "LogCell")

        tableView.backgroundColor = .clear
        tableView.separatorStyle = .none
        tableView.showsVerticalScrollIndicator = true
        tableView.indicatorStyle = .white
        tableView.verticalScrollIndicatorInsets.right = 2

        addConstrainedSubviews([tableView]) {
            tableView.pinEdgesToSuperview(.all().excluding(.top).excluding(.bottom))
            tableView.topAnchor.constraint(equalTo: searchField.bottomAnchor, constant: 4)
        }
    }

    private func setUpBottomHandle() {
        bottomHandleView.backgroundColor = .clear

        addConstrainedSubviews([bottomHandleView]) {
            bottomHandleView.topAnchor.constraint(equalTo: tableView.bottomAnchor)
            bottomHandleView.pinEdgesToSuperview(.all().excluding(.top))
            bottomHandleView.heightAnchor.constraint(equalToConstant: 20)
        }

        bottomHandleBar.backgroundColor = .white.withAlphaComponent(0.5)
        bottomHandleBar.layer.cornerRadius = 2

        bottomHandleView.addConstrainedSubviews([bottomHandleBar]) {
            bottomHandleBar.centerXAnchor.constraint(equalTo: bottomHandleView.centerXAnchor)
            bottomHandleBar.centerYAnchor.constraint(equalTo: bottomHandleView.centerYAnchor)
            bottomHandleBar.widthAnchor.constraint(equalToConstant: 36)
            bottomHandleBar.heightAnchor.constraint(equalToConstant: 4)
        }

        let resizeGesture = UIPanGestureRecognizer(target: self, action: #selector(handleResize(_:)))
        bottomHandleView.addGestureRecognizer(resizeGesture)
    }

    private func addEntry(_ entry: String) {
        entries.append(entry)

        if matchesFilter(entry) {
            filteredEntries.append(entry)

            let indexPath = IndexPath(row: filteredEntries.count - 1, section: 0)
            tableView.insertRows(at: [indexPath], with: .none)

            scrollToBottom()
        }
    }

    private func clearEntries() {
        entries.removeAll()
        filteredEntries.removeAll()

        tableView.reloadData()
        scrollToBottom()
    }

    private func applyFilter() {
        filteredEntries = entries.filter { matchesFilter($0) }

        tableView.reloadData()
        scrollToBottom()
    }

    private func matchesFilter(_ entry: String) -> Bool {
        searchText.isEmpty || entry.localizedCaseInsensitiveContains(searchText)
    }

    private func scrollToBottom() {
        guard filteredEntries.count > 0 else { return }

        let indexPath = IndexPath(row: filteredEntries.count - 1, section: 0)
        tableView.scrollToRow(at: indexPath, at: .bottom, animated: true)
    }

    // MARK: - Actions

    @objc private func handleClearButton(_ sender: UIButton) {
        clearEntries()
    }

    @objc private func searchTextChanged() {
        searchText = searchField.text ?? ""
        applyFilter()
    }

    @objc private func clearSearchField() {
        searchField.text = ""
        searchTextChanged()
    }

    // MARK: - Resizing

    @objc private func handleResize(_ gesture: UIPanGestureRecognizer) {
        switch gesture.state {
        case .began:
            resizeStartHeight = frame.height
        case .changed:
            let translation = gesture.translation(in: superview).y
            let newHeight = min(max(resizeStartHeight + translation, minPanelHeight), maxPanelHeight)

            frame.size.height = newHeight
            panelHeight = newHeight
        case .ended, .cancelled:
            scrollToBottom()
        default:
            break
        }
    }

    // MARK: - Dragging

    @objc private func handlePan(_ gesture: UIPanGestureRecognizer) {
        switch gesture.state {
        case .began:
            dragStartY = frame.origin.y
        case .changed:
            let translation = gesture.translation(in: superview).y
            frame.origin.y = dragStartY + translation
        case .ended, .cancelled:
            let currentY = frame.origin.y
            let targetY = snapYPositions.min(by: { abs($0 - currentY) < abs($1 - currentY) }) ?? safeTop

            UIView.animate(
                withDuration: 0.35,
                delay: 0,
                usingSpringWithDamping: 0.8,
                initialSpringVelocity: 0,
                options: .curveEaseOut
            ) {
                self.frame.origin.y = targetY
            }
        default:
            break
        }
    }
}

extension LogView: UITableViewDataSource {
    func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        filteredEntries.count
    }

    func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let cell = tableView.dequeueReusableCell(withIdentifier: "LogCell", for: indexPath)

        var config = cell.defaultContentConfiguration()
        config.text = filteredEntries[indexPath.row]
        config.textProperties.font = .monospacedSystemFont(ofSize: 11, weight: .regular)
        config.textProperties.color = .white

        cell.contentConfiguration = config
        cell.backgroundColor = .clear
        cell.selectionStyle = .none

        return cell
    }
}

extension LogView: UITableViewDelegate {
    func tableView(_ tableView: UITableView, shouldShowMenuForRowAt indexPath: IndexPath) -> Bool {
        true
    }

    func tableView(
        _ tableView: UITableView,
        canPerformAction action: Selector,
        forRowAt indexPath: IndexPath,
        withSender sender: Any?
    ) -> Bool {
        action == #selector(copy(_:))
    }

    func tableView(
        _ tableView: UITableView,
        performAction action: Selector,
        forRowAt indexPath: IndexPath,
        withSender sender: Any?
    ) {
        UIPasteboard.general.string = filteredEntries[indexPath.row]
    }
}

extension LogView: UITextFieldDelegate {
    func textFieldShouldReturn(_ textField: UITextField) -> Bool {
        textField.resignFirstResponder()
        return true
    }
}
