import React from 'react';

export const useIsDefaultActiveElementAfterMount = () => {
  const [isDefaultActiveElementAfterMount, setIsDefaultActiveElementAfterMount] = React.useState<
    boolean | undefined
  >(undefined);

  React.useEffect(() => {
    if (typeof document !== 'undefined') {
      const isBodyOrDocumentElement =
        document.activeElement === document.body ||
        document.activeElement === document.documentElement;

      setIsDefaultActiveElementAfterMount(isBodyOrDocumentElement);
    }

    return () => {
      console.log('umount');
      setIsDefaultActiveElementAfterMount(undefined);
    };
  }, []);

  console.log(
    'isDefaultActiveElementAfterMount',
    isDefaultActiveElementAfterMount,
    document.activeElement,
  );

  return isDefaultActiveElementAfterMount;
};
