import * as React from 'react';
import { Button, Component, Styles, Text, TextInput, Types, View } from 'reactxp';
import { colors } from '../../config.json';
import ImageView from './ImageView';

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
  input: {
    frame: Styles.createViewStyle({
      flexGrow: 0,
      backgroundColor: 'rgba(255,255,255,0.1)',
      borderRadius: 4,
      paddingHorizontal: 2,
      paddingVertical: 2,
    }),
    text: Styles.createTextStyle({
      color: colors.white,
      backgroundColor: 'transparent',
      fontFamily: 'Open Sans',
      fontSize: 20,
      fontWeight: '600',
      lineHeight: 26,
      textAlign: 'right',
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
  cellHoverStyle?: Types.StyleRuleSetRecursive<Types.ButtonStyleRuleSet>;
  style?: Types.StyleRuleSetRecursive<Types.ButtonStyleRuleSet>;
  onPress?: () => void;
}

interface IState {
  hovered: boolean;
}

const CellSectionContext = React.createContext<boolean>(false);
const CellHoverContext = React.createContext<boolean>(false);

export class CellButton extends Component<ICellButtonProps, IState> {
  public state = { hovered: false };

  public onHoverStart = () => (!this.props.disabled ? this.setState({ hovered: true }) : null);
  public onHoverEnd = () => (!this.props.disabled ? this.setState({ hovered: false }) : null);

  public render() {
    const { children, style, cellHoverStyle, ...otherProps } = this.props;
    const hoverStyle = cellHoverStyle || styles.cellButton.hover;
    return (
      <CellSectionContext.Consumer>
        {(containedInSection) => (
          <Button
            style={[
              styles.cellButton.base,
              containedInSection ? styles.cellButton.section : undefined,
              style,
              this.state.hovered ? hoverStyle : undefined,
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

export function SectionTitle(props: ISectionTitleProps) {
  return <Text style={styles.sectionTitle}>{props.children}</Text>;
}

interface ISectionProps {
  children?: React.ReactNode;
}

export class Section extends Component<ISectionProps> {
  public render() {
    return (
      <View>
        <CellSectionContext.Provider value={true}>
          {this.props.children}
        </CellSectionContext.Provider>
      </View>
    );
  }
}

interface IContainerProps {
  children: React.ReactNode;
}

export function Container({ children }: IContainerProps) {
  return <View style={styles.cellContainer}>{children}</View>;
}

interface ILabelProps {
  containerStyle?: Types.ViewStyleRuleSet;
  textStyle?: Types.TextStyleRuleSet;
  cellHoverContainerStyle?: Types.ViewStyleRuleSet;
  cellHoverTextStyle?: Types.TextStyleRuleSet;
  onPress?: (event: Types.SyntheticEvent) => void;
  children?: React.ReactNode;
}

export function Label(props: ILabelProps) {
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
}

interface InputFrameProps {
  children?: React.ReactNode;
  style?: Types.StyleRuleSetRecursive<Types.ViewStyleRuleSet>;
}

export function InputFrame(props: InputFrameProps) {
  const { style, children } = props;

  return <View style={[styles.input.frame, style]}>{children}</View>;
}

export const Input = React.forwardRef(function InputT(
  props: Types.TextInputProps,
  ref?: React.Ref<TextInput>,
) {
  const { style, ...otherProps } = props;

  return (
    <TextInput
      ref={ref as any}
      maxLength={10}
      placeholderTextColor={colors.white60}
      autoCorrect={false}
      autoFocus={false}
      style={[styles.input.text, style]}
      {...otherProps}
    />
  );
});

type SubTextProps = Types.TextProps & {
  cellHoverStyle?: Types.ViewStyle;
};

export function SubText(props: SubTextProps) {
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
}

export function Icon(props: ImageView['props']) {
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
}

export function Footer({ children }: IContainerProps) {
  return (
    <View style={styles.footer.container}>
      <Text style={styles.footer.text}>{children}</Text>
    </View>
  );
}
