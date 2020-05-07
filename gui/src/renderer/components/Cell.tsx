import * as React from 'react';
import { Button, Component, Styles, Text, TextInput, Types, View } from 'reactxp';
import { colors } from '../../config.json';
import styles, { StyledIcon } from './CellStyles';
import { IImageViewProps } from './ImageView';
import { default as SwitchControl } from './Switch';

export { StyledIcon as UntintedIcon } from './CellStyles';

interface ICellButtonProps {
  children?: React.ReactNode;
  disabled?: boolean;
  selected?: boolean;
  style?: Types.StyleRuleSetRecursive<Types.ButtonStyleRuleSet>;
  hoverStyle?: Types.StyleRuleSetRecursive<Types.ButtonStyleRuleSet>;
  onPress?: () => void;
}

interface IState {
  hovered: boolean;
}

const CellSectionContext = React.createContext<boolean>(false);
const CellHoverContext = React.createContext<boolean>(false);

export class CellButton extends Component<ICellButtonProps, IState> {
  public state: IState = { hovered: false };

  public onHoverStart = () => (!this.props.disabled ? this.setState({ hovered: true }) : null);
  public onHoverEnd = () => (!this.props.disabled ? this.setState({ hovered: false }) : null);

  public render() {
    const { children, style, hoverStyle, ...otherProps } = this.props;

    const stateStyle = this.props.selected
      ? styles.cellButton.selected
      : this.state.hovered
      ? hoverStyle || styles.cellButton.hover
      : undefined;

    return (
      <CellSectionContext.Consumer>
        {(containedInSection) => (
          <Button
            style={[
              styles.cellButton.base,
              containedInSection ? styles.cellButton.section : undefined,
              style,
              stateStyle,
            ]}
            onHoverStart={this.onHoverStart}
            onHoverEnd={this.onHoverEnd}
            {...otherProps}>
            <CellHoverContext.Provider value={this.state.hovered}>
              {children}
            </CellHoverContext.Provider>
          </Button>
        )}
      </CellSectionContext.Consumer>
    );
  }
}

interface ISectionTitleProps {
  children?: React.ReactText;
}

export const SectionTitle = function CellSectionTitle(props: ISectionTitleProps) {
  return <Text style={styles.sectionTitle}>{props.children}</Text>;
};

interface ISectionProps {
  children?: React.ReactNode;
  style?: Types.StyleRuleSetRecursive<Types.ViewStyleRuleSet>;
}

export const Section = class CellSection extends Component<ISectionProps> {
  public render() {
    return (
      <View style={this.props.style}>
        <CellSectionContext.Provider value={true}>
          {this.props.children}
        </CellSectionContext.Provider>
      </View>
    );
  }
};

interface IContainerProps {
  children: React.ReactNode;
}

export const Container = function CellContainer({ children }: IContainerProps) {
  return <View style={styles.cellContainer}>{children}</View>;
};

interface ILabelProps {
  containerStyle?: Types.ViewStyleRuleSet;
  textStyle?: Types.TextStyleRuleSet;
  cellHoverContainerStyle?: Types.ViewStyleRuleSet;
  cellHoverTextStyle?: Types.TextStyleRuleSet;
  onPress?: (event: Types.SyntheticEvent) => void;
  children?: React.ReactNode;
}

export const Label = function CellLabel(props: ILabelProps) {
  const {
    children,
    containerStyle,
    textStyle,
    cellHoverContainerStyle,
    cellHoverTextStyle,
    ...otherProps
  } = props;

  return (
    <CellHoverContext.Consumer>
      {(hovered) => (
        <View
          style={[
            styles.label.container,
            containerStyle,
            hovered ? cellHoverContainerStyle : undefined,
          ]}
          {...otherProps}>
          <Text style={[styles.label.text, textStyle, hovered ? cellHoverTextStyle : undefined]}>
            {children}
          </Text>
        </View>
      )}
    </CellHoverContext.Consumer>
  );
};

export const Switch = React.forwardRef(function CellSwitch(
  props: SwitchControl['props'],
  ref?: React.Ref<SwitchControl>,
) {
  return (
    <View style={styles.switch}>
      <SwitchControl ref={ref} {...props} />
    </View>
  );
});

interface IInputFrameProps {
  children?: React.ReactNode;
  style?: Types.StyleRuleSetRecursive<Types.ViewStyleRuleSet>;
}

export const InputFrame = function CellInputFrame(props: IInputFrameProps) {
  const { style, children } = props;

  return <View style={[styles.input.frame, style]}>{children}</View>;
};

interface IInputProps extends Types.TextInputProps {
  validateValue?: (value: string) => boolean;
  modifyValue?: (value: string) => string;
  submitOnBlur?: boolean;
  onSubmit?: (value: string) => void;
}

interface IInputState {
  value?: string;
  focused: boolean;
}

export class Input extends Component<IInputProps, IInputState> {
  public state = {
    value: this.props.value || '',
    focused: false,
  };

