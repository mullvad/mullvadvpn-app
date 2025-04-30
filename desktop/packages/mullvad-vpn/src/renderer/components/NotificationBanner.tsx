import React, { useEffect, useState } from 'react';
import styled from 'styled-components';

import { messages } from '../../shared/gettext';
import { InAppNotificationIndicatorType } from '../../shared/notifications/notification';
import { IconButton } from '../lib/components';
import { Colors } from '../lib/foundations';
import { useExclusiveTask } from '../lib/hooks/use-exclusive-task';
import { useEffectEvent, useLastDefinedValue, useStyledRef } from '../lib/utility-hooks';
import { tinyText } from './common-styles';

const NOTIFICATION_AREA_ID = 'notification-area';

export const NotificationTitle = styled.span(tinyText, {
  color: Colors.white,
});

export const NotificationSubtitleText = styled.span(tinyText, {
  color: Colors.white60,
});

interface INotificationSubtitleProps {
  children?: React.ReactNode;
}

export function NotificationSubtitle(props: INotificationSubtitleProps) {
  return React.Children.count(props.children) > 0 ? <NotificationSubtitleText {...props} /> : null;
}

interface NotificationActionProps {
  onClick: () => Promise<void>;
}

export function NotificationOpenLinkAction(props: NotificationActionProps) {
  const [onClick] = useExclusiveTask(props.onClick);
  return (
    <IconButton
      size="small"
      variant="secondary"
      onClick={onClick}
      aria-describedby={NOTIFICATION_AREA_ID}
      aria-label={messages.gettext('Open URL')}>
      <IconButton.Icon icon="external" />
    </IconButton>
  );
}

export function NotificationTroubleshootDialogAction(props: NotificationActionProps) {
  return (
    <IconButton
      size="small"
      variant="secondary"
      aria-describedby={NOTIFICATION_AREA_ID}
      aria-label={messages.gettext('Troubleshoot')}
      onClick={props.onClick}>
      <IconButton.Icon icon="info-circle" />
    </IconButton>
  );
}

export function NotificationCloseAction(props: NotificationActionProps) {
  return (
    <IconButton
      aria-describedby={NOTIFICATION_AREA_ID}
      variant="secondary"
      aria-label={messages.pgettext('accessibility', 'Close notification')}
      onClick={props.onClick}
      size="small">
      <IconButton.Icon icon="cross-circle" />
    </IconButton>
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
  $type?: InAppNotificationIndicatorType;
}

const notificationIndicatorTypeColorMap = {
  success: Colors.green,
  warning: Colors.yellow,
  error: Colors.red,
};

export const NotificationIndicator = styled.div<INotificationIndicatorProps>((props) => ({
  width: '10px',
  height: '10px',
  borderRadius: '5px',
  marginTop: '4px',
  marginRight: '8px',
  backgroundColor: props.$type ? notificationIndicatorTypeColorMap[props.$type] : 'transparent',
}));

interface ICollapsibleProps {
  $alignBottom: boolean;
  $height?: number;
}

const Collapsible = styled.div<ICollapsibleProps>((props) => {
  return {
    display: 'flex',
    flexDirection: 'column',
    justifyContent: props.$alignBottom ? 'flex-end' : 'flex-start',
    backgroundColor: Colors.darkerBlue,
    overflow: 'hidden',
    // Using auto as the initial value prevents transition if a notification is visible on mount.
    height: props.$height === undefined ? 'auto' : `${props.$height}px`,
    transition: 'height 250ms ease-in-out',
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
}

export function NotificationBanner(props: INotificationBannerProps) {
  const [contentHeight, setContentHeight] = useState<number>();
  const [alignBottom, setAlignBottom] = useState(false);

  const contentRef = useStyledRef<HTMLDivElement>();

  const children = useLastDefinedValue(props.children);

  const updateHeightEvent = useEffectEvent(() => {
    const newHeight =
      props.children !== undefined ? (contentRef.current?.getBoundingClientRect().height ?? 0) : 0;
    if (newHeight !== contentHeight) {
      setContentHeight(newHeight);
      setAlignBottom((alignBottom) => alignBottom || contentHeight === 0 || newHeight === 0);
    }
  });

  useEffect(() => updateHeightEvent());

  return (
    <Collapsible $height={contentHeight} className={props.className} $alignBottom={alignBottom}>
      <Content ref={contentRef}>{children}</Content>
    </Collapsible>
  );
}
