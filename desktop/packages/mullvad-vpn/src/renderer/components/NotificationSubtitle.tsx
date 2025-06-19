import React from 'react';
import styled from 'styled-components';

import { InAppNotificationSubtitle } from '../../shared/notifications';
import { LabelTiny, Link } from '../lib/components';
import { formatHtml } from '../lib/html-formatter';
import { ExternalLink } from './ExternalLink';
import { InternalLink } from './InternalLink';

export type NotificationSubtitleProps = {
  subtitle?: string | InAppNotificationSubtitle[];
};

const StyledLink = styled(Link)``;

const formatSubtitle = (subtitle: InAppNotificationSubtitle) => {
  const content = formatHtml(subtitle.content);
  if (subtitle.action) {
    switch (subtitle.action.type) {
      case 'navigate-internal':
        return (
          <InternalLink variant="labelTiny" {...subtitle.action.link}>
            <InternalLink.Text>{content}</InternalLink.Text>
          </InternalLink>
        );
      case 'navigate-external':
        return (
          <ExternalLink variant="labelTiny" {...subtitle.action.link}>
            <ExternalLink.Text>{content}</ExternalLink.Text>
            <ExternalLink.Icon icon="external" />
          </ExternalLink>
        );
      case 'run-function':
        return (
          <StyledLink
            forwardedAs="button"
            color="white"
            variant="labelTiny"
            {...subtitle.action.button}>
            <StyledLink.Text>{content}</StyledLink.Text>
          </StyledLink>
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
      <LabelTiny color="whiteAlpha60" {...props}>
        {formatHtml(subtitle)}
      </LabelTiny>
    );
  }

  return (
    <LabelTiny color="whiteAlpha60" {...props}>
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
