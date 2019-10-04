import * as React from 'react';
import { Button, Component, Styles, Text, TextInput, Types, View } from 'reactxp';
import { colors } from '../../config.json';
import ImageView from './ImageView';
import { default as SwitchControl } from './Switch';

const styles = {
  cellButton: {
    base: Styles.createButtonStyle({
      backgroundColor: colors.blue,
      paddingVertical: 0,
      paddingHorizontal: 16,
      marginBottom: 1,
      flex: 1,
      flexDirection: 'row',
      alignItems: 'center',
      alignContent: 'center',
      cursor: 'default',
    }),
    section: Styles.createButtonStyle({
      backgroundColor: colors.blue40,
    }),
    hover: Styles.createButtonStyle({
      backgroundColor: colors.blue80,
    }),
    selected: Styles.createViewStyle({
      backgroundColor: colors.green,
    }),
  },
  cellContainer: Styles.createViewStyle({
    backgroundColor: colors.blue,
    flexDirection: 'row',
    alignItems: 'center',
    paddingLeft: 16,
    paddingRight: 12,
  }),
  footer: {
    container: Styles.createViewStyle({
      paddingTop: 8,
      paddingRight: 24,
      paddingBottom: 24,
      paddingLeft: 24,
    }),
    text: Styles.createTextStyle({
      fontFamily: 'Open Sans',
      fontSize: 13,
      fontWeight: '600',
      lineHeight: 20,
      letterSpacing: -0.2,
      color: colors.white80,
    }),
    boldText: Styles.createTextStyle({
      fontWeight: '900',
    }),
  },
  label: {
    container: Styles.createViewStyle({
      marginLeft: 8,
      marginTop: 14,
      marginBottom: 14,
      flex: 1,
    }),
    text: Styles.createTextStyle({
      fontFamily: 'DINPro',
      fontSize: 20,
      fontWeight: '900',
      lineHeight: 26,
      letterSpacing: -0.2,
      color: colors.white,
    }),
  },
  switch: Styles.createViewStyle({
    flex: 0,
  }),
  input: {
    frame: Styles.createViewStyle({
      flexGrow: 0,
      backgroundColor: 'rgba(255,255,255,0.1)',
      borderRadius: 4,
      padding: 4,
    }),
    text: Styles.createTextInputStyle({
      color: colors.white,
      backgroundColor: 'transparent',
      fontFamily: 'Open Sans',
      fontSize: 20,
      fontWeight: '600',
      lineHeight: 26,
      textAlign: 'right',
      padding: 0,
    }),
  },
  autoSizingInputContainer: {
    measuringView: Styles.createViewStyle({
      position: 'absolute',
      opacity: 0,
    }),
    measureText: Styles.createTextStyle({
      width: undefined,
      flexBasis: undefined,
    }),
  },
  icon: Styles.createViewStyle({
    marginLeft: 8,
  }),
  subtext: Styles.createTextStyle({
    color: colors.white60,
    fontFamily: 'Open Sans',
    fontSize: 13,
    fontWeight: '800',
    flex: -1,
    textAlign: 'right',
    marginLeft: 8,
  }),
  sectionTitle: Styles.createTextStyle({
    backgroundColor: colors.blue,
    paddingVertical: 14,
    paddingHorizontal: 24,
    marginBottom: 1,
    fontFamily: 'DINPro',
    fontSize: 20,
    fontWeight: '900',
    lineHeight: 26,
    color: colors.white,
  }),
};

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

export const Input = React.forwardRef(function CellInput(
  props: Types.TextInputProps,
  ref?: React.Ref<TextInput>,
) {
  const { style, ...otherProps } = props;

  return (
    <TextInput
      ref={ref}
      placeholderTextColor={colors.white60}
      autoCorrect={false}
      style={[styles.input.text, style]}
      {...otherProps}
    />
  );
});

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

export const Icon = function CellIcon(props: ImageView['props']) {
  const { children: _children, style, tintColor, tintHoverColor, ...otherProps } = props;

  return (
    <CellHoverContext.Consumer>
      {(hovered) => (
        <ImageView
          tintColor={(hovered && tintHoverColor) || tintColor || colors.white60}
          style={[styles.icon, style]}
          {...otherProps}
        />
      )}
    </CellHoverContext.Consumer>
  );
};

export const UntintedIcon = function CellIcon(props: ImageView['props']) {
  return <ImageView {...props} style={[styles.icon, props.style]} />;
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
