//
//  AlertViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-05-19.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

enum AlertActionStyle {
    case `default`
    case destructive

    fileprivate var buttonStyle: AppButton.Style {
        switch self {
        case .default:
            return .default
        case .destructive:
            return .danger
        }
    }
}

enum AlertIcon {
    case alert
    case warning
    case info
    case spinner

    fileprivate var image: UIImage? {
        switch self {
        case .alert:
            return UIImage.Buttons.alert.withTintColor(.dangerColor)
        case .warning:
            return UIImage.Buttons.alert.withTintColor(.white)
        case .info:
            return UIImage.Buttons.info.withTintColor(.white)
        default:
            return nil
        }
    }
}

class AlertViewController: UIViewController {
    typealias Handler = () -> Void
    var onDismiss: Handler?

    private let scrollView = UIScrollView()
    private var scrollViewHeightConstraint: NSLayoutConstraint!
    private let presentation: AlertPresentation

    private let viewContainer: UIView = {
        let view = UIView()

        view.backgroundColor = .secondaryColor
        view.layer.cornerRadius = 11

        return view
    }()

    private let buttonView: UIStackView = {
        let view = UIStackView()

        view.axis = .vertical
        view.spacing = UIMetrics.CustomAlert.containerSpacing
        view.isLayoutMarginsRelativeArrangement = true
        view.directionalLayoutMargins = UIMetrics.CustomAlert.containerMargins

        return view
    }()

    private let contentView: UIStackView = {
        let view = UIStackView()

        view.axis = .vertical
        view.spacing = UIMetrics.CustomAlert.containerSpacing
        view.isLayoutMarginsRelativeArrangement = true
        view.directionalLayoutMargins = UIMetrics.CustomAlert.containerMargins

        return view
    }()

    private var handlers = [UIButton: Handler]()

    init(presentation: AlertPresentation) {
        self.presentation = presentation

        super.init(nibName: nil, bundle: nil)

        modalPresentationStyle = .overFullScreen
        modalTransitionStyle = .crossDissolve
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()

        view.layoutIfNeeded()
        scrollViewHeightConstraint.constant = scrollView.contentSize.height

        adjustButtonMargins()
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.setAccessibilityIdentifier(presentation.accessibilityIdentifier ?? .alertContainerView)
        view.backgroundColor = .black.withAlphaComponent(0.5)

        setContent()
        setConstraints()
    }

    private func setContent() {
        presentation.icon.flatMap { addIcon($0) }
        presentation.header.flatMap { addHeader($0) }
        presentation.title.flatMap { addTitle($0) }

        if let message = presentation.attributedMessage {
            addMessage(message)
        } else if let message = presentation.message {
            addMessage(message)
        }

        presentation.buttons.forEach { action in
            addAction(
                title: action.title,
                style: action.style,
                accessibilityId: action.accessibilityId,
                handler: action.handler
            )
        }

        // Icon only spinner alerts should have no background and equal top and bottom margins.
        if presentation.icon == .spinner, contentView.arrangedSubviews.count == 1 {
            viewContainer.backgroundColor = .clear
            contentView.directionalLayoutMargins.bottom = UIMetrics.CustomAlert.containerMargins.top
        }
    }

    private func setConstraints() {
        viewContainer.addConstrainedSubviews([scrollView, buttonView]) {
            scrollView.pinEdgesToSuperview(.all().excluding(.bottom))
            buttonView.pinEdgesToSuperview(.all().excluding(.top))
            buttonView.topAnchor.constraint(equalTo: scrollView.bottomAnchor)
        }

        scrollView.addConstrainedSubviews([contentView]) {
            contentView.pinEdgesToSuperview(.all())
            contentView.widthAnchor.constraint(equalTo: scrollView.widthAnchor)
        }

        scrollViewHeightConstraint = scrollView.heightAnchor.constraint(equalToConstant: 0).withPriority(.defaultLow)
        scrollViewHeightConstraint.isActive = true

        view.addConstrainedSubviews([viewContainer]) {
            viewContainer.centerXAnchor.constraint(equalTo: view.centerXAnchor)
            viewContainer.centerYAnchor.constraint(equalTo: view.centerYAnchor)

            viewContainer.topAnchor
                .constraint(greaterThanOrEqualTo: view.layoutMarginsGuide.topAnchor)
                .withPriority(.defaultHigh)

            view.layoutMarginsGuide.bottomAnchor
                .constraint(greaterThanOrEqualTo: viewContainer.bottomAnchor)
                .withPriority(.defaultHigh)

            let leadingConstraint = viewContainer.leadingAnchor
                .constraint(equalTo: view.layoutMarginsGuide.leadingAnchor)
            let trailingConstraint = view.layoutMarginsGuide.trailingAnchor
                .constraint(equalTo: viewContainer.trailingAnchor)

            if traitCollection.userInterfaceIdiom == .pad {
                viewContainer.widthAnchor
                    .constraint(lessThanOrEqualToConstant: UIMetrics.preferredFormSheetContentSize.width)
                leadingConstraint.withPriority(.defaultHigh)
                trailingConstraint.withPriority(.defaultHigh)
            } else {
                leadingConstraint
                trailingConstraint
            }
        }
    }

