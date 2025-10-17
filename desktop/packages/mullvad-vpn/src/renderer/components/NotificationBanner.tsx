import { motion } from 'motion/react';
import React from 'react';
import styled from 'styled-components';

import { messages } from '../../shared/gettext';
import { InAppNotificationIndicatorType } from '../../shared/notifications/notification';
import { IconButton } from '../lib/components';
import { colors } from '../lib/foundations';
import { useExclusiveTask } from '../lib/hooks/use-exclusive-task';
import { tinyText } from './common-styles';

const NOTIFICATION_AREA_ID = 'notification-area';

export const NotificationTitle = styled.span(tinyText, {
  color: colors.white,
});

export const NotificationSubtitleText = styled.span(tinyText, {
  color: colors.whiteAlpha60,
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
  backgroundColor: props.$type
    ? notificationIndicatorTypeColorMap[props.$type]
    : colors.transparent,
}));

const Collapsible = styled(motion.div)({
  display: 'flex',
  flexDirection: 'column',
  justifyContent: 'flex-start',
  translateY: '0%',
  backgroundColor: colors.darkerBlue50,
  overflow: 'hidden',
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
  animateIn: boolean;
}

export function NotificationBanner({ className, children, animateIn }: INotificationBannerProps) {
  const translateYInitial = animateIn ? '-100%' : '0%';

  return (
    <Collapsible
      animate={{ translateY: '0%' }}
      className={className}
      exit={{ translateY: '-100%' }}
      initial={{ translateY: translateYInitial }}
      transition={{ duration: 0.25 }}>
      <Content>{children}</Content>
    </Collapsible>
  );
}
