//
//  AlertViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-05-19.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
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
    case info
    case spinner

    fileprivate var image: UIImage? {
        switch self {
        case .alert:
            return UIImage(named: "IconAlert")?.withTintColor(.dangerColor)
        case .info:
            return UIImage(named: "IconInfo")?.withTintColor(.white)
        default:
            return nil
        }
    }
}

class AlertViewController: UIViewController {
    typealias Handler = () -> Void
    var onDismiss: Handler?

    private let containerView: UIStackView = {
        let view = UIStackView()

        view.axis = .vertical
        view.backgroundColor = .secondaryColor
        view.layer.cornerRadius = 11
        view.spacing = UIMetrics.CustomAlert.containerSpacing
        view.isLayoutMarginsRelativeArrangement = true
        view.directionalLayoutMargins = UIMetrics.CustomAlert.containerMargins

        return view
    }()

    private var handlers = [UIButton: Handler]()

    init(presentation: AlertPresentation) {
        super.init(nibName: nil, bundle: nil)

        setUp(
            header: presentation.header,
            title: presentation.title,
            icon: presentation.icon
        ) {
            if let message = presentation.attributedMessage {
                addMessage(message)
            } else if let message = presentation.message {
                addMessage(message)
            }
        }

        presentation.buttons.forEach { action in
            addAction(
                title: action.title,
                style: action.style,
                handler: action.handler
            )
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // This code runs before viewDidLoad(). As such, no implicit calls to self.view should be made before this point.
    private func setUp(header: String?, title: String?, icon: AlertIcon?, addMessageCallback: () -> Void) {
        modalPresentationStyle = .overFullScreen
        modalTransitionStyle = .crossDissolve

        icon.flatMap { addIcon($0) }
        header.flatMap { addHeader($0) }
        title.flatMap { addTitle($0) }
        addMessageCallback()

        containerView.arrangedSubviews.last.flatMap {
            containerView.setCustomSpacing(UIMetrics.CustomAlert.containerMargins.top, after: $0)
        }

        // Icon only alerts should have equal top and bottom margin.
        if icon != nil, containerView.arrangedSubviews.count == 1 {
            containerView.directionalLayoutMargins.bottom = UIMetrics.CustomAlert.containerMargins.top
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .black.withAlphaComponent(0.5)

        view.addConstrainedSubviews([containerView]) {
            containerView.centerXAnchor.constraint(equalTo: view.centerXAnchor)
            containerView.centerYAnchor.constraint(equalTo: view.centerYAnchor)

            containerView.widthAnchor
                .constraint(lessThanOrEqualToConstant: UIMetrics.preferredFormSheetContentSize.width)

            containerView.leadingAnchor
                .constraint(equalTo: view.layoutMarginsGuide.leadingAnchor)
                .withPriority(.defaultHigh)

            view.layoutMarginsGuide.trailingAnchor
                .constraint(equalTo: containerView.trailingAnchor)
                .withPriority(.defaultHigh)
        }
    }

    func addAction(title: String, style: AlertActionStyle, handler: (() -> Void)? = nil) {
        // The presence of a button should reset any custom button margin to default.
        containerView.directionalLayoutMargins.bottom = UIMetrics.CustomAlert.containerMargins.bottom

        let button = AppButton(style: style.buttonStyle)

        button.setTitle(title, for: .normal)
        button.addTarget(self, action: #selector(didTapButton), for: .touchUpInside)

        containerView.addArrangedSubview(button)
        handler.flatMap { handlers[button] = $0 }
    }

    private func addHeader(_ title: String) {
        let header = UILabel()

        header.text = title
        header.font = .preferredFont(forTextStyle: .largeTitle, weight: .bold)
        header.textColor = .white
        header.adjustsFontForContentSizeCategory = true
        header.textAlignment = .center
        header.numberOfLines = 0

        containerView.addArrangedSubview(header)
        containerView.setCustomSpacing(16, after: header)
    }

    private func addTitle(_ title: String) {
        let label = UILabel()

        label.text = title
        label.font = .preferredFont(forTextStyle: .title3, weight: .semibold)
        label.textColor = .white.withAlphaComponent(0.9)
        label.adjustsFontForContentSizeCategory = true
        label.numberOfLines = 0

        containerView.addArrangedSubview(label)
        containerView.setCustomSpacing(8, after: label)
    }

    private func addMessage(_ message: String) {
        let label = UILabel()

        let font = UIFont.preferredFont(forTextStyle: .body)
        let style = NSMutableParagraphStyle()
        style.paragraphSpacing = 16
        style.lineBreakMode = .byWordWrapping

        label.attributedText = NSAttributedString(
            markdownString: message,
            options: MarkdownStylingOptions(font: font, paragraphStyle: style)
        )
        label.font = font
        label.textColor = .white.withAlphaComponent(0.8)
        label.adjustsFontForContentSizeCategory = true
        label.numberOfLines = 0

        containerView.addArrangedSubview(label)
    }

    private func addMessage(_ message: NSAttributedString) {
        let label = UILabel()

        label.attributedText = message
        label.textColor = .white.withAlphaComponent(0.8)
        label.adjustsFontForContentSizeCategory = true
        label.numberOfLines = 0

        containerView.addArrangedSubview(label)
    }

    private func addIcon(_ icon: AlertIcon) {
        let iconView = icon == .spinner ? getSpinnerView() : getImageView(for: icon)
        containerView.addArrangedSubview(iconView)
    }

    private func getImageView(for icon: AlertIcon) -> UIView {
        let imageView = UIImageView()
        let imageContainerView = UIView()

        imageContainerView.addConstrainedSubviews([imageView]) {
            imageView.pinEdges(.init([.top(0), .bottom(0)]), to: imageContainerView)
            imageView.centerXAnchor.constraint(equalTo: imageContainerView.centerXAnchor, constant: 0)
            imageView.heightAnchor.constraint(equalToConstant: 44)
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
        if let handler = handlers.removeValue(forKey: button) {
            handler()
        }

        handlers.removeAll()
        onDismiss?()
    }
}
