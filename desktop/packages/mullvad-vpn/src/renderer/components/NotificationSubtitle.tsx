import React from 'react';
import styled from 'styled-components';

import { InAppNotification, InAppNotificationSubtitle } from '../../shared/notifications';
import { LabelTinySemiBold } from '../lib/components';
import { Link } from '../lib/components/link';
import { formatHtml } from '../lib/html-formatter';
import { ExternalLink } from './ExternalLink';
import { InternalLink } from './InternalLink';

export type NotificationSubtitleProps = Pick<InAppNotification, 'subtitle'>;

const StyledLink = styled(Link)``;

const formatSubtitle = (subtitle: InAppNotificationSubtitle) => {
  const content = formatHtml(subtitle.content);
  if (subtitle.action) {
    switch (subtitle.action.type) {
      case 'navigate-internal':
        return (
          <InternalLink variant="labelTinySemiBold" {...subtitle.action.link}>
            <InternalLink.Text>{content}</InternalLink.Text>
          </InternalLink>
        );
      case 'navigate-external':
        return (
          <ExternalLink variant="labelTinySemiBold" {...subtitle.action.link}>
            <ExternalLink.Text>{content}</ExternalLink.Text>
            <ExternalLink.Icon icon="external" />
          </ExternalLink>
        );
      case 'run-function':
        return (
          <StyledLink forwardedAs="button" variant="labelTinySemiBold" {...subtitle.action.button}>
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
      <LabelTinySemiBold color="whiteAlpha60" {...props}>
        {formatHtml(subtitle)}
      </LabelTinySemiBold>
    );
  }

  return (
    <LabelTinySemiBold color="whiteAlpha60" {...props}>
      {subtitle.map((subtitle, index, arr) => {
        const content = formatSubtitle(subtitle);

        return (
          <React.Fragment key={subtitle.content}>
            {content}
            {index !== arr.length - 1 && ' '}
          </React.Fragment>
        );
      })}
    </LabelTinySemiBold>
  );
};
