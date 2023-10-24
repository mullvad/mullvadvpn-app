import React, { useEffect, useRef, useState } from 'react';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import { InAppNotificationIndicatorType } from '../../shared/notifications/notification';
import { useStyledRef } from '../lib/utilityHooks';
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

export const NotificationActionButton = styled(AppButton.SimpleButton)({
  flex: 1,
  justifyContent: 'center',
  cursor: 'default',
  padding: '4px',
  background: 'transparent',
  border: 'none',
});

export const NotificationActionButtonInner = styled(ImageView)({
  [NotificationActionButton + ':hover &&']: {
    backgroundColor: colors.white80,
  },
});

interface NotificationActionProps {
  onClick: () => Promise<void>;
}

export function NotificationOpenLinkAction(props: NotificationActionProps) {
  return (
    <AppButton.BlockingButton onClick={props.onClick}>
      <NotificationActionButton
        aria-describedby={NOTIFICATION_AREA_ID}
        aria-label={messages.gettext('Open URL')}>
        <NotificationActionButtonInner
          height={12}
          width={12}
          tintColor={colors.white60}
          source="icon-extLink"
        />
      </NotificationActionButton>
    </AppButton.BlockingButton>
  );
}

export function NotificationTroubleshootDialogAction(props: NotificationActionProps) {
  return (
    <NotificationActionButton
      aria-describedby={NOTIFICATION_AREA_ID}
      aria-label={messages.gettext('Troubleshoot')}
      onClick={props.onClick}>
      <NotificationActionButtonInner
        height={12}
        width={12}
        tintColor={colors.white60}
        source="icon-info"
      />
    </NotificationActionButton>
  );
}

export function NotificationCloseAction(props: NotificationActionProps) {
  return (
    <NotificationActionButton
      aria-describedby={NOTIFICATION_AREA_ID}
      aria-label={messages.pgettext('accessibility', 'Close notification')}
      onClick={props.onClick}>
      <NotificationActionButtonInner source="icon-close" width={16} tintColor={colors.white60} />
    </NotificationActionButton>
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
  success: colors.green,
  warning: colors.yellow,
  error: colors.red,
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
    backgroundColor: 'rgba(25, 38, 56, 0.95)',
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

  // Save last non-undefined children to be able to show them during the hide-transition.
  const prevChildren = useRef<React.ReactNode>();
  useEffect(() => {
    prevChildren.current = props.children ?? prevChildren.current;
  }, [props.children]);

  useEffect(() => {
    const newHeight =
      props.children !== undefined ? contentRef.current?.getBoundingClientRect().height ?? 0 : 0;
    if (newHeight !== contentHeight) {
      setContentHeight(newHeight);
      setAlignBottom((alignBottom) => alignBottom || contentHeight === 0 || newHeight === 0);
    }
  });

  return (
    <Collapsible $height={contentHeight} className={props.className} $alignBottom={alignBottom}>
      <Content ref={contentRef}>{props.children ?? prevChildren.current}</Content>
    </Collapsible>
  );
}
