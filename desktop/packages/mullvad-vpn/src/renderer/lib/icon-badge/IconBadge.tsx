import { Image } from '../components';

export interface IconBadgeProps {
  state: 'positive' | 'negative';
}

const sources = {
  positive: 'positive',
  negative: 'negative',
};

export const IconBadge = ({ state }: IconBadgeProps) => {
  return <Image source={sources[state]} width={48} height={48} />;
};
