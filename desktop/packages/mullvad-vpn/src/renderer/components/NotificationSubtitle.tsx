import React from 'react';
import styled from 'styled-components';

import { InAppNotificationSubtitle } from '../../shared/notifications';
import { Icon, LabelTiny } from '../lib/components';
import { Colors } from '../lib/foundations';
import { formatHtml } from '../lib/html-formatter';
import { ExternalLink } from './ExternalLink';
import { InternalLink } from './InternalLink';

export type NotificationSubtitleProps = {
  subtitle?: string | InAppNotificationSubtitle[];
};

const StyledIcon = styled(Icon)`
  display: inline-flex;
  vertical-align: middle;
`;

const formatSubtitle = (subtitle: InAppNotificationSubtitle) => {
  const content = formatHtml(subtitle.content);
  if (subtitle.action) {
    switch (subtitle.action.type) {
      case 'navigate-internal':
        return (
          <InternalLink variant="labelTiny" {...subtitle.action.link}>
            {content}
          </InternalLink>
        );
      case 'navigate-external':
        return (
          <ExternalLink variant="labelTiny" {...subtitle.action.link}>
            {content}
            <StyledIcon icon="external" size="small" />
          </ExternalLink>
        );
      default:
        break;
    }
  }
  return content;
};

export const NotificationSubtitle = ({ subtitle, ...props }: NotificationSubtitleProps) => {
  if (!subtitle) {
    return null;
  }

  if (!Array.isArray(subtitle)) {
    return (
      <LabelTiny color={Colors.white60} {...props}>
        {formatHtml(subtitle)}
      </LabelTiny>
    );
  }

  return (
    <LabelTiny color={Colors.white60} {...props}>
      {subtitle.map((subtitle, index, arr) => {
        const content = formatSubtitle(subtitle);

        return (
          <React.Fragment key={subtitle.content}>
            {content}
            {index !== arr.length - 1 && ' '}
          </React.Fragment>
        );
      })}
    </LabelTiny>
  );
};
