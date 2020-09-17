import React, { useCallback, useContext, useLayoutEffect, useRef, useState } from 'react';
import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import CustomScrollbars, { IScrollEvent } from './CustomScrollbars';
import {
  StyledBackBarItemButton,
  StyledBackBarItemIcon,
  StyledBackBarItemLabel,
  StyledCloseBarItemButton,
  StyledCloseBarItemIcon,
  StyledNavigationBar,
  StyledNavigationBarSeparator,
  StyledNavigationBarWrapper,
  StyledTitleBarItemContainer,
  StyledTitleBarItemLabel,
  StyledTitleBarItemMeasuringLabel,
} from './NavigationBarStyles';

export { StyledNavigationItems as NavigationItems } from './NavigationBarStyles';

interface INavigationContainerProps {
  children?: React.ReactNode;
}

interface INavigationContainerState {
  showsBarTitle: boolean;
  showsBarSeparator: boolean;
}

const NavigationScrollContext = React.createContext({
  showsBarTitle: false,
  showsBarSeparator: false,
  onScroll(_event: IScrollEvent): void {
    throw Error('NavigationScrollContext provider missing');
  },
});

export class NavigationContainer extends React.Component<
  INavigationContainerProps,
  INavigationContainerState
> {
  public state = {
    showsBarTitle: false,
    showsBarSeparator: false,
  };

  private scrollEventListeners: Array<(event: IScrollEvent) => void> = [];

  public componentDidMount() {
    this.updateBarAppearance({ scrollLeft: 0, scrollTop: 0 });
  }

  public render() {
    return (
      <NavigationScrollContext.Provider
        value={{
          ...this.state,
          onScroll: this.onScroll,
        }}>
        {this.props.children}
      </NavigationScrollContext.Provider>
    );
  }

  public onScroll = (event: IScrollEvent) => {
    this.notifyScrollEventListeners(event);
    this.updateBarAppearance(event);
  };

  public addScrollEventListener(fn: (event: IScrollEvent) => void) {
    const index = this.scrollEventListeners.indexOf(fn);
    if (index === -1) {
      this.scrollEventListeners.push(fn);
    }
  }

  public removeScrollEventListener(fn: (event: IScrollEvent) => void) {
    const index = this.scrollEventListeners.indexOf(fn);
    if (index !== -1) {
      this.scrollEventListeners.splice(index, 1);
    }
  }

  private notifyScrollEventListeners(event: IScrollEvent) {
    this.scrollEventListeners.forEach((listener) => listener(event));
  }

  private updateBarAppearance(event: IScrollEvent) {
    // that's where SettingsHeader.HeaderTitle intersects the navigation bar
    const showsBarSeparator = event.scrollTop > 11;

    // that's when SettingsHeader.HeaderTitle goes behind the navigation bar
    const showsBarTitle = event.scrollTop > 20;

    if (
      this.state.showsBarSeparator !== showsBarSeparator ||
      this.state.showsBarTitle !== showsBarTitle
    ) {
      this.setState({ showsBarSeparator, showsBarTitle });
    }
  }
}

interface INavigationScrollbarsProps {
  onScroll?: (value: IScrollEvent) => void;
  className?: string;
  fillContainer?: boolean;
  children?: React.ReactNode;
}

export const NavigationScrollbars = React.forwardRef(function NavigationScrollbarsT(
  props: INavigationScrollbarsProps,
  ref?: React.Ref<CustomScrollbars>,
) {
  const { onScroll } = useContext(NavigationScrollContext);

  const handleScroll = useCallback((event: IScrollEvent) => {
    onScroll(event);
    props.onScroll?.(event);
  }, []);

  return (
    <CustomScrollbars
      ref={ref}
      className={props.className}
      fillContainer={props.fillContainer}
      onScroll={handleScroll}>
      {props.children}
    </CustomScrollbars>
  );
});

const TitleBarItemContext = React.createContext({
  titleAdjustment: 0,
  visible: false,
  get titleContainerRef(): React.RefObject<HTMLDivElement> {
    throw Error('Missing TitleBarItemContext provider');
  },
  get measuringTitleRef(): React.RefObject<HTMLHeadingElement> {
    throw Error('Missing TitleBarItemContext provider');
  },
});

interface INavigationBarProps {
  children?: React.ReactNode;
  alwaysDisplayBarTitle?: boolean;
}

