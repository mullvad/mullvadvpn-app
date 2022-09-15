import React, { useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import { InAppNotificationIndicatorType } from '../../shared/notifications/notification';
import * as AppButton from './AppButton';
import { tinyText } from './common-styles';
import ImageView from './ImageView';

const NOTIFICATION_AREA_ID = 'notification-area';

export const NotificationTitle = styled.span(tinyText, {
  color: colors.white,
});

export const NotificationSubtitleText = styled.span(tinyText, {
  color: colors.white60,
});

interface INotificationSubtitleProps {
  children?: React.ReactNode;
}

export function NotificationSubtitle(props: INotificationSubtitleProps) {
  return React.Children.count(props.children) > 0 ? <NotificationSubtitleText {...props} /> : null;
}

export const NotificationOpenLinkActionButton = styled(AppButton.SimpleButton)({
  flex: 1,
  justifyContent: 'center',
  cursor: 'default',
  padding: '4px',
  background: 'transparent',
  border: 'none',
});

export const NotificationOpenLinkActionIcon = styled(ImageView)({
  [NotificationOpenLinkActionButton + ':hover &']: {
    backgroundColor: colors.white80,
  },
});

interface INotifcationOpenLinkActionProps {
  onClick: () => Promise<void>;
  children?: React.ReactNode;
}

export function NotificationOpenLinkAction(props: INotifcationOpenLinkActionProps) {
  return (
    <AppButton.BlockingButton onClick={props.onClick}>
      <NotificationOpenLinkActionButton
        aria-describedby={NOTIFICATION_AREA_ID}
        aria-label={messages.gettext('Open URL')}>
        <NotificationOpenLinkActionIcon
          height={12}
          width={12}
          tintColor={colors.white60}
          source="icon-extLink"
        />
      </NotificationOpenLinkActionButton>
    </AppButton.BlockingButton>
  );
}

export const NotificationContent = styled.div.attrs({ id: NOTIFICATION_AREA_ID })({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  paddingRight: '4px',
});

export const NotificationActions = styled.div({
  display: 'flex',
  flex: 0,
  flexDirection: 'column',
  justifyContent: 'center',
});

interface INotificationIndicatorProps {
  type?: InAppNotificationIndicatorType;
}

const notificationIndicatorTypeColorMap = {
  success: colors.green,
  warning: colors.yellow,
  error: colors.red,
};

export const NotificationIndicator = styled.div((props: INotificationIndicatorProps) => ({
  width: '10px',
  height: '10px',
  borderRadius: '5px',
  marginTop: '4px',
  marginRight: '8px',
  backgroundColor: props.type ? notificationIndicatorTypeColorMap[props.type] : 'transparent',
}));

interface ICollapsibleProps {
  alignBottom: boolean;
  contentHeight?: number;
  collapsibleHeight?: number;
}

const TRANSITION_DURATION = 350;
// 52px is the height of the banner when the notification contains a title and subtitle which are
// one line each.
const TRANSITION_BASE_DISTANCE = 52;

const Collapsible = styled.div({}, (props: ICollapsibleProps) => {
  // Calculate the transition duration based on travel distance.
  const distance = Math.abs((props.collapsibleHeight ?? 0) - (props.contentHeight ?? 0));
  const duration = Math.ceil(TRANSITION_DURATION * (distance / TRANSITION_BASE_DISTANCE));

  return {
    display: 'flex',
    flexDirection: 'column',
    justifyContent: props.alignBottom ? 'flex-end' : 'flex-start',
    backgroundColor: 'rgba(25, 38, 56, 0.95)',
    overflow: 'hidden',
    // Using auto as the initial value prevents transition if a notification is visible on mount.
    height: props.contentHeight === undefined ? 'auto' : `${props.contentHeight}px`,
    transition: `height ${duration}ms ease-in-out`,
  };
});

const Content = styled.section({
  display: 'flex',
  flexDirection: 'row',
  padding: '8px 12px 8px 16px',
  height: 'fit-content',
});

interface INotificationBannerProps {
  children?: React.ReactNode; // Array<NotificationContent | NotificationActions>,
  className?: string;
  visible: boolean;
}

export function NotificationBanner(props: INotificationBannerProps) {
  const [contentHeight, setContentHeight] = useState<number>();
  const [alignBottom, setAlignBottom] = useState(false);

  const contentRef = useRef() as React.RefObject<HTMLDivElement>;
  const collapsibleRef = useRef() as React.RefObject<HTMLDivElement>;

  // Save last non-undefined children to be able to show them during the hide-transition.
  const prevChildren = useRef<React.ReactNode>();
  useEffect(() => {
    prevChildren.current = props.children ?? prevChildren.current;
  }, [props.children]);

  const onTransitionEnd = useCallback(() => setAlignBottom(false), []);

  useLayoutEffect(() => {
    const newHeight = props.visible ? contentRef.current?.getBoundingClientRect().height ?? 0 : 0;
    if (newHeight !== contentHeight) {
      setContentHeight(newHeight);
      setAlignBottom((alignBottom) => alignBottom || contentHeight === 0 || newHeight === 0);
    }
  });

  return (
    <Collapsible
      ref={collapsibleRef}
      alignBottom={alignBottom}
      contentHeight={contentHeight}
      collapsibleHeight={collapsibleRef.current?.getBoundingClientRect().height ?? 0}
      className={props.className}
      onTransitionEnd={onTransitionEnd}>
      <Content ref={contentRef}>{props.visible ? props.children : prevChildren.current}</Content>
    </Collapsible>
  );
}
