import * as React from 'react';
import { createPortal } from 'react-dom';

import { useEffectEvent } from '../../../../utility-hooks';

type DialogPortalProps = {
  children: React.ReactNode;
  containerId?: string;
};

export function DialogPortal({ children, containerId = 'modal-container' }: DialogPortalProps) {
  const [container, setContainer] = React.useState<HTMLElement | null>(null);

  const setContainerEffectEvent = useEffectEvent((id: string) => {
    setContainer(document.getElementById(id) ?? document.body);
  });

  React.useEffect(() => {
    setContainerEffectEvent(containerId);
  }, [containerId]);

  if (!container) return null;

  return createPortal(children, container);
}

export type DialogItemProps = React.PropsWithChildren<React.ComponentPropsWithoutRef<'dialog'>>;
