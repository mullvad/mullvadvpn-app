import { AnimatePresentVertical, AnimatePresentVerticalProps } from './components';

export type AnimateProps = {
  type: 'present-vertical';
} & AnimatePresentVerticalProps;

export function Animate({ type, ...props }: AnimateProps) {
  switch (type) {
    case 'present-vertical':
      return <AnimatePresentVertical {...props} />;
    default:
      return type satisfies never;
  }
}
