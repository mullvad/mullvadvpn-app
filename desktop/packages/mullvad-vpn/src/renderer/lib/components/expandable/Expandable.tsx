import { AnimatePresence, type AnimatePresenceProps } from 'motion/react';

import { ExpandableContent } from './components';

export type ExpandableProps = React.PropsWithChildren<AnimatePresenceProps> & {
  expanded?: boolean;
};

function Expandable({ expanded, children, ...props }: ExpandableProps) {
  return (
    <AnimatePresence initial={false} {...props}>
      {expanded && children}
    </AnimatePresence>
  );
}

const ExpandableNamespace = Object.assign(Expandable, {
  Content: ExpandableContent,
});

export { ExpandableNamespace as Expandable };
