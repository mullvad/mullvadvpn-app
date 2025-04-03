import React from 'react';

import { InAppNotificationSubtitle } from '../../shared/notifications';
import { LabelTiny } from '../lib/components';
import { Colors } from '../lib/foundations';
import { formatHtml } from '../lib/html-formatter';
import { ExternalLink } from './ExternalLink';
import { InternalLink } from './InternalLink';
import styled from 'styled-components';

export type NotificationSubtitleProps = {
  subtitle?: string | InAppNotificationSubtitle[];
};

const StyledExternalLink = styled(ExternalLink)`
  display: flex;
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
          <StyledExternalLink variant="labelTiny" {...subtitle.action.link}>
            {content}
            <ExternalLink.Icon icon="external" size="small" />
          </StyledExternalLink>
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
