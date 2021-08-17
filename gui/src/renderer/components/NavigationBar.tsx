import React, { useCallback, useContext, useLayoutEffect, useRef } from 'react';
import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import useActions from '../lib/actionsHook';
import { useHistory } from '../lib/history';
import { useCombinedRefs } from '../lib/utilityHooks';
import { useSelector } from '../redux/store';
import userInterface from '../redux/userinterface/actions';
import CustomScrollbars, { IScrollEvent } from './CustomScrollbars';
import {
  StyledBackBarItemButton,
  StyledBackBarItemIcon,
  StyledBackBarItemLabel,
  StyledCloseBarItemButton,
  StyledCloseBarItemIcon,
  StyledNavigationBar,
  StyledNavigationBarSeparator,
  StyledTitleBarItemLabel,
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
  forwardedRef?: React.Ref<CustomScrollbars>,
) {
  const history = useHistory();
  const { onScroll } = useContext(NavigationScrollContext);

  const ref = useRef<CustomScrollbars>();
  const combinedRefs = useCombinedRefs(forwardedRef, ref);

  const { addScrollPosition, removeScrollPosition } = useActions(userInterface);
  const scrollPosition = useSelector(
    (state) => state.userInterface.scrollPosition[history.location.pathname],
  );

  useLayoutEffect(() => {
    const path = history.location.pathname;

    if (history.action === 'POP' && scrollPosition) {
      ref.current?.scrollTo(...scrollPosition);
      removeScrollPosition(path);
    }

    return () => {
      if (history.action === 'PUSH' && ref.current) {
        addScrollPosition(path, ref.current.getScrollPosition());
      }
    };
  }, []);

  const handleScroll = useCallback((event: IScrollEvent) => {
    onScroll(event);
    props.onScroll?.(event);
  }, []);

  return (
    <CustomScrollbars
      ref={combinedRefs}
      className={props.className}
      fillContainer={props.fillContainer}
      onScroll={handleScroll}>
      {props.children}
    </CustomScrollbars>
  );
});

const TitleBarItemContext = React.createContext({
  visible: false,
});

interface INavigationBarProps {
  children?: React.ReactNode;
  alwaysDisplayBarTitle?: boolean;
}

export const NavigationBar = function NavigationBarT(props: INavigationBarProps) {
  const { showsBarSeparator, showsBarTitle } = useContext(NavigationScrollContext);
  const unpinnedWindow = useSelector((state) => state.settings.guiSettings.unpinnedWindow);

  return (
    <StyledNavigationBar unpinnedWindow={unpinnedWindow}>
      <TitleBarItemContext.Provider
        value={{ visible: props.alwaysDisplayBarTitle || showsBarTitle }}>
        {props.children}
      </TitleBarItemContext.Provider>
      {showsBarSeparator && <StyledNavigationBarSeparator />}
    </StyledNavigationBar>
  );
};

interface ITitleBarItemProps {
  children?: React.ReactText;
}

export const TitleBarItem = React.memo(function TitleBarItemT(props: ITitleBarItemProps) {
  const { visible } = useContext(TitleBarItemContext);
  return <StyledTitleBarItemLabel visible={visible}>{props.children}</StyledTitleBarItemLabel>;
});

interface ICloseBarItemProps {
  action: () => void;
}

export function CloseBarItem(props: ICloseBarItemProps) {
  return (
    <StyledCloseBarItemButton aria-label={messages.gettext('Close')} onClick={props.action}>
      <StyledCloseBarItemIcon
        height={24}
        width={24}
        source={'icon-close-down'}
        tintColor={colors.white40}
        tintHoverColor={colors.white60}
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
      <StyledBackBarItemIcon source="icon-back" tintColor={colors.white40} />
      <StyledBackBarItemLabel>{props.children}</StyledBackBarItemLabel>
    </StyledBackBarItemButton>
  );
}