  public componentDidUpdate(prevProps: IInputProps, _prevState: IInputState) {
    if (
      !this.state.focused &&
      prevProps.value !== this.props.value &&
      this.props.value !== this.state.value
    ) {
      this.setState((_state, props) => ({
        value: props.value,
      }));
    }
  }

  public render() {
    const {
      style,
      value: _value,
      onChangeText: _onChangeText,
      onFocus: _onFocus,
      onBlur: _onBlur,
      onSubmitEditing: _onSubmitEditing,
      ...otherProps
    } = this.props;

    const validityStyle =
      this.props.validateValue && this.props.validateValue(this.state.value)
        ? styles.input.validValue
        : styles.input.invalidValue;

    return (
      <TextInput
        placeholderTextColor={colors.white60}
        autoCorrect={false}
        style={[styles.input.text, validityStyle, style]}
        onChangeText={this.onChangeText}
        onFocus={this.onFocus}
        onBlur={this.onBlur}
        onSubmitEditing={this.onSubmitEditing}
        value={this.state.value}
        {...otherProps}
      />
    );
  }

  private onChangeText = (value: string) => {
    this.setState({ value });
    if (this.props.onChangeText) {
      this.props.onChangeText(value);
    }
  };

  private onFocus = (e: Types.FocusEvent) => {
    this.setState({ focused: true });
    if (this.props.onFocus) {
      this.props.onFocus(e);
    }
  };

  private onBlur = (e: Types.FocusEvent) => {
    this.setState({ focused: false });
    if (this.props.onBlur) {
      this.props.onBlur(e);
    }
    if (this.props.submitOnBlur && this.props.onSubmit) {
      this.props.onSubmit(this.state.value);
    }
  };

  private onSubmitEditing = () => {
    if (this.props.onSubmit) {
      this.props.onSubmit(this.state.value);
    }
    if (this.props.onSubmitEditing) {
      this.props.onSubmitEditing();
    }
  };
}

interface IAutoSizingTextInputContainerProps {
  style?: Types.StyleRuleSetRecursive<Types.ViewStyleRuleSet>;
  children: React.ReactElement<Types.TextInputProps>;
}

interface IAutoSizingTextInputContainerState {
  placeholderWidth?: number;
  widthStyle?: Types.TextInputStyleRuleSet;
}

export const AutoSizingTextInputContainer = class CellAutoSizingTextInputContainer extends Component<
  IAutoSizingTextInputContainerProps,
  IAutoSizingTextInputContainerState
> {
  public state: IAutoSizingTextInputContainerState = {};

  public render() {
    const children: React.ReactElement<Types.TextInputProps> = this.props.children;

    return (
      <View style={this.props.style}>
        <View style={styles.autoSizingInputContainer.measuringView} onLayout={this.onLayout}>
          <Text
            style={[
              styles.input.text,

              // TextInputStyle is basically an alias for TextStyle, so it's legit to assume that we
              // can use both of them interchangably.
              children.props.style,

              // this style resets any style properties that could constraint the text width.
              styles.autoSizingInputContainer.measureText,
            ]}
            numberOfLines={1}>
            {children.props.placeholder}
          </Text>
        </View>

        {React.cloneElement(children, {
          ...children.props,
          style: [children.props.style, this.state.widthStyle],
        })}
      </View>
    );
  }

  private onLayout = (layout: Types.ViewOnLayoutEvent) => {
    if (this.state.placeholderWidth !== layout.width) {
      this.setState({
        placeholderWidth: layout.width,
        widthStyle: Styles.createTextInputStyle(
          {
            width: layout.width,
          },
          false,
        ),
      });
    }
  };
};

type SubTextProps = Types.TextProps & {
  cellHoverStyle?: Types.ViewStyle;
};

export const SubText = function CellSubText(props: SubTextProps) {
  const { children, ref: _, style, cellHoverStyle, ...otherProps } = props;

  return (
    <CellHoverContext.Consumer>
      {(hovered) => (
        <Text style={[styles.subtext, style, hovered ? cellHoverStyle : undefined]} {...otherProps}>
          {children}
        </Text>
      )}
    </CellHoverContext.Consumer>
  );
};

export const Icon = function CellIcon(props: IImageViewProps) {
  const { tintColor, tintHoverColor, ...otherProps } = props;

  return (
    <CellHoverContext.Consumer>
      {(hovered) => (
        <StyledIcon
          tintColor={(hovered && tintHoverColor) || tintColor || colors.white60}
          {...otherProps}
        />
      )}
    </CellHoverContext.Consumer>
  );
};

export const Footer = function CellFooter({ children }: IContainerProps) {
  return <View style={styles.footer.container}>{children}</View>;
};

export const FooterText = function CellFooterText(props: Text['props']) {
  return (
    <Text {...props} style={[styles.footer.text, props.style]}>
      {props.children}
    </Text>
  );
};

export const FooterBoldText = function CellFooterText(props: Text['props']) {
  return (
    <Text {...props} style={[styles.footer.text, styles.footer.boldText, props.style]}>
      {props.children}
    </Text>
  );
};
