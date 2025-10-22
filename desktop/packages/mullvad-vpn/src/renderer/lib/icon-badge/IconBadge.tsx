import styled from 'styled-components';

import { Image, ImageProps } from '../components';

export type IconBadgeProps = Omit<ImageProps, 'source'> & {
  state: 'positive' | 'negative';
};

const sources = {
  positive: 'positive',
  negative: 'negative',
};

export const StyledIconBadge = styled(Image)``;

export const IconBadge = ({ state, ...props }: IconBadgeProps) => {
  return (
    <StyledIconBadge source={sources[state]} width={48} height={48} aria-hidden="true" {...props} />
  );
};
