import React, { useCallback, useContext, useEffect, useLayoutEffect, useRef } from 'react';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { transitions, useHistory } from '../lib/history';
import { useCombinedRefs, useEffectEvent, useInitialValue } from '../lib/utility-hooks';
import CustomScrollbars, { CustomScrollbarsRef, IScrollEvent } from './CustomScrollbars';
import InfoButton from './InfoButton';
import { BackActionContext } from './KeyboardNavigation';
import {
  StyledBackBarItemButton,
  StyledBackBarItemIcon,
  StyledNavigationBar,
  StyledNavigationBarSeparator,
  StyledNavigationItems,
  StyledTitleBarItemLabel,
} from './NavigationBarStyles';

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
    this.updateBarAppearance(event);
  };

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
  className?: string;
  fillContainer?: boolean;
  children?: React.ReactNode;
}

export const NavigationScrollbars = React.forwardRef(function NavigationScrollbarsT(
  props: INavigationScrollbarsProps,
  forwardedRef?: React.Ref<CustomScrollbarsRef>,
) {
  const history = useHistory();
  const { setNavigationHistory } = useAppContext();
  const { onScroll } = useContext(NavigationScrollContext);

  const ref = useRef<CustomScrollbarsRef>();
  const combinedRefs = useCombinedRefs(forwardedRef, ref);

  const beforeunload = useEffectEvent(() => {
    if (ref.current) {
      history.recordScrollPosition(ref.current.getScrollPosition());
      setNavigationHistory(history.asObject);
    }
  });

  useEffect(() => {
    window.addEventListener('beforeunload', beforeunload);
    return () => window.removeEventListener('beforeunload', beforeunload);
  }, []);

  const onMount = useEffectEvent(() => {
    const location = history.location;
    if (history.action === 'POP') {
      ref.current?.scrollTo(...location.state.scrollPosition);
    }
  });

  const onUnmount = useEffectEvent(() => {
    if (history.action === 'PUSH' && ref.current) {
      history.recordScrollPosition(ref.current.getScrollPosition());
      setNavigationHistory(history.asObject);
    }
  });

  useLayoutEffect(() => {
    onMount();
    return () => onUnmount();
  }, []);

  const handleScroll = useCallback(
    (event: IScrollEvent) => {
      onScroll(event);
    },
    [onScroll],
  );

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

  return (
    <StyledNavigationBar>
      <TitleBarItemContext.Provider
        value={{ visible: props.alwaysDisplayBarTitle || showsBarTitle }}>
        {props.children}
      </TitleBarItemContext.Provider>
      {showsBarSeparator && <StyledNavigationBarSeparator />}
    </StyledNavigationBar>
  );
};

interface INavigationItemsProps {
  children: React.ReactNode;
}

export function NavigationItems(props: INavigationItemsProps) {
  const { parentBackAction } = useContext(BackActionContext);
  return (
    <StyledNavigationItems>
      {parentBackAction && <BackBarItem />}
      {props.children}
    </StyledNavigationItems>
  );
}

interface ITitleBarItemProps {
  children?: React.ReactText;
}

export const TitleBarItem = React.memo(function TitleBarItemT(props: ITitleBarItemProps) {
  const { visible } = useContext(TitleBarItemContext);
  return <StyledTitleBarItemLabel $visible={visible}>{props.children}</StyledTitleBarItemLabel>;
});

export function BackBarItem() {
  const history = useHistory();
  // Compare the transition name with dismiss to infer wheter or not the view will slide
  // horizontally or vertically and then use matching button.
  const backIcon = useInitialValue(
    () => history.getPopTransition().name !== transitions.dismiss.name,
  );
  const { parentBackAction } = useContext(BackActionContext);
  const iconSource = backIcon ? 'icon-back' : 'icon-close-down';
  const ariaLabel = backIcon ? messages.gettext('Back') : messages.gettext('Close');

  return (
    <StyledBackBarItemButton aria-label={ariaLabel} onClick={parentBackAction}>
      <StyledBackBarItemIcon source={iconSource} tintColor={colors.white40} width={24} />
    </StyledBackBarItemButton>
  );
}

const navigationRightHandSideButton: React.CSSProperties = {
  justifySelf: 'end',
  borderWidth: 0,
  padding: 0,
  margin: 0,
  cursor: 'default',
  backgroundColor: 'transparent',
};

export const NavigationBarButton = styled.button({ ...navigationRightHandSideButton });
export const NavigationInfoButton = styled(InfoButton).attrs({
  size: 24,
  tintColor: colors.white40,
  tintHoverColor: colors.white60,
})({
  ...navigationRightHandSideButton,
});