    private func adjustButtonMargins() {
        if !buttonView.arrangedSubviews.isEmpty {
            // The presence of a button should yield a custom top margin.
            buttonView.directionalLayoutMargins.top = UIMetrics.CustomAlert.interContainerSpacing

            // Buttons below scrollable content should have more margin.
            if scrollView.contentSize.height > scrollView.bounds.size.height {
                buttonView.directionalLayoutMargins.top = UIMetrics.CustomAlert.containerSpacing
            }
        }
    }

    private func addHeader(_ title: String) {
        let label = UILabel()

        label.text = title
        label.font = .preferredFont(forTextStyle: .largeTitle, weight: .bold)
        label.textColor = .white
        label.adjustsFontForContentSizeCategory = true
        label.textAlignment = .center
        label.numberOfLines = 0
        label.setAccessibilityIdentifier(.alertTitle)

        contentView.addArrangedSubview(label)
        contentView.setCustomSpacing(16, after: label)
    }

    private func addTitle(_ title: String) {
        let label = UILabel()

        label.text = title
        label.font = .preferredFont(forTextStyle: .title3, weight: .semibold)
        label.textColor = .white.withAlphaComponent(0.9)
        label.adjustsFontForContentSizeCategory = true
        label.numberOfLines = 0

        contentView.addArrangedSubview(label)
        contentView.setCustomSpacing(8, after: label)
    }

    private func addMessage(_ message: String) {
        let label = UILabel()

        let message = NSMutableAttributedString(string: message)
        message.apply(paragraphStyle: .alert)

        label.attributedText = message
        label.font = .preferredFont(forTextStyle: .body)
        label.textColor = .white.withAlphaComponent(0.8)
        label.adjustsFontForContentSizeCategory = true
        label.numberOfLines = 0

        contentView.addArrangedSubview(label)
    }

    private func addMessage(_ message: NSAttributedString) {
        let label = UILabel()

        let message = NSMutableAttributedString(attributedString: message)
        message.removeAttribute(.paragraphStyle, range: NSRange(location: 0, length: message.length))
        message.apply(paragraphStyle: .alert)

        label.attributedText = message
        label.textColor = .white.withAlphaComponent(0.8)
        label.adjustsFontForContentSizeCategory = true
        label.numberOfLines = 0

        contentView.addArrangedSubview(label)
    }

    private func addIcon(_ icon: AlertIcon) {
        let iconView = icon == .spinner ? getSpinnerView() : getImageView(for: icon)
        contentView.addArrangedSubview(iconView)
    }

    private func addAction(
        title: String,
        style: AlertActionStyle,
        accessibilityId: AccessibilityIdentifier?,
        handler: (() -> Void)? = nil
    ) {
        let button = AppButton(style: style.buttonStyle)

        button.setTitle(title, for: .normal)
        button.setAccessibilityIdentifier(accessibilityId)
        button.addTarget(self, action: #selector(didTapButton), for: .touchUpInside)

        buttonView.addArrangedSubview(button)
        handler.flatMap { handlers[button] = $0 }
    }

    private func getImageView(for icon: AlertIcon) -> UIView {
        let imageView = UIImageView()
        let imageContainerView = UIView()

        imageContainerView.addConstrainedSubviews([imageView]) {
            imageView.pinEdges(.init([.top(0), .bottom(0)]), to: imageContainerView)
            imageView.centerXAnchor.constraint(equalTo: imageContainerView.centerXAnchor, constant: 0)
            imageView.heightAnchor.constraint(equalToConstant: 48)
            imageView.widthAnchor.constraint(equalTo: imageView.heightAnchor, multiplier: 1)
        }

        imageView.image = icon.image?.withRenderingMode(.alwaysOriginal)
        imageView.contentMode = .scaleAspectFit

        return imageContainerView
    }

    private func getSpinnerView() -> UIView {
        let spinnerView = SpinnerActivityIndicatorView(style: .large)
        let spinnerContainerView = UIView()

        spinnerContainerView.addConstrainedSubviews([spinnerView]) {
            spinnerView.pinEdges(.init([.top(0), .bottom(0)]), to: spinnerContainerView)
            spinnerView.centerXAnchor.constraint(equalTo: spinnerContainerView.centerXAnchor, constant: 0)
        }

        spinnerView.startAnimating()

        return spinnerContainerView
    }

    @objc private func didTapButton(_ button: AppButton) {
        onDismiss?()
        if let handler = handlers.removeValue(forKey: button) {
            handler()
        }
        handlers.removeAll()
    }
}
