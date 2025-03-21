import { AnimatePresentVertical, AnimatePresentVerticalProps } from './components';

export type AnimateProps = {
  type: 'present-vertical';
} & AnimatePresentVerticalProps;

export const Animate = ({ type, ...props }: AnimateProps) => {
  switch (type) {
    case 'present-vertical':
      return <AnimatePresentVertical {...props} />;
    default:
      return null;
  }
};
