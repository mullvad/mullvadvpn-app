import * as React from 'react';
import { createPortal } from 'react-dom';

type DialogPortalProps = {
  children: React.ReactNode;
  containerId?: string;
};

export function DialogPortal({ children, containerId = 'modal-container' }: DialogPortalProps) {
  const [container, setContainer] = React.useState<HTMLElement | null>(null);

  React.useEffect(() => {
    setContainer(document.getElementById(containerId) ?? document.body);
  }, [containerId]);

  if (!container) return null;

  return createPortal(children, container);
}

export type DialogItemProps = React.PropsWithChildren<React.ComponentPropsWithoutRef<'dialog'>>;