export const NavigationBar = function NavigationBarT(props: INavigationBarProps) {
  const { showsBarSeparator, showsBarTitle } = useContext(NavigationScrollContext);
  const [titleAdjustment, setTitleAdjustment] = useState(0);

  const titleContainerRef = useRef() as React.RefObject<HTMLDivElement>;
  const measuringTitleRef = useRef() as React.RefObject<HTMLHeadingElement>;
  const navigationBarRef = useRef() as React.RefObject<HTMLDivElement>;

  useLayoutEffect(() => {
    const titleContainerRect = titleContainerRef.current?.getBoundingClientRect();
    const measuringTitleRect = measuringTitleRef.current?.getBoundingClientRect();
    const navigationBarRect = navigationBarRef.current?.getBoundingClientRect();

    if (titleContainerRect && measuringTitleRect && navigationBarRect) {
      // calculate the width of the elements preceding the title view container
      const leadingSpace = titleContainerRect.x - navigationBarRect.x;

      // calculate the width of the elements succeeding the title view container
      const trailingSpace = navigationBarRect.width - titleContainerRect.width - leadingSpace;

      // calculate the adjustment needed to center the title view within navigation bar
      const titleAdjustment = Math.floor(trailingSpace - leadingSpace);

      // calculate the maximum possible adjustment that when applied should keep the text fully
      // visible, unless the title container itself is smaller than the space needed to accommodate
      // the text
      const maxTitleAdjustment = Math.floor(
        Math.max(titleContainerRect.width - measuringTitleRect.width, 0),
      );

      // cap the adjustment to remain within the allowed bounds
      const cappedTitleAdjustment = Math.min(
        Math.max(-maxTitleAdjustment, titleAdjustment),
        maxTitleAdjustment,
      );

      setTitleAdjustment(cappedTitleAdjustment);
    }
  });

  return (
    <StyledNavigationBar>
      <StyledNavigationBarWrapper ref={navigationBarRef}>
        <TitleBarItemContext.Provider
          value={{
            titleAdjustment: titleAdjustment,
            visible: props.alwaysDisplayBarTitle || showsBarTitle,
            titleContainerRef,
            measuringTitleRef,
          }}>
          {props.children}
        </TitleBarItemContext.Provider>
      </StyledNavigationBarWrapper>
      {showsBarSeparator && <StyledNavigationBarSeparator />}
    </StyledNavigationBar>
  );
};

interface ITitleBarItemProps {
  children?: React.ReactText;
}

export const TitleBarItem = React.memo(function TitleBarItemT(props: ITitleBarItemProps) {
  const { measuringTitleRef, titleAdjustment, titleContainerRef, visible } = useContext(
    TitleBarItemContext,
  );

  return (
    <StyledTitleBarItemContainer ref={titleContainerRef}>
      <StyledTitleBarItemLabel titleAdjustment={titleAdjustment} visible={visible}>
        {props.children}
      </StyledTitleBarItemLabel>

      <StyledTitleBarItemMeasuringLabel
        titleAdjustment={0}
        ref={measuringTitleRef}
        aria-hidden={true}>
        {props.children}
      </StyledTitleBarItemMeasuringLabel>
    </StyledTitleBarItemContainer>
  );
});

interface ICloseBarItemProps {
  action: () => void;
}

export function CloseBarItem(props: ICloseBarItemProps) {
  // Use the arrow down icon on Linux, to avoid confusion with the close button in the window
  // title bar.
  const iconName = process.platform === 'linux' ? 'icon-close-down' : 'icon-close';
  return (
    <StyledCloseBarItemButton aria-label={messages.gettext('Close')} onClick={props.action}>
      <StyledCloseBarItemIcon
        height={24}
        width={24}
        source={iconName}
        tintColor={colors.white60}
        tintHoverColor={colors.white80}
      />
    </StyledCloseBarItemButton>
  );
}

interface IBackBarItemProps {
  children?: React.ReactText;
  action: () => void;
}

export function BackBarItem(props: IBackBarItemProps) {
  return (
    <StyledBackBarItemButton onClick={props.action}>
      <StyledBackBarItemIcon source="icon-back" tintColor={colors.white60} />
      <StyledBackBarItemLabel>{props.children}</StyledBackBarItemLabel>
    </StyledBackBarItemButton>
  );
}
