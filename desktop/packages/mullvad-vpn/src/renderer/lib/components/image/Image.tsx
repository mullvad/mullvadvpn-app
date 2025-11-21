import React from 'react';

export type ImageProps = Omit<React.ComponentPropsWithRef<'img'>, 'src'> & {
  source: string;
};

export const Image = ({ source, ...props }: ImageProps) => {
  const url = source.startsWith('data:') ? source : `assets/images/${source}.svg`;

  return <img src={url} {...props} />;
};
